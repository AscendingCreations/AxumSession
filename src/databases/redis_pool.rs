use crate::{AxumDatabasePool, AxumSession, AxumSessionStore, SessionError};
use async_trait::async_trait;
use redis::Client;

pub type AxumRedisSession = AxumSession<AxumRedisPool>;
pub type AxumRedisSessionStore = AxumSessionStore<AxumRedisPool>;

///Mysql's Pool type for AxumDatabasePool
#[derive(Debug, Clone)]
pub struct AxumRedisPool {
    client: Client,
}

impl From<Client> for AxumRedisPool {
    fn from(client: Client) -> Self {
        AxumRedisPool { client }
    }
}

#[async_trait]
impl AxumDatabasePool for AxumRedisPool {
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
        let mut result: Vec<String> = redis::pipe().get(id).query_async(&mut con).await?;
        Ok(result.pop())
    }

    async fn delete_one_by_id(&self, id: &str, _table_name: &str) -> Result<(), SessionError> {
        let mut con = self.client.get_async_connection().await?;
        redis::pipe().del(id).query_async(&mut con).await?;
        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, SessionError> {
        let mut con = self.client.get_async_connection().await?;
        let exists: bool = redis::pipe().exists(id).query_async(&mut con).await?;

        Ok(exists)
    }

    async fn delete_all(&self, _table_name: &str) -> Result<(), SessionError> {
        let mut con = self.client.get_async_connection().await?;
        redis::pipe().cmd("FLUSHDB").query_async(&mut con).await?;
        Ok(())
    }
}
