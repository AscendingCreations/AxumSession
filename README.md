# Axum_database_Sessions

Library to Provide a Sqlx Database Session management layer. You must also include Tower_cookies in order to use these Library.

You must choose only one of ['postgres', 'mysql', 'sqlite'] features to use this library.

[![https://crates.io/crates/axum_database_sessions](https://img.shields.io/badge/crates.io-v0.1.0-blue)](https://crates.io/crates/axum_database_sessions)
[![Docs](https://docs.rs/axum_database_sessions/badge.svg)](https://docs.rs/axum_database_sessions)

# Example

```rust
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_database_sessions::{AxumSession, AxumSessionConfig, AxumSessionStore, axum_session_runner};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {
    # async {
    let poll = connect_to_database().await.unwrap();

    let session_config = AxumSessionConfig::default()
        .with_database("test")
        .with_table_name("test_table");

    let session_store = AxumSessionStore::new(poll.clone().into(), session_config);

    // build our application with some routes
    let app = Router::new()
        .route("/greet/:name", get(greet))
        .layer(tower_cookies::CookieManagerLayer::new())
        .layer(axum_session_runner!(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    # };
}

async fn greet(session: AxumSession) -> String {
    let mut count: usize = session.get("count").await.unwrap_or(0);
    count += 1;
    session.set("count", count).await;

    count.to_string()
}

async fn connect_to_database() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
    // ...
    # unimplemented!()
}
```
