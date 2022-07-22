#[cfg(feature = "mysql")]
#[doc(hidden)]
mod mysql;
#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub use mysql::*;

#[cfg(feature = "postgres")]
#[doc(hidden)]
mod postgres;
#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub use postgres::*;

#[cfg(feature = "sqlite")]
#[doc(hidden)]
mod sqlite;
#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub use sqlite::*;

pub mod databases;
pub use databases::AxumDatabasePool;
