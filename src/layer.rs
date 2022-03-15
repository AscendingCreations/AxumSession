use crate::{AxumSessionService, AxumSessionStore};
use axum_core::{
    body,
    response::{IntoResponse, Response},
    BoxError,
};
use http::Request;
use http::{self, StatusCode};
use http_body::Body as HttpBody;
use pin_project_lite::pin_project;
use std::{
    any::type_name,
    convert::Infallible,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{util::BoxCloneService, ServiceBuilder};
use tower_http::ServiceBuilderExt;
use tower_layer::Layer;
use tower_service::Service;

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
