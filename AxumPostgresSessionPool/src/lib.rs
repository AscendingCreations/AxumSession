#![doc = include_str!("../README.md")]
#![allow(dead_code)]

use sqlx::{pool::Pool, PgPool, Postgres};

///Mysql's Pool type for AxumDatabasePool
#[derive(Debug, Clone)]
pub struct AxumDatabasePool(PgPool);

impl AxumDatabasePool {
    /// Grabs the Pool for direct usage
    pub fn inner(&self) -> &PgPool {
        &self.0
    }
}

impl From<Pool<Postgres>> for AxumDatabasePool {
    fn from(conn: PgPool) -> Self {
        AxumDatabasePool(conn)
    }
}

pub fn migrate_query() -> String {
    String::from(
        r#"
            CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                "id" VARCHAR(128) NOT NULL PRIMARY KEY,
                "expires" INTEGER NULL,
                "session" TEXT NOT NULL
            )
        "#,
    )
}

pub fn cleanup_query() -> String {
    String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE expires < $1"#)
}

pub fn count_query() -> String {
    String::from(r#"SELECT COUNT(*) FROM %%TABLE_NAME%%"#)
}

pub fn load_query() -> String {
    String::from(
        r#"SELECT session FROM %%TABLE_NAME%% WHERE id = $1 AND (expires IS NULL OR expires > $2)"#,
    )
}
pub fn store_query() -> String {
    String::from(
        r#"
            INSERT INTO %%TABLE_NAME%%
                (id, session, expires) SELECT $1, $2, $3
            ON CONFLICT(id) DO UPDATE SET
                expires = EXCLUDED.expires,
                session = EXCLUDED.session
        "#,
    )
}
pub fn destroy_query() -> String {
    String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE id = $1"#)
}
pub fn clear_query() -> String {
    String::from(r#"TRUNCATE %%TABLE_NAME%%"#)
}
