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

pub const MIGRATE_QUERY: &str = r#"
        CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
            "id" VARCHAR(128) NOT NULL PRIMARY KEY,
            "expires" INTEGER NULL,
            "session" TEXT NOT NULL
        )
    "#;

pub const CLEANUP_QUERY: &str = r#"DELETE FROM %%TABLE_NAME%% WHERE expires < $1"#;

pub const COUNT_QUERY: &str = r#"SELECT COUNT(*) FROM %%TABLE_NAME%%"#;

pub const LOAD_QUERY: &str = r#"
        SELECT session FROM %%TABLE_NAME%%
        WHERE id = $1 AND (expires IS NULL OR expires > $2)
    "#;

pub const STORE_QUERY: &str = r#"
        INSERT INTO %%TABLE_NAME%%
            (id, session, expires) SELECT $1, $2, $3
        ON CONFLICT(id) DO UPDATE SET
            expires = EXCLUDED.expires,
            session = EXCLUDED.session
    "#;
pub const DESTROY_QUERY: &str = r#"DELETE FROM %%TABLE_NAME%% WHERE id = $1"#;

pub const CLEAR_QUERY: &str = r#"TRUNCATE %%TABLE_NAME%%"#;
