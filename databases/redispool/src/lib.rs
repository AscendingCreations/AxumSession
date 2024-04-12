#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![warn(clippy::all, nonstandard_style, future_incompatible)]
#![forbid(unsafe_code)]

mod redis_pool;
pub use self::redis_pool::*;

#[cfg(feature = "redis-clusterdb")]
mod redis_cluster_pool;
#[cfg(feature = "redis-clusterdb")]
pub use self::redis_cluster_pool::*;

pub(crate) mod redis_tools;
