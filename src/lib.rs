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
pub use databases::database::AxumDatabasePool;
pub use errors::SessionError;
pub use layer::AxumSessionLayer;
pub use session::AxumSession;
pub use session_store::AxumSessionStore;

pub(crate) use service::{AxumSessionService, CookiesExt};
pub(crate) use session_data::AxumSessionData;
pub(crate) use session_id::AxumSessionID;
pub(crate) use session_timers::AxumSessionTimers;