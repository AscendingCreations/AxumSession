use std::fmt;

use crate::{AxumDatabasePool, AxumSessionService, AxumSessionStore};
use tower_layer::Layer;

/// Sessions Layer used with Axum to activate the Service.
///
/// # Examples
/// ```
/// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
/// use uuid::Uuid;
///
/// let config = AxumSessionConfig::default();
/// let session_store = AxumSessionStore::new(None, &config);
/// let layer = AxumSessionLayer::new(session_store);
/// ```
///
#[derive(Clone)]
pub struct AxumSessionLayer<T>
where
    T: AxumDatabasePool + Clone + fmt::Debug + std::marker::Sync + std::marker::Send + 'static,
{
    session_store: AxumSessionStore<T>,
}

impl<T> AxumSessionLayer<T>
where
    T: AxumDatabasePool + Clone + fmt::Debug + std::marker::Sync + std::marker::Send + 'static,
{
    /// Constructs a AxumSessionLayer used with Axum to activate the Service.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
    /// use uuid::Uuid;
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, &config);
    /// let layer = AxumSessionLayer::new(session_store);
    /// ```
    ///
    pub fn new(session_store: AxumSessionStore<T>) -> Self {
        AxumSessionLayer { session_store }
    }
}

impl<S, T> Layer<S> for AxumSessionLayer<T>
where
    T: AxumDatabasePool + Clone + fmt::Debug + std::marker::Sync + std::marker::Send + 'static,
{
    type Service = AxumSessionService<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        AxumSessionService {
            session_store: self.session_store.clone(),
            inner,
        }
    }
}
