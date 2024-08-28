use async_trait::async_trait;
use axum_session::{DatabaseError, DatabasePool, Session, SessionStore};
use chrono::Utc;
use sqlx::{pool::Pool, Sqlite};

///Sqlite's Session Helper type for the DatabasePool.
pub type SessionSqliteSession = Session<SessionSqlitePool>;
///Sqlite's Session Store Helper type for the DatabasePool.
pub type SessionSqliteSessionStore = SessionStore<SessionSqlitePool>;

///Sqlite's Pool type for the DatabasePool
#[derive(Debug, Clone)]
pub struct SessionSqlitePool {
    pool: Pool<Sqlite>,
}

impl From<Pool<Sqlite>> for SessionSqlitePool {
    fn from(conn: Pool<Sqlite>) -> Self {
        SessionSqlitePool { pool: conn }
    }
}

#[async_trait]
impl DatabasePool for SessionSqlitePool {
    async fn initiate(&self, table_name: &str) -> Result<(), DatabaseError> {
        sqlx::query(
            &r#"
            CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                "id" VARCHAR(128) NOT NULL PRIMARY KEY,
                "expires" BIGINT NULL,
                "session" TEXT NOT NULL
            )
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .execute(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericCreateError(err.to_string()))?;

        Ok(())
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        let result: Vec<(String,)> = sqlx::query_as(
            &r#"
            SELECT id FROM %%TABLE_NAME%%
            WHERE (expires IS NULL OR expires < $1)
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(Utc::now().timestamp())
        .fetch_all(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        let result: Vec<String> = result.into_iter().map(|(s,)| s).collect();

        sqlx::query(
            &r#"DELETE FROM %%TABLE_NAME%% WHERE expires < $1"#
                .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(Utc::now().timestamp())
        .execute(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
        Ok(result)
    }

    async fn count(&self, table_name: &str) -> Result<i64, DatabaseError> {
        let (count,) = sqlx::query_as(
            &r#"SELECT COUNT(*) FROM %%TABLE_NAME%%"#.replace("%%TABLE_NAME%%", table_name),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        return Ok(count);
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), DatabaseError> {
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
        .bind(id)
        .bind(session)
        .bind(expires)
        .execute(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericInsertError(err.to_string()))?;
        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, DatabaseError> {
        let result: Option<(String,)> = sqlx::query_as(
            &r#"
            SELECT session FROM %%TABLE_NAME%%
            WHERE id = $1 AND (expires IS NULL OR expires > $2)
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(id)
        .bind(Utc::now().timestamp())
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        Ok(result.map(|(session,)| session))
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), DatabaseError> {
        sqlx::query(
            &r#"DELETE FROM %%TABLE_NAME%% WHERE id = $1"#.replace("%%TABLE_NAME%%", table_name),
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, DatabaseError> {
        let result: Option<(i64,)> = sqlx::query_as(
            &r#"
            SELECT COUNT(*) FROM %%TABLE_NAME%%
            WHERE id = $1 AND (expires IS NULL OR expires > $2)
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(id)
        .bind(Utc::now().timestamp())
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        Ok(result.map(|(o,)| o).unwrap_or(0) > 0)
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), DatabaseError> {
        sqlx::query(&r#"DELETE FROM %%TABLE_NAME%%"#.replace("%%TABLE_NAME%%", table_name))
            .execute(&self.pool)
            .await
            .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
        Ok(())
    }

    async fn get_ids(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        let result: Vec<(String,)> = sqlx::query_as(
            &r#"
            SELECT id FROM %%TABLE_NAME%%
            WHERE (expires IS NULL OR expires > $1)
        "#
            .replace("%%TABLE_NAME%%", table_name),
        )
        .bind(Utc::now().timestamp())
        .fetch_all(&self.pool)
        .await
        .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        let result: Vec<String> = result.into_iter().map(|(s,)| s).collect();

        Ok(result)
    }

    fn auto_handles_expiry(&self) -> bool {
        false
    }
}
