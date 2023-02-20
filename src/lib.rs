#![doc = include_str!("../README.md")]
#![allow(dead_code)]

mod config;
pub mod databases;
mod errors;
mod layer;
mod service;
mod session;
mod session_data;
mod session_store;

pub use config::{Key, SameSite, SessionConfig, SessionMode};
pub use databases::*;
pub use errors::SessionError;
pub use layer::SessionLayer;
pub use session::Session;
pub use session_store::SessionStore;

pub(crate) use service::{CookiesExt, SessionService};
pub(crate) use session_data::{SessionData, SessionID, SessionTimers};

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use axum::{
        body::Body,
        http::{header, Request},
        response::Redirect,
        routing::get,
        Router,
    };
    use log::LevelFilter;
    use serde::{Deserialize, Serialize};
    use sqlx::{
        postgres::{PgConnectOptions, PgPoolOptions},
        ConnectOptions,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn basic() {
        let config = SessionConfig::new()
            .with_key(Key::generate())
            .with_table_name("test_table");

        let mut connect_opts = PgConnectOptions::new();
        connect_opts.log_statements(LevelFilter::Debug);
        connect_opts = connect_opts.database("postgres");
        connect_opts = connect_opts.username("postgres");
        connect_opts = connect_opts.password("password");
        connect_opts = connect_opts.host("localhost");
        connect_opts = connect_opts.port(5432);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(connect_opts)
            .await
            .unwrap();

        let session_store = SessionStore::<SessionPgPool>::new(Some(pool.into()), config);
        //generate the table needed!
        session_store.initiate().await.unwrap();

        let app = Router::new()
            .route("/set_session", get(set_session))
            .route("/test_session", get(test_session))
            .layer(SessionLayer::new(session_store));

        #[derive(Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
        pub struct Test {
            a: u32,
            b: String,
        }

        #[axum::debug_handler]
        async fn set_session(session: Session<SessionPgPool>) -> Redirect {
            let test = Test {
                a: 2,
                b: "Hello World".to_owned(),
            };

            session.set("test", test);
            Redirect::to("/")
        }

        async fn test_session(session: Session<SessionPgPool>) -> String {
            let test: Test = session.get("test").unwrap_or_default();
            let other = Test {
                a: 2,
                b: "Hello World".to_owned(),
            };

            if test == other {
                "Success".to_owned()
            } else {
                "Failed".to_owned()
            }
        }

        let request = Request::<()>::builder()
            .uri("/set_session")
            .body(Body::empty())
            .unwrap();
        let mut response = app.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_redirection());

        //get the session acceptance cookie first.
        let entries = response.headers_mut().get_all(header::SET_COOKIE);
        let mut cookies = Vec::with_capacity(3);

        for entry in entries {
            cookies.push(entry.clone());
        }

        let mut request = Request::builder()
            .uri("/test_session")
            //.header(header::COOKIE, cookie)
            .body(Body::empty())
            .unwrap();

        for cookie in cookies {
            request.headers_mut().append(header::COOKIE, cookie);
        }

        let response = app.clone().oneshot(request).await.unwrap();

        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Success");
    }
}
