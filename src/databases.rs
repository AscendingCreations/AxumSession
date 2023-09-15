#[cfg(any(feature = "mysql-rustls", feature = "mysql-native"))]
mod mysql;
#[cfg(any(feature = "mysql-rustls", feature = "mysql-native"))]
pub use self::mysql::*;

#[cfg(any(feature = "postgres-rustls", feature = "postgres-native"))]
mod postgres;
#[cfg(any(feature = "postgres-rustls", feature = "postgres-native"))]
pub use self::postgres::*;

#[cfg(any(feature = "sqlite-rustls", feature = "sqlite-native"))]
mod sqlite;
#[cfg(any(feature = "sqlite-rustls", feature = "sqlite-native"))]
pub use self::sqlite::*;

#[cfg(feature = "redis-db")]
mod redis_pool;
#[cfg(feature = "redis-db")]
pub use self::redis_pool::*;

#[cfg(feature = "redis-clusterdb")]
mod redis_cluster_pool;
#[cfg(feature = "redis-clusterdb")]
pub use self::redis_cluster_pool::*;

#[cfg(feature = "mongo")]
mod mongodb;
#[cfg(feature = "mongo")]
pub use self::mongodb::*;

#[cfg(feature = "surrealdb_tag")]
mod surreal;
#[cfg(feature = "surrealdb_tag")]
pub use self::surreal::*;

mod database;
mod null;

pub use database::DatabasePool;
pub use null::*;
