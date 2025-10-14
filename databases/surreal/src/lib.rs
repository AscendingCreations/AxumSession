#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![warn(clippy::all, nonstandard_style, future_incompatible)]
#![forbid(unsafe_code)]

use async_trait::async_trait;
use axum_session::{DatabaseError, DatabasePool, Session, SessionStore};
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

    pub async fn is_valid(&self) -> Result<(), DatabaseError> {
        self.connection
            .query("SELECT * FROM 1;")
            .await
            .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl<C: Connection> DatabasePool for SessionSurrealPool<C> {
    async fn initiate(&self, _table_name: &str) -> Result<(), DatabaseError> {
        Ok(())
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        let now = Utc::now().timestamp();

        let mut res = self
            .connection
            .query(
                "SELECT sessionid FROM type::table($table_name)
                WHERE sessionexpires = NONE OR type::number(sessionexpires) < $expires;",
            )
            .bind(("table_name", table_name.to_string()))
            .bind(("expires", now))
            .await
            .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        let ids: Vec<String> = res
            .take("sessionid")
            .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        self.connection
            .query(
                "DELETE type::table($table_name)
                WHERE sessionexpires = NONE OR type::number(sessionexpires) < $expires;",
            )
            .bind(("table_name", table_name.to_string()))
            .bind(("expires", now))
            .await
            .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;

        Ok(ids)
    }

    async fn count(&self, table_name: &str) -> Result<i64, DatabaseError> {
        let mut res = self
            .connection
            .query("SELECT count() AS amount FROM type::table($table_name) GROUP BY amount;")
            .bind(("table_name", table_name.to_string()))
            .await
            .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        let response: Option<i64> = res
            .take("amount")
            .map_err(|err| DatabaseError::GenericNotSupportedError(err.to_string()))?;
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
    ) -> Result<(), DatabaseError> {
        self.connection
        .query(
            "UPSERT type::thing($table_name, $session_id) SET sessionstore = $store, sessionexpires = $expire, sessionid = $session_id;",
        )
        .bind(("table_name", table_name.to_string()))
        .bind(("session_id", id.to_string()))
        .bind(("expire", expires.to_string()))
        .bind(("store", session.to_string()))
        .await.map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, DatabaseError> {
        let mut res = self
            .connection
            .query(
                "SELECT sessionstore FROM type::thing($table_name, $session_id)
                WHERE sessionexpires = NONE OR sessionexpires > $expires;",
            )
            .bind(("table_name", table_name.to_string()))
            .bind(("session_id", id.to_string()))
            .bind(("expires", Utc::now().timestamp()))
            .await
            .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        let response: Option<String> = res
            .take("sessionstore")
            .map_err(|err| DatabaseError::GenericNotSupportedError(err.to_string()))?;
        Ok(response)
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), DatabaseError> {
        self.connection
            .query("DELETE type::table($table_name) WHERE sessionid < $session_id;")
            .bind(("table_name", table_name.to_string()))
            .bind(("session_id", id.to_string()))
            .await
            .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;

        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, DatabaseError> {
        let mut res = self
            .connection
            .query(
                "SELECT count() AS amount FROM type::thing($table_name, $session_id)
                WHERE sessionexpires = NONE OR sessionexpires > $expires GROUP BY amount;",
            )
            .bind(("table_name", table_name.to_string()))
            .bind(("session_id", id.to_string()))
            .bind(("expires", Utc::now().timestamp()))
            .await
            .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        let response: Option<i64> = res
            .take("amount")
            .map_err(|err| DatabaseError::GenericNotSupportedError(err.to_string()))?;
        Ok(response.map(|f| f > 0).unwrap_or_default())
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), DatabaseError> {
        self.connection
            .query("DELETE type::table($table_name);")
            .bind(("table_name", table_name.to_string()))
            .await
            .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;

        Ok(())
    }

    async fn get_ids(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        let mut res = self
            .connection
            .query(
                "SELECT sessionid FROM type::table($table_name)
                WHERE sessionexpires = NONE OR sessionexpires > $expires;",
            )
            .bind(("table_name", table_name.to_string()))
            .bind(("expires", Utc::now().timestamp()))
            .await
            .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        let ids: Vec<String> = res
            .take("sessionid")
            .map_err(|err| DatabaseError::GenericNotSupportedError(err.to_string()))?;
        Ok(ids)
    }

    fn auto_handles_expiry(&self) -> bool {
        false
    }
}
