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

pub const MIGRATE_QUERY: &str = r#"
        CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
            id TEXT PRIMARY KEY NOT NULL,
            expires INTEGER NULL,
            session TEXT NOT NULL
        )
    "#;

pub const CLEANUP_QUERY: &str = r#"DELETE FROM %%TABLE_NAME%% WHERE expires < ?"#;

pub const COUNT_QUERY: &str = r#"SELECT COUNT(*) FROM %%TABLE_NAME%%"#;

pub const LOAD_QUERY: &str = r#"
        SELECT session FROM %%TABLE_NAME%%
        WHERE id = ? AND (expires IS NULL OR expires > ?)
    "#;

pub const STORE_QUERY: &str = r#"
        INSERT INTO %%TABLE_NAME%%
            (id, session, expires) VALUES (?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            expires = excluded.expires,
            session = excluded.session
    "#;
pub const DESTROY_QUERY: &str = r#"DELETE FROM %%TABLE_NAME%% WHERE id = ?"#;

pub const CLEAR_QUERY: &str = r#"DELETE FROM %%TABLE_NAME%%"#;
