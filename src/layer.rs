#[cfg(feature = "mysql")]
use axum_mysql_sessions_pool::*;

#[cfg(feature = "postgres")]
use axum_postgres_sessions_pool::*;

#[cfg(feature = "sqlite")]
use axum_sqlite_sessions_pool::*;

use crate::{AxumDatabaseSessionManager, AxumSessionConfig, AxumSessionStore};
use tower_layer::Layer;

/// Session layer struct used for starting the Manager when a user comes on board.
#[derive(Clone, Debug)]
pub struct AxumSessionLayer {
    store: AxumSessionStore,
}

impl AxumSessionLayer {
    /// Creates the Sqlx Session Layer.
    pub fn new(config: AxumSessionConfig, poll: AxumDatabasePool) -> Self {
        let store = AxumSessionStore::new(poll, config);
        Self { store }
    }
}

impl<S> Layer<S> for AxumSessionLayer {
    type Service = AxumDatabaseSessionManager<S>;

    ///This is called as soon as the session layer is placed within .layer of axum.
    fn layer(&self, service: S) -> Self::Service {
        AxumDatabaseSessionManager::new(service, self.store.clone())
    }
}
