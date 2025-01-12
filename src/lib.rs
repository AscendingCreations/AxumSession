#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![warn(clippy::all, nonstandard_style, future_incompatible)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod config;
pub mod databases;
mod errors;
pub(crate) mod headers;
mod layer;
mod sec;
mod service;
mod session;
mod session_data;
mod session_store;

pub use config::{Key, SameSite, SessionConfig, SessionMode};
pub use databases::*;
pub use errors::SessionError;
pub use layer::SessionLayer;
pub use sec::*;
pub use session::{ReadOnlySession, Session};
pub use session_store::SessionStore;

pub(crate) use service::SessionService;
pub(crate) use session_data::{SessionData, SessionTimers};
