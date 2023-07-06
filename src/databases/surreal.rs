use crate::{DatabasePool, SessionError, SessionStore};
use async_trait::async_trait;
use chrono::Utc;
use surrealdb::{Connection, Surreal};

///Surreal's Session Helper type for the DatabasePool.
pub type SessionSurrealSession<C> = crate::Session<SessionSurrealPool<C>>;
///Surreal's Session Store Helper type for the DatabasePool.
pub type SessionSurrealSessionStore<C> = SessionStore<SessionSurrealPool<C>>;

///Surreal internal Managed Pool type for DatabasePool
/// Please refer to https://docs.rs/surrealdb/1.0.0-beta.9+20230402/surrealdb/struct.Surreal.html#method.new
#[derive(Debug)]
pub struct SessionSurrealPool<C: Connection> {
    connection: Surreal<C>,
}

// We do this to avoid Any needing Clone when being used in the Type traits.
impl<C> Clone for SessionSurrealPool<C>
where
    C: Connection,
{
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone(),
        }
    }
}

impl<C: Connection> From<Surreal<C>> for SessionSurrealPool<C> {
    fn from(connection: Surreal<C>) -> Self {
        SessionSurrealPool { connection }
    }
}

impl<C: Connection> SessionSurrealPool<C> {
    /// Creates a New Session pool from a Connection.
    /// Please refer to https://docs.rs/surrealdb/1.0.0-beta.9+20230402/surrealdb/struct.Surreal.html#method.new
    pub fn new(connection: Surreal<C>) -> Self {
        Self { connection }
    }

    pub async fn is_valid(&self) -> Result<(), SessionError> {
        self.connection.query("SELECT * FROM 1;").await?;
        Ok(())
    }
}

#[async_trait]
impl<C: Connection> DatabasePool for SessionSurrealPool<C> {
    async fn initiate(&self, table_name: &str) -> Result<(), SessionError> {
        self.connection
            .query(
                &r#"
                    DEFINE TABLE %%TABLE_NAME%% SCHEMAFULL;
                    DEFINE FIELD sessionid ON TABLE %%TABLE_NAME%% TYPE string ASSERT $value != NONE;
                    DEFINE FIELD sessionexpires ON TABLE %%TABLE_NAME%% TYPE int;
                    DEFINE FIELD sessionstore ON TABLE %%TABLE_NAME%% TYPE string ASSERT $value != NONE;
                    DEFINE INDEX %%TABLE_NAME%%IdIndex ON TABLE %%TABLE_NAME%% COLUMNS sessionid UNIQUE;
                "#
                .replace("%%TABLE_NAME%%", table_name),
            )
            .await?;

        Ok(())
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<Vec<String>, SessionError> {
        let mut res = self
            .connection
            .query(
                &r#"
            SELECT sessionid FROM %%TABLE_NAME%%
                WHERE sessionexpires = NONE OR sessionexpires > $expire;
        "#
                .replace("%%TABLE_NAME%%", table_name),
            )
            .bind(("expire", Utc::now().timestamp()))
            .await?;

        let ids: Vec<String> = res.take("sessionid")?;

        self.connection
            .query(
                &r#"DELETE %%TABLE_NAME%% WHERE sessionexpires < $expire;"#
                    .replace("%%TABLE_NAME%%", table_name),
            )
            .bind(("expire", Utc::now().timestamp()))
            .await?;

        Ok(ids)
    }

    async fn count(&self, table_name: &str) -> Result<i64, SessionError> {
        let mut res = self
            .connection
            .query(
                &r#"SELECT count() AS amount FROM %%TABLE_NAME%% GROUP BY amount;"#
                    .replace("%%TABLE_NAME%%", table_name),
            )
            .await?;

        let response: Option<i64> = res.take("amount")?;
        if let Some(count) = response {
            Ok(count)
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
        self.connection
            .query(
                &r#"
                DELETE %%TABLE_NAME%% WHERE sessionid=$session_id;
                INSERT INTO %%TABLE_NAME%%
                (sessionid, sessionstore, sessionexpires) VALUES ($session_id, $store, $expire);
        "#
                .replace("%%TABLE_NAME%%", table_name),
            )
            .bind(("session_id", id))
            .bind(("store", session))
            .bind(("expire", expires))
            .await?;

        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, SessionError> {
        let mut res = self
            .connection
            .query(
                &r#"
                SELECT sessionstore FROM %%TABLE_NAME%%
                WHERE sessionid = $session_id AND (sessionexpires = NONE OR sessionexpires > $expire);
            "#
                .replace("%%TABLE_NAME%%", table_name),
            ).bind(("session_id", id))
            .bind(("expire", Utc::now().timestamp()))
            .await?;

        let response: Option<String> = res.take("sessionstore")?;
        Ok(response)
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), SessionError> {
        self.connection
            .query(
                &r#"DELETE %%TABLE_NAME%% WHERE sessionid < $session_id;"#
                    .replace("%%TABLE_NAME%%", table_name),
            )
            .bind(("session_id", id))
            .await?;

        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, SessionError> {
        self.connection.set("id".to_string(), id).await?;
        self.connection
            .set("expires".to_string(), Utc::now().timestamp())
            .await?;

        let mut res = self
            .connection
            .query(
                &r#"
                SELECT count() AS amount FROM %%TABLE_NAME%% WHERE sessionid = $session_id AND 
                (sessionexpires = NONE OR sessionexpires > $expire);
                "#
                .replace("%%TABLE_NAME%%", table_name),
            )
            .bind(("session_id", id))
            .bind(("expire", Utc::now().timestamp()))
            .await?;

        let response: Option<i64> = res.take("amount")?;
        Ok(response.map(|f| f > 0).unwrap_or_default())
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), SessionError> {
        self.connection
            .query(&r#"DELETE %%TABLE_NAME%%;"#.replace("%%TABLE_NAME%%", table_name))
            .await?;

        Ok(())
    }

    async fn get_ids(&self, table_name: &str) -> Result<Vec<String>, SessionError> {
        let mut res = self
            .connection
            .query(
                &r#"
            SELECT sessionid FROM %%TABLE_NAME%%
                WHERE sessionexpires = NONE OR sessionexpires > $expires;
        "#
                .replace("%%TABLE_NAME%%", table_name),
            )
            .bind(("expire", Utc::now().timestamp()))
            .await?;

        let ids: Vec<String> = res.take("sessionid")?;

        Ok(ids)
    }

    fn auto_handles_expiry(&self) -> bool {
        false
    }
}
