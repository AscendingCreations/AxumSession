#![doc = include_str!("../README.md")]
#![allow(dead_code)]

mod config;
pub mod databases;
mod errors;
mod layer;
mod service;
mod session;
mod session_data;
mod session_id;
mod session_store;
mod session_timers;

pub use config::{AxumSessionConfig, AxumSessionMode, Key, SameSite};
pub use databases::*;
pub use errors::SessionError;
pub use layer::AxumSessionLayer;
pub use session::AxumSession;
pub use session_store::AxumSessionStore;

pub(crate) use service::{AxumSessionService, CookiesExt};
pub(crate) use session_data::AxumSessionData;
pub(crate) use session_id::AxumSessionID;
pub(crate) use session_timers::AxumSessionTimers;
