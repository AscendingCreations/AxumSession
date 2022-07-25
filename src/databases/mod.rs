#[cfg(any(feature = "mysql-rustls", feature = "mysql-native"))]
mod mysql;
#[cfg(any(feature = "mysql-rustls", feature = "mysql-native"))]
pub use mysql::{AxumMySqlPool, AxumMySqlSession, AxumMySqlSessionStore};

#[cfg(any(feature = "postgres-rustls", feature = "postgres-native"))]
mod postgres;
#[cfg(any(feature = "postgres-rustls", feature = "postgres-native"))]
pub use postgres::{AxumPgPool, AxumPgSession, AxumPgSessionStore};

#[cfg(any(feature = "sqlite-rustls", feature = "sqlite-native"))]
mod sqlite;
#[cfg(any(feature = "sqlite-rustls", feature = "sqlite-native"))]
pub use sqlite::{AxumSqlitePool, AxumSqliteSession, AxumSqliteSessionStore};

#[cfg(feature = "redis")]
mod redis_pool;
#[cfg(feature = "redis")]
pub use redis_pool::{AxumRedisPool, AxumRedisSession, AxumRedisSessionStore};

mod database;
mod null;

pub use database::AxumDatabasePool;
pub use null::{AxumNullPool, AxumNullSession, AxumNullSessionStore};
