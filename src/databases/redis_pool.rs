use crate::{DatabasePool, Session, SessionError, SessionStore};
use async_trait::async_trait;
use redis::Client;

///Redis's Session Helper type for the DatabasePool.
pub type SessionRedisSession = Session<SessionRedisPool>;
///Redis's Session Store Helper type for the DatabasePool.
pub type SessionRedisSessionStore = SessionStore<SessionRedisPool>;

///Redis's Pool type for the DatabasePool. Needs a redis Client.
#[derive(Debug, Clone)]
pub struct SessionRedisPool {
    client: Client,
}

impl From<Client> for SessionRedisPool {
    fn from(client: Client) -> Self {
        SessionRedisPool { client }
    }
}

#[async_trait]
impl DatabasePool for SessionRedisPool {
    async fn initiate(&self, _table_name: &str) -> Result<(), SessionError> {
        // Redis does not actually use Tables so there is no way we can make one.
        Ok(())
    }

    async fn delete_by_expiry(&self, _table_name: &str) -> Result<(), SessionError> {
        // Redis does this for use using the Expiry Options.
        Ok(())
    }

    async fn count(&self, _table_name: &str) -> Result<i64, SessionError> {
        let mut con = self.client.get_async_connection().await?;
        let count: i64 = redis::cmd("DBSIZE").query_async(&mut con).await?;

        Ok(count)
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        _table_name: &str,
    ) -> Result<(), SessionError> {
        let mut con = self.client.get_async_connection().await?;
        redis::pipe()
            .set(id, session)
            .ignore()
            .expire_at(id, expires as usize)
            .ignore()
            .query_async(&mut con)
            .await?;
        Ok(())
    }

    async fn load(&self, id: &str, _table_name: &str) -> Result<Option<String>, SessionError> {
        let mut con = self.client.get_async_connection().await?;
        let result: String = redis::cmd("GET").arg(id).query_async(&mut con).await?;
        Ok(Some(result))
    }

    async fn delete_one_by_id(&self, id: &str, _table_name: &str) -> Result<(), SessionError> {
        let mut con = self.client.get_async_connection().await?;
        redis::cmd("DEL").arg(id).query_async(&mut con).await?;
        Ok(())
    }

    async fn exists(&self, id: &str, _table_name: &str) -> Result<bool, SessionError> {
        let mut con = self.client.get_async_connection().await?;
        let exists: bool = redis::cmd("EXISTS").arg(id).query_async(&mut con).await?;

        Ok(exists)
    }

    async fn delete_all(&self, _table_name: &str) -> Result<(), SessionError> {
        let mut con = self.client.get_async_connection().await?;
        redis::cmd("FLUSHDB").query_async(&mut con).await?;
        Ok(())
    }
}
