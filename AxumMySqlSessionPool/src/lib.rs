#![doc = include_str!("../README.md")]
#![allow(dead_code)]

use sqlx::{pool::Pool, MySql, MySqlPool};

///Mysql's Pool type for AxumDatabasePool
#[derive(Debug, Clone)]
pub struct AxumDatabasePool(MySqlPool);

impl AxumDatabasePool {
    /// Grabs the Pool for direct usage
    pub fn inner(&self) -> &MySqlPool {
        &self.0
    }
}

impl From<Pool<MySql>> for AxumDatabasePool {
    fn from(conn: MySqlPool) -> Self {
        AxumDatabasePool(conn)
    }
}

pub fn migrate_query() -> String {
    String::from(
        r#"
            CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                `id` VARCHAR(128) NOT NULL,
                `expires` INTEGER NULL,
                `session` TEXT NOT NULL,
                PRIMARY KEY (`id`),
                KEY `expires` (`expires`)
            )
            ENGINE=InnoDB
            DEFAULT CHARSET=utf8mb4
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
            (id, session, expires) VALUES(?, ?, ?)
        ON DUPLICATE KEY UPDATE
            expires = VALUES(expires),
            session = VALUES(session)
        "#,
    )
}
pub fn destroy_query() -> String {
    String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE id = ?"#)
}
pub fn clear_query() -> String {
    String::from(r#"TRUNCATE %%TABLE_NAME%%"#)
}
