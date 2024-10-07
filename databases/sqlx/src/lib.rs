#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![warn(clippy::all, nonstandard_style, future_incompatible)]
#![forbid(unsafe_code)]

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::*;

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::*;

#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "mysql")]
pub use mysql::*;

#[cfg(test)]
mod tests {
    use super::SessionPgPool;
    use axum::{
        body::Body,
        http::{header, Request},
        response::Redirect,
        routing::get,
        Router,
    };
    #[allow(unused_imports)]
    use axum_session::*;
    use http_body_util::BodyExt;
    use log::LevelFilter;
    use serde::{Deserialize, Serialize};
    use sqlx::{
        postgres::{PgConnectOptions, PgPoolOptions},
        ConnectOptions,
    };
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn basic() {
        let config = SessionConfig::new()
            .with_key(Key::generate())
            .with_table_name("sessions_table");

        let mut connect_opts = PgConnectOptions::new();
        connect_opts = connect_opts.log_statements(LevelFilter::Debug);
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

        //create session_store and generate the table needed!
        let session_store = SessionStore::<SessionPgPool>::new(Some(pool.into()), config)
            .await
            .unwrap();

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

        let bytes = response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        let body = String::from_utf8(bytes).unwrap();
        assert_eq!(body, "Success");
    }
}
