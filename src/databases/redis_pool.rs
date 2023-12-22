use async_trait::async_trait;
use redis_pool::SingleRedisPool;

use crate::{DatabasePool, Session, SessionError, SessionStore};

///Redis's Session Helper type for the DatabasePool.
pub type SessionRedisSession = Session<SessionRedisPool>;
///Redis's Session Store Helper type for the DatabasePool.
pub type SessionRedisSessionStore = SessionStore<SessionRedisPool>;

///Redis's Pool type for the DatabasePool. Needs a redis Client.
#[derive(Clone)]
pub struct SessionRedisPool {
    pool: SingleRedisPool,
}

impl From<SingleRedisPool> for SessionRedisPool {
    fn from(pool: SingleRedisPool) -> Self {
        SessionRedisPool { pool }
    }
}

impl std::fmt::Debug for SessionRedisPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionRedisPool").finish()
    }
}

#[async_trait]
impl DatabasePool for SessionRedisPool {
    async fn initiate(&self, _table_name: &str) -> Result<(), SessionError> {
        // Redis does not actually use Tables so there is no way we can make one.
        Ok(())
    }

    async fn delete_by_expiry(&self, _table_name: &str) -> Result<Vec<String>, SessionError> {
        // Redis does this for use using the Expiry Options.
        Ok(Vec::new())
    }

    async fn count(&self, table_name: &str) -> Result<i64, SessionError> {
        let mut con = self.pool.aquire().await?;

        let count: i64 = if table_name.is_empty() {
            redis::cmd("DBSIZE").query_async(&mut con).await?
        } else {
            // Assuming we have a table name, we need to count all the keys that match the table name.
            // We can't use DBSIZE because that would count all the keys in the database.
            let keys =
                super::redis_tools::scan_keys(&mut con, &format!("{}:*", table_name)).await?;
            keys.len() as i64
        };

        Ok(count)
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), SessionError> {
        let id = if table_name.is_empty() {
            id.to_string()
        } else {
            format!("{}:{}", table_name, id)
        };
        let mut con = self.pool.aquire().await?;
        redis::pipe()
            .atomic() //makes this a transation.
            .set(&id, session)
            .ignore()
            .expire_at(&id, expires)
            .ignore()
            .query_async(&mut con)
            .await?;
        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, SessionError> {
        let mut con = self.pool.aquire().await?;
        let id = if table_name.is_empty() {
            id.to_string()
        } else {
            format!("{}:{}", table_name, id)
        };
        let result: String = redis::cmd("GET").arg(id).query_async(&mut con).await?;
        Ok(Some(result))
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), SessionError> {
        let mut con = self.pool.aquire().await?;
        let id = if table_name.is_empty() {
            id.to_string()
        } else {
            format!("{}:{}", table_name, id)
        };
        redis::cmd("DEL").arg(id).query_async(&mut con).await?;
        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, SessionError> {
        let mut con = self.pool.aquire().await?;
        let id = if table_name.is_empty() {
            id.to_string()
        } else {
            format!("{}:{}", table_name, id)
        };
        let exists: bool = redis::cmd("EXISTS").arg(id).query_async(&mut con).await?;

        Ok(exists)
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), SessionError> {
        let mut con = self.pool.aquire().await?;
        if table_name.is_empty() {
            redis::cmd("FLUSHDB").query_async(&mut con).await?;
        } else {
            // Assuming we have a table name, we need to delete all the keys that match the table name.
            // We can't use FLUSHDB because that would delete all the keys in the database.
            let keys =
                super::redis_tools::scan_keys(&mut con, &format!("{}:*", table_name)).await?;

            for key in keys {
                redis::cmd("DEL").arg(key).query_async(&mut con).await?;
            }
        }

        Ok(())
    }

    async fn get_ids(&self, table_name: &str) -> Result<Vec<String>, SessionError> {
        let mut con = self.pool.aquire().await?;
        let table_name = if table_name.is_empty() {
            "*".to_string()
        } else {
            format!("{}:0", table_name)
        };

        let result: Vec<String> =
            super::redis_tools::scan_keys(&mut con, &format!("{}:*", table_name)).await?;
        Ok(result)
    }

    fn auto_handles_expiry(&self) -> bool {
        true
    }
}
