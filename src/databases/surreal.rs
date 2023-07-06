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
    async fn initiate(&self, _table_name: &str) -> Result<(), SessionError> {
        Ok(())
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<Vec<String>, SessionError> {
        let mut res = self
            .connection
            .query(
                "SELECT sessionid FROM type::table($table_name)
                WHERE sessionexpires = NONE OR sessionexpires > $expires;",
            )
            .bind(("table_name", table_name))
            .await?;

        let ids: Vec<String> = res.take("sessionid")?;

        self.connection
            .query("DELETE type::table($table_name) WHERE sessionexpires < $expires;")
            .bind(("table_name", table_name))
            .bind(("expires", Utc::now().timestamp()))
            .await?;

        Ok(ids)
    }

    async fn count(&self, table_name: &str) -> Result<i64, SessionError> {
        let mut res = self
            .connection
            .query("SELECT count() AS amount FROM type::table($table_name) GROUP BY amount;")
            .bind(("table_name", table_name))
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
            "UPDATE type::thing($table_name, $session_id) SET sessionstore = $store, sessionexpires = $expire, sessionid = $session_id;",
        )
        .bind(("table_name", table_name))
        .bind(("session_id", id.to_string()))
        .bind(("expire", expires.to_string()))
        .bind(("store", session))
        .await?;

        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, SessionError> {
        let mut res = self
            .connection
            .query(
                "SELECT sessionstore FROM type::thing($table_name, $session_id)
                WHERE sessionexpires = NONE OR sessionexpires > $expires;",
            )
            .bind(("table_name", table_name))
            .bind(("session_id", id))
            .bind(("expires", Utc::now().timestamp()))
            .await
            .unwrap();

        let response: Option<String> = res.take("sessionstore")?;
        Ok(response)
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), SessionError> {
        self.connection
            .query("DELETE type::table($table_name) WHERE sessionid < $session_id;")
            .bind(("table_name", table_name))
            .bind(("session_id", id))
            .await?;

        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, SessionError> {
        let mut res = self
            .connection
            .query(
                "SELECT count() AS amount FROM type::thing($table_name, $session_id) 
                WHERE sessionexpires = NONE OR sessionexpires > $expires GROUP BY amount;",
            )
            .bind(("table_name", table_name))
            .bind(("session_id", id))
            .bind(("expires", Utc::now().timestamp()))
            .await?;

        let response: Option<i64> = res.take("amount")?;
        Ok(response.map(|f| f > 0).unwrap_or_default())
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), SessionError> {
        self.connection
            .query("DELETE type::table($table_name);")
            .bind(("table_name", table_name))
            .await?;

        Ok(())
    }

    async fn get_ids(&self, table_name: &str) -> Result<Vec<String>, SessionError> {
        let mut res = self
            .connection
            .query(
                "SELECT sessionid FROM type::table($table_name)
                WHERE sessionexpires = NONE OR sessionexpires > $expires;",
            )
            .bind(("table_name", table_name))
            .bind(("expires", Utc::now().timestamp()))
            .await?;

        let ids: Vec<String> = res.take("sessionid")?;
        Ok(ids)
    }

    fn auto_handles_expiry(&self) -> bool {
        false
    }
}
