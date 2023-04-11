use crate::{DatabasePool,  SessionError, SessionStore};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::BTreeMap;
use surrealdb::{sql::Value, Datastore, Error, Response, dbs::Session};

pub type SessionSurrealSession = crate::Session<SessionSurrealPool>;
pub type SessionSurrealSessionStore = SessionStore<SessionSurrealPool>;

///Surreal internal Managed Pool type for DatabasePool
#[derive(Debug, Clone)]
pub struct SessionSurrealPool {
    connection_type: ConnectionType,
    session: Session,
}

enum ConnectionType {
    #[cfg(feature = "surrealdb-mem")]
    Memory,
    #[cfg(feature = "surrealdb-rocksdb")]
    File(String),
    #[cfg(feature = "surrealdb-rocksdb")]
    Rockdb(String),
    #[cfg(feature = "surrealdb-tikv")]
    TiKV(String),
    #[cfg(feature = "surrealdb-indxdb")]
    Indxdb(String),
    #[cfg(feature = "fdb_tag")]
    Fdb(String),
}

pub(crate) struct Connection {
    ds: Datastore,
    ses: Session,
}

impl SessionSurrealPool {
    #[cfg(feature = "surrealdb-mem")]
    pub async fn memory(session: Session) -> Self {
        Self {
            session,
            connection_type: ConnectionType::Memory,
        }
    }

    #[cfg(feature = "surrealdb-rocksdb")]
    pub async fn file(path: impl AsRef<str>, session: Session) -> Self {
        Self {
            session,
            connection_type: ConnectionType::File(format!("file://{}", path.as_ref())),
        }
    }

    #[cfg(feature = "surrealdb-tikv")]
    pub async fn tikv(uri: impl AsRef<str>, session: Session) -> Self {
        Self {
            session,
            connection_type: ConnectionType::TiKV(format!("tikv://{}", uri.as_ref())),
        }
    }

    #[cfg(feature = "surrealdb-rocksdb")]
    pub async fn rockdb(uri: impl AsRef<str>, session: Session) -> Self {
        Self {
            session,
            connection_type: ConnectionType::Rockdb(format!("rocksdb://{}", uri.as_ref())),
        }
    }

    #[cfg(feature = "surrealdb-indxdb")]
    pub async fn indxdb(uri: impl AsRef<str>, session: Session) -> Self {
        Self {
            session,
            connection_type: ConnectionType::Indxdb(format!("indxdb://{}", uri.as_ref())),
        }
    }

    #[cfg(feature = "fdb_tag")]
    pub async fn fdb(uri: impl AsRef<str>, session: Session) -> Self {
        Self {
            session,
            connection_type: ConnectionType::Fdb(format!("fdb://{}", uri.as_ref())),
        }
    }

    pub(crate) async fn connect(&self) -> Result<Connection, SessionError> {
        Ok(Connection {
            ds: match &self.connection_type {
                #[cfg(feature = "surrealdb-mem")]
                ConnectionType::Memory => Datastore::new("memory").await?,
                #[cfg(feature = "surrealdb-rocksdb")]
                ConnectionType::File(path) => Datastore::new(path.as_ref()).await?,
                #[cfg(feature = "surrealdb-tikv")]
                ConnectionType::TiKV(uri) => Datastore::new(uri.as_ref()).await?,
                #[cfg(feature = "surrealdb-rocksdb")]
                ConnectionType::Rockdb(uri) => Datastore::new(uri.as_ref()).await?,
                #[cfg(feature = "surrealdb-indxdb")]
                ConnectionType::Indxdb(uri) => Datastore::new(uri.as_ref()).await?,
                #[cfg(feature = "fdb_tag")]
                ConnectionType::Fdb(uri) => Datastore::new(uri.as_ref()).await?,
            },
            ses: self.session.clone(),
        })
    }

    pub async fn is_valid(&self) -> Result<(), SessionError> {
        let connection = self.connect().await?;
        connection.execute("SELECT * FROM 1;", None, false).await?;
        Ok(())
    }
}

