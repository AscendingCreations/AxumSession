use crate::{DatabasePool, Session, SessionError, SessionStore};
use async_trait::async_trait;

///Null's Session Helper type for a DatabaseLess Session.
pub type SessionNullSession = Session<SessionNullPool>;
///Null's Session Store Helper type for a DatabaseLess Session.
pub type SessionNullSessionStore = SessionStore<SessionNullPool>;

/// Null Pool type for a DatabaseLess Session.
/// Use this when you do not want to load any database.
#[derive(Debug, Clone)]
pub struct SessionNullPool;

#[async_trait]
impl DatabasePool for SessionNullPool {
    async fn initiate(&self, _table_name: &str) -> Result<(), SessionError> {
        Ok(())
    }

    async fn delete_by_expiry(&self, _table_name: &str) -> Result<Vec<String>, SessionError> {
        Ok(Vec::new())
    }

    async fn count(&self, _table_name: &str) -> Result<i64, SessionError> {
        return Ok(0);
    }

    async fn store(
        &self,
        _id: &str,
        _session: &str,
        _expires: i64,
        _table_name: &str,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    async fn load(&self, _id: &str, _table_name: &str) -> Result<Option<String>, SessionError> {
        Ok(None)
    }

    async fn delete_one_by_id(&self, _id: &str, _table_name: &str) -> Result<(), SessionError> {
        Ok(())
    }

    async fn exists(&self, _id: &str, _table_name: &str) -> Result<bool, SessionError> {
        Ok(false)
    }

    async fn delete_all(&self, _table_name: &str) -> Result<(), SessionError> {
        Ok(())
    }

    async fn get_ids(&self, _table_name: &str) -> Result<Vec<String>, SessionError> {
        Ok(Vec::new())
    }

    fn auto_handles_expiry(&self) -> bool {
        false
    }
}
