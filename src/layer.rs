use std::fmt;

use crate::{DatabasePool, SessionService, SessionStore};
use tower_layer::Layer;

/// Sessions Layer used with Axum to activate the Service.
///
/// # Examples
/// ```
/// use axum_session::{SessionNullPool, SessionConfig, SessionStore, SessionLayer};
/// use uuid::Uuid;
///
/// let config = SessionConfig::default();
/// let session_store = SessionStore::<SessionNullPool>::new(None, config).await.unwrap();
/// let layer = SessionLayer::new(session_store);
/// ```
///
#[derive(Clone)]
pub struct SessionLayer<T>
where
    T: DatabasePool + Clone + fmt::Debug + std::marker::Sync + std::marker::Send + 'static,
{
    session_store: SessionStore<T>,
}

impl<T> SessionLayer<T>
where
    T: DatabasePool + Clone + fmt::Debug + std::marker::Sync + std::marker::Send + 'static,
{
    /// Constructs a SessionLayer used with Axum to activate the Service.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore, SessionLayer};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config).await.unwrap();
    /// let layer = SessionLayer::new(session_store);
    /// ```
    ///
    #[inline]
    pub fn new(session_store: SessionStore<T>) -> Self {
        SessionLayer { session_store }
    }
}

impl<S, T> Layer<S> for SessionLayer<T>
where
    T: DatabasePool + Clone + fmt::Debug + std::marker::Sync + std::marker::Send + 'static,
{
    type Service = SessionService<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionService {
            session_store: self.session_store.clone(),
            inner,
        }
    }
}
