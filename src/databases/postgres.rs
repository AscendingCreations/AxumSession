use crate::{AxumDatabasePool, AxumSession, AxumSessionStore, SessionError};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::{pool::Pool, PgPool, Postgres};

pub type AxumPgSession = AxumSession<AxumPgPool>;
pub type AxumPgSessionStore = AxumSessionStore<AxumPgPool>;

///Mysql's Pool type for AxumDatabasePool
#[derive(Debug, Clone)]
pub struct AxumPgPool {
    pool: Pool<Postgres>,
}

impl From<Pool<Postgres>> for AxumPgPool {
    fn from(conn: PgPool) -> Self {
        AxumPgPool { pool: conn }
    }
}

#[async_trait]
impl AxumDatabasePool for AxumPgPool {
    async fn initiate(&self, table_name: &str) -> Result<(), SessionError> {
        sqlx::query(
            &r#"
            CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                "id" VARCHAR(128) NOT NULL PRIMARY KEY,
                "expires" INTEGER NULL,
                "session" TEXT NOT NULL
            )
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<(), SessionError> {
        sqlx::query(
            &r#"DELETE FROM %%TABLE_NAME%% WHERE expires < $1"#
                .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(Utc::now().timestamp())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn count(&self, table_name: &str) -> Result<i64, SessionError> {
        let (count,) = sqlx::query_as(
            &r#"SELECT COUNT(*) FROM %%TABLE_NAME%%"#.replace("%%TABLE_NAME%%", table_name),
        )
        .fetch_one(&self.pool)
        .await?;

        return Ok(count);
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), SessionError> {
        sqlx::query(
            &r#"
        INSERT INTO %%TABLE_NAME%%
            (id, session, expires) SELECT $1, $2, $3
        ON CONFLICT(id) DO UPDATE SET
            expires = EXCLUDED.expires,
            session = EXCLUDED.session
    "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(&id)
        .bind(&session)
        .bind(&expires)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, SessionError> {
        let result: Option<(String,)> = sqlx::query_as(
            &r#"
            SELECT session FROM %%TABLE_NAME%%
            WHERE id = $1 AND (expires IS NULL OR expires > $2)
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(&id)
        .bind(Utc::now().timestamp())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(session,)| session))
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), SessionError> {
        sqlx::query(
            &r#"DELETE FROM %%TABLE_NAME%% WHERE id = $1"#.replace("%%TABLE_NAME%%", table_name),
        )
        .bind(&id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), SessionError> {
        sqlx::query(&r#"TRUNCATE %%TABLE_NAME%%"#.replace("%%TABLE_NAME%%", table_name))
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
