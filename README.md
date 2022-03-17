# Axum_database_Sessions

Library to Provide a Sqlx Database Session management layer. You must also include Tower_cookies in order to use these Library.

You must choose only one of ['postgres', 'mysql', 'sqlite'] features to use this library.

[![https://crates.io/crates/axum_database_sessions](https://img.shields.io/badge/crates.io-v0.3.0-blue)](https://crates.io/crates/axum_database_sessions)
[![Docs](https://docs.rs/axum_database_sessions/badge.svg)](https://docs.rs/axum_database_sessions)

## Install

Axum Database Sessions uses [`tokio`] runtime along with ['sqlx']; it supports [`native-tls`] and [`rustls`] TLS backends. When adding the dependency, you must chose a database feature that is `DatabaseType` and a `tls` backend. You can only choose one database type and one TLS Backend.

[`tokio`]: https://github.com/tokio-rs/tokio
[`native-tls`]: https://crates.io/crates/native-tls
[`rustls`]: https://crates.io/crates/rustls
[`sqlx`]: https://crates.io/crates/sqlx

```toml
# Cargo.toml
[dependencies]
# Postgres + rustls
axum_database_sessions = { version = "0.3", features = [ "postgres", "rustls"] }
```

#### Cargo Feature Flags
`sqlite`: `Sqlx` support for the self-contained [SQLite](https://sqlite.org/) database engine.
`postgres`: `Sqlx` support for the Postgres database server.
`mysql`: `Sqlx` support for the MySQL/MariaDB database server.
`native-tls`: Use the `tokio` runtime and `native-tls` TLS backend.
`rustls`: Use the `tokio` runtime and `rustls` TLS backend.

# Example

```rust no_run
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_database_sessions::{AxumSession, AxumSessionConfig, AxumSessionStore, AxumSessionLayer};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {

    let poll = connect_to_database().await.unwrap();

    let session_config = AxumSessionConfig::default()
        .with_table_name("test_table");

    let session_store = AxumSessionStore::new(Some(poll.clone().into()), session_config);

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(AxumSessionLayer::new(session_store))
        .layer(tower_cookies::CookieManagerLayer::new());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn greet(session: AxumSession) -> String {
    let mut count: usize = session.get("count").await.unwrap_or(0);
    count += 1;
    session.set("count", count).await;

    count.to_string()
}

async fn connect_to_database() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
    // ...
    unimplemented!()
}
```

To use Axum_database_session in non_persistant mode Set the client to None.
# Example

```rust no_run
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_database_sessions::{AxumSession, AxumSessionConfig, AxumSessionStore, AxumSessionLayer};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {
    let session_config = AxumSessionConfig::default()
        .with_table_name("test_table");

    let session_store = AxumSessionStore::new(None, session_config);

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(AxumSessionLayer::new(session_store))
        .layer(tower_cookies::CookieManagerLayer::new());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn greet(session: AxumSession) -> String {
    let mut count: usize = session.get("count").await.unwrap_or(0);
    count += 1;
    session.set("count", count).await;

    count.to_string()
}

```