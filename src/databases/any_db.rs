use crate::{DatabaseError, DatabasePool, Session, SessionStore};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
///Any Session Helper type for the DatabasePool.
pub type SessionAnySession = Session<SessionAnyPool>;
///Any Session Store Helper type for the DatabasePool.
pub type SessionAnySessionStore = SessionStore<SessionAnyPool>;

/// [SessionAnyPool] is effectively a `dyn DatabasePool`. It can be useful if your application
///
/// requires a runtime decision between multiple database backends. For example using `sqlite`
/// in development builds but `postgres` in production builds.
#[derive(Clone)]
pub struct SessionAnyPool {
    pool: Arc<dyn DatabasePool + Send + Sync>,
}

impl SessionAnyPool {
    pub fn new<Pool>(pool: Pool) -> Self
    where
        Pool: 'static + DatabasePool + Send + Sync,
    {
        Self {
            pool: Arc::new(pool),
        }
    }
}

impl Debug for SessionAnyPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionAnyPool").finish()
    }
}

#[async_trait]
impl DatabasePool for SessionAnyPool {
    async fn initiate(&self, table_name: &str) -> Result<(), DatabaseError> {
        self.pool.initiate(table_name).await
    }

    async fn count(&self, table_name: &str) -> Result<i64, DatabaseError> {
        self.pool.count(table_name).await
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), DatabaseError> {
        self.pool.store(id, session, expires, table_name).await
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, DatabaseError> {
        self.pool.load(id, table_name).await
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), DatabaseError> {
        self.pool.delete_one_by_id(id, table_name).await
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, DatabaseError> {
        self.pool.exists(id, table_name).await
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        self.pool.delete_by_expiry(table_name).await
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), DatabaseError> {
        self.pool.delete_all(table_name).await
    }

    async fn get_ids(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        self.pool.get_ids(table_name).await
    }

    fn auto_handles_expiry(&self) -> bool {
        self.pool.auto_handles_expiry()
    }
}
