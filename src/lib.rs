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

pub use config::{AxumSessionConfig, SameSite};
pub use databases::AxumDatabasePool;
pub use errors::SessionError;
pub use layer::AxumSessionLayer;
pub(crate) use service::AxumSessionService;
pub use session::AxumSession;
pub use session_data::AxumSessionData;
pub use session_id::AxumSessionID;
pub use session_store::AxumSessionStore;
pub use session_timers::AxumSessionTimers;

#[macro_export]
/// Used to Setup the Layer for Axum_Sessions.
/// For backwards compatibility.
/// TODO: Remove this later on.
macro_rules! axum_session_runner {
    ($session_store:expr) => {
        $crate::AxumSessionLayer::new($session_store.clone())
    };
}