#[async_trait]
impl DatabasePool for SessionSurrealPool {
    async fn initiate(&self, table_name: &str) -> Result<(), SessionError> {
        let conn = self.connect().await?;

        conn.ds
            .execute(
                &r#"
                    DEFINE TABLE %%TABLE_NAME%% SCHEMAFULL;
                    DEFINE FIELD id ON TABLE %%TABLE_NAME%% TYPE string ASSERT $value != NONE;
                    DEFINE FIELD expire ON TABLE %%TABLE_NAME%% TYPE int;
                    DEFINE FIELD session ON TABLE %%TABLE_NAME%% TYPE string ASSERT $value != NONE;
                    DEFINE INDEX %%TABLE_NAME%%IdIndex ON TABLE %%TABLE_NAME%% COLUMNS id UNIQUE;
                "#
                .replace("%%TABLE_NAME%%", table_name),
                &ses,
                None,
                false,
            )
            .await?;

        Ok(())
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<(), SessionError> {
        let conn = self.connect().await?;
        let mut vars = BTreeMap::<String, Value>::new();

        vars.insert("expires".to_string(), Utc::now().timestamp().into());

        conn.ds
            .execute(
                &r#"DELETE %%TABLE_NAME%% WHERE expires < $expires"#
                    .replace("%%TABLE_NAME%%", table_name),
                &ses,
                Some(vars),
                false,
            )
            .await?;

        Ok(())
    }

    async fn count(&self, table_name: &str) -> Result<i64, SessionError> {
        let conn = self.connect().await?;

        let mut res = conn
            .ds
            .execute(
                &r#"SELECT COUNT() FROM %%TABLE_NAME%%"#.replace("%%TABLE_NAME%%", table_name),
                &ses,
                None,
                false,
            )
            .await?;

        if let Some(response) = res.pop() {
            Ok(response.result?.as_int())
        } else {
            Ok(0)
        }
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), SessionError> {
        let conn = self.connect().await?;
        let mut vars = BTreeMap::<String, Value>::new();

        vars.insert("id".to_string(), id.into());
        vars.insert("session".to_string(), session.into());
        vars.insert("expires".to_string(), expires.into());

        conn.ds
            .execute(
                &r#"
            INSERT INTO %%TABLE_NAME%%
                (id, session, expires) VALUES $id, $session, $expires
                ON DUPLICATE KEY UPDATE
                expires = $expires,
                session = $session
        "#
                .replace("%%TABLE_NAME%%", table_name),
                &ses,
                Some(vars),
                false,
            )
            .await?;

        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, SessionError> {
        let conn = self.connect().await?;
        let mut vars = BTreeMap::<String, Value>::new();

        vars.insert("id".to_string(), id.into());
        vars.insert("expires".to_string(), Utc::now().timestamp().into());

        let mut res = conn
            .ds
            .execute(
                &r#"
                SELECT session FROM %%TABLE_NAME%%
                WHERE id = $id AND (expires = NONE OR expires > $expires)
            "#
                .replace("%%TABLE_NAME%%", table_name),
                &ses,
                Some(vars),
                false,
            )
            .await?;

        if let Some(response) = res.pop() {
            Ok(Some(response.result?.as_string()))
        } else {
            Ok(None)
        }
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), SessionError> {
        let conn = self.connect().await?;
        let mut vars = BTreeMap::<String, Value>::new();

        vars.insert("id".to_string(), id.into());

        conn.ds
            .execute(
                &r#"DELETE %%TABLE_NAME%% WHERE id < $id"#.replace("%%TABLE_NAME%%", table_name),
                &ses,
                Some(vars),
                false,
            )
            .await?;

        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, SessionError> {
        let conn = self.connect().await?;
        let mut vars = BTreeMap::<String, Value>::new();

        vars.insert("id".to_string(), id.into());
        vars.insert("expires".to_string(), Utc::now().timestamp().into());

        let mut res = conn
            .ds
            .execute(
                &r#"SELECT COUNT() FROM %%TABLE_NAME%% WHERE id = $id AND (expires = NONE OR expires > $expires)"#.replace("%%TABLE_NAME%%", table_name),
                &ses,
                Some(vars),
                false,
            )
            .await?;

        if let Some(response) = res.pop() {
            Ok(response.result?.as_int() > 0)
        } else {
            Ok(false)
        }
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), SessionError> {
        let conn = self.connect().await?;

        conn.ds
            .execute(
                &r#"DELETE %%TABLE_NAME%%"#.replace("%%TABLE_NAME%%", table_name),
                &ses,
                None,
                false,
            )
            .await?;

        Ok(())
    }
}
