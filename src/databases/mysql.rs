use sqlx::{pool::Pool, MySql, MySqlPool};

/// Mysql's Pool type for AxumDatabasePool
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

pub const MIGRATE_QUERY: &str = r#"
        CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
            `id` VARCHAR(128) NOT NULL,
            `expires` INTEGER NULL,
            `session` TEXT NOT NULL,
            PRIMARY KEY (`id`),
            KEY `expires` (`expires`)
        )
        ENGINE=InnoDB
        DEFAULT CHARSET=utf8mb4
    "#;

pub const CLEANUP_QUERY: &str = r#"DELETE FROM %%TABLE_NAME%% WHERE expires < ?"#;

pub const COUNT_QUERY: &str = r#"SELECT COUNT(*) FROM %%TABLE_NAME%%"#;

pub const LOAD_QUERY: &str = r#"
        SELECT session FROM %%TABLE_NAME%%
        WHERE id = ? AND (expires IS NULL OR expires > ?)
    "#;

pub const STORE_QUERY: &str = r#"
        INSERT INTO %%TABLE_NAME%%
            (id, session, expires) VALUES(?, ?, ?)
        ON DUPLICATE KEY UPDATE
            expires = VALUES(expires),
            session = VALUES(session)
    "#;
pub const DESTROY_QUERY: &str = r#"DELETE FROM %%TABLE_NAME%% WHERE id = ?"#;

pub const CLEAR_QUERY: &str = r#"TRUNCATE %%TABLE_NAME%%"#;
