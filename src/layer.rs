use crate::{AxumSessionService, AxumSessionStore};
use tower_layer::Layer;

#[derive(Clone)]
pub struct AxumSessionLayer {
    session_store: AxumSessionStore,
}

impl AxumSessionLayer {
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
