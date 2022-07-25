#[cfg(any(feature = "mysql-rustls", feature = "mysql-native"))]
mod mysql;
#[cfg(any(feature = "mysql-rustls", feature = "mysql-native"))]
pub use mysql::*;

#[cfg(any(feature = "postgres-rustls", feature = "postgres-native"))]
mod postgres;
#[cfg(any(feature = "postgres-rustls", feature = "postgres-native"))]
pub use postgres::*;

#[cfg(any(feature = "sqlite-rustls", feature = "sqlite-native"))]
mod sqlite;
#[cfg(any(feature = "sqlite-rustls", feature = "sqlite-native"))]
pub use sqlite::*;

#[cfg(feature = "redis-db")]
mod redis_pool;
#[cfg(feature = "redis-db")]
pub use redis_pool::*;

mod database;
mod null;

pub use database::AxumDatabasePool;
pub use null::*;
