use crate::{AxumDatabasePool, SessionError};
use async_trait::async_trait;
use chrono::Utc;

/// Null Pool type for AxumDatabasePool.
/// Use this when you do not want to load any database.
pub struct AxumNullPool;

#[async_trait]
impl AxumDatabasePool for AxumNullPool {
    async fn migrate(&self, table_name: &str) -> Result<(), SessionError> {
        Ok(())
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<(), SessionError> {
        Ok(())
    }

    async fn count(&self, table_name: &str) -> Result<i64, SessionError> {
        return Ok(0);
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, SessionError> {
        Ok(None)
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), SessionError> {
        Ok(())
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), SessionError> {
        Ok(())
    }
}
