#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(any(
    feature = "postgres",
    feature = "mysql",
    feature = "sqlite",
    features = "DatabaseTrait"
)))]
compile_error!(
    "one of the features ['postgres', 'mysql', 'sqlite','DatabaseTrait'] must be enabled"
);

#[cfg(any(
    all(feature = "postgres", feature = "mysql"),
    all(feature = "postgres", feature = "sqlite"),
    all(feature = "postgres", feature = "DatabaseTrait"),
    all(feature = "mysql", feature = "DatabaseTrait"),
    all(feature = "mysql", feature = "sqlite"),
))]
compile_error!("only one of ['postgres', 'mysql', 'sqlite', 'DatabaseTrait'] can be enabled");

mod config;
mod databases;
mod errors;
mod layer;
mod service;
mod session;
mod session_data;
mod session_id;
mod session_store;
mod session_timers;

pub use config::{AxumSessionConfig, AxumSessionMode, Key, SameSite};
#[cfg(feature = "DatabaseTrait")]
pub use databases::AxumDatabasePool;
#[cfg(feature = "postgres")]
pub use databases::AxumPgPool;
#[cfg(feature = "mysql")]
pub use databases::AxumMySqlPool;
#[cfg(feature = "sqlite")]
pub use databases::AxumSqlitePool;
pub use errors::SessionError;
pub use layer::AxumSessionLayer;
pub use session::AxumSession;
pub use session_store::AxumSessionStore;

pub(crate) use service::{AxumSessionService, CookiesExt};
pub(crate) use session_data::AxumSessionData;
pub(crate) use session_id::AxumSessionID;
pub(crate) use session_timers::AxumSessionTimers;
