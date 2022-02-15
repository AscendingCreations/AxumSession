#![doc = include_str!("../README.md")]
#![allow(dead_code)]

use sqlx::{pool::Pool, Sqlite, SqlitePool};

///Mysql's Pool type for AxumDatabasePool
#[derive(Debug, Clone)]
pub struct AxumDatabasePool(SqlitePool);

impl AxumDatabasePool {
    /// Grabs the Pool for direct usage
    pub fn inner(&self) -> &SqlitePool {
        &self.0
    }
}

impl From<Pool<Sqlite>> for AxumDatabasePool {
    fn from(conn: SqlitePool) -> Self {
        AxumDatabasePool(conn)
    }
}

pub fn migrate_query() -> String {
    String::from(
        r#"
            CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                id TEXT PRIMARY KEY NOT NULL,
                expires INTEGER NULL,
                session TEXT NOT NULL
            )
        "#,
    )
}

pub fn cleanup_query() -> String {
    String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE expires < ?"#)
}

pub fn count_query() -> String {
    String::from(r#"SELECT COUNT(*) FROM %%TABLE_NAME%%"#)
}

pub fn load_query() -> String {
    String::from(
        r#"SELECT session FROM %%TABLE_NAME%% WHERE id = ? AND (expires IS NULL OR expires > ?)"#,
    )
}
pub fn store_query() -> String {
    String::from(
        r#"
            INSERT INTO %%TABLE_NAME%%
                (id, session, expires) VALUES (?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                expires = excluded.expires,
                session = excluded.session
        "#,
    )
}
pub fn destroy_query() -> String {
    String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE id = ?"#)
}
pub fn clear_query() -> String {
    String::from(r#"DELETE FROM %%TABLE_NAME%%"#)
}
