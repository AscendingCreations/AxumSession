use crate::{AxumSessionService, AxumSessionStore};
use tower_layer::Layer;

/// Sessions Layer used with Axum to activate the Service.
///
/// # Examples
/// ```
/// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
/// use uuid::Uuid;
///
/// let config = AxumSessionConfig::default();
/// let token = Uuid::new_v4();
/// let session_store = AxumSessionStore::new(None, &config);
/// let layer = AxumSessionLayer::new(session_store);
/// ```
///
#[derive(Clone)]
pub struct AxumSessionLayer {
    session_store: AxumSessionStore,
}

impl AxumSessionLayer {
    /// Constructs a AxumSessionLayer used with Axum to activate the Service.
    ///
    /// # Examples
    /// ```
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
    /// use uuid::Uuid;
    ///
    /// let config = AxumSessionConfig::default();
    /// let token = Uuid::new_v4();
    /// let session_store = AxumSessionStore::new(None, &config);
    /// let layer = AxumSessionLayer::new(session_store);
    /// ```
    ///
    pub fn new(session_store: AxumSessionStore) -> Self {
        AxumSessionLayer { session_store }
    }
}

impl<S> Layer<S> for AxumSessionLayer {
    type Service = AxumSessionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AxumSessionService {
            session_store: self.session_store.clone(),
            inner,
        }
    }
}
