#[cfg(any(feature = "mysql-rustls", feature = "mysql-native"))]
#[doc(hidden)]
mod mysql;
#[cfg(any(feature = "mysql-rustls", feature = "mysql-native"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "mysql-rustls", feature = "mysql-native")))
)]
pub use mysql::*;

#[cfg(any(feature = "postgres-rustls", feature = "postgres-native"))]
#[doc(hidden)]
mod postgres;
#[cfg(any(feature = "postgres-rustls", feature = "postgres-native"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "postgres-rustls", feature = "postgres-native")))
)]
pub use postgres::*;

#[cfg(any(feature = "sqlite-rustls", feature = "sqlite-native"))]
#[doc(hidden)]
mod sqlite;
#[cfg(any(feature = "sqlite-rustls", feature = "sqlite-native"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "sqlite-rustls", feature = "sqlite-native")))
)]
pub use sqlite::*;

mod database;
mod null;

pub use database::AxumDatabasePool;
pub use null::*;
