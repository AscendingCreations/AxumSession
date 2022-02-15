#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(any(feature = "postgres", feature = "mysql", feature = "sqlite",)))]
compile_error!("one of the features ['postgres', 'mysql', 'sqlite'] must be enabled");

#[cfg(any(
    all(feature = "postgres", feature = "mysql"),
    all(feature = "postgres", feature = "sqlite"),
    all(feature = "mysql", feature = "sqlite"),
))]
compile_error!("only one of ['postgres', 'mysql', 'sqlite'] can be enabled");

mod config;
mod errors;
mod future;
mod layer;
mod manager;
mod session;
mod session_data;
mod session_id;
mod session_store;
mod session_timers;

#[cfg(feature = "mysql")]
pub use axum_mysql_sessions_pool::AxumDatabasePool;

#[cfg(feature = "postgres")]
pub use axum_postgres_sessions_pool::AxumDatabasePool;

#[cfg(feature = "sqlite")]
pub use axum_sqlite_sessions_pool::AxumDatabasePool;

pub use config::AxumSessionConfig;
pub use errors::SessionError;
pub use future::AxumDatabaseResponseFuture;
pub use layer::AxumSessionLayer;
pub use manager::AxumDatabaseSessionManager;
pub use session::AxumSession;
pub use session_data::AxumSessionData;
pub use session_id::AxumSessionID;
pub use session_store::AxumSessionStore;
pub use session_timers::AxumSessionTimers;
