# Axum_Database_Sessions

Library to Provide a Session management layer. This stores all session data within a MemoryStore internally. Usage of a database is Optional. We also offer the ability to add new 
storage types by implementing them with our AxumDatabasePool trait.

[![https://crates.io/crates/axum_database_sessions](https://img.shields.io/crates/v/axum_database_sessions?style=plastic)](https://crates.io/crates/axum_database_sessions)
[![Docs](https://docs.rs/axum_database_sessions/badge.svg)](https://docs.rs/axum_database_sessions)

## Help

If you need help with this library or have suggestions please go to our [Discord Group](https://discord.gg/xKkm7UhM36)

## Install

Axum Database Sessions uses [`tokio`]

[`tokio`]: https://github.com/tokio-rs/tokio

```toml
# Cargo.toml
[dependencies]
# Postgres + rustls
axum_database_sessions = { version = "4.1.0", features = [ "postgres-rustls"] }
```

#### Cargo Feature Flags
`default`: [`postgres-rustls`]

`sqlite-rustls`: `Sqlx 0.6.0` support for the self-contained [SQLite](https://sqlite.org/) database engine and `rustls`.

`sqlite-native`: `Sqlx 0.6.0` support for the self-contained [SQLite](https://sqlite.org/) database engine and `native-tls`.

`postgres-rustls`: `Sqlx 0.6.0` support for the Postgres database server and `rustls`.

`postgres-native`: `Sqlx 0.6.0` support for the Postgres database server and `native-tls`.

`mysql-rustls`: `Sqlx 0.6.0` support for the MySQL/MariaDB database server and `rustls`.

`mysql-native`: `Sqlx 0.6.0` support for the MySQL/MariaDB database server and `native-tls`.

`redis-db`:  `redis 0.21.5` session support.

# Example

```rust ignore
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_database_sessions::{AxumSession, AxumPgPool, AxumSessionConfig, AxumSessionStore, AxumSessionLayer};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {

    let poll = connect_to_database().await.unwrap();

    //This Defaults as normal Cookies.
    //To enable Private cookies for integrity, and authenticity please check the next Example.
    let session_config = AxumSessionConfig::default()
        .with_table_name("test_table");

    let session_store = AxumSessionStore::<AxumPgPool>::new(Some(poll.clone().into()), session_config);

    //Create the Database table for storing our Session Data.
    session_store.initiate().await.unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(AxumSessionLayer::new(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn greet(session: AxumSession<AxumPgPool>) -> String {
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

To enable private cookies for confidentiality, integrity, and authenticity.
When a Key is set it will automatically set the Cookie into an encypted Private cookie which
both protects the cookies data from prying eye's it also ensures the authenticity of the cookie.
# Example

```rust ignore
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_database_sessions::{AxumSession, AxumPgPool, AxumSessionConfig, AxumSessionStore, AxumSessionLayer, AxumSessionMode, Key};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {
    let session_config = AxumSessionConfig::default()
        .with_table_name("test_table")
        // 'Key::generate()' will generate a new key each restart of the server.
        // If you want it to be more permanent then generate and set it to a config file.
        // If with_key() is used it will set all cookies as private, which guarantees integrity, and authenticity.
        .with_key(Key::generate());

    let session_store = AxumSessionStore::<AxumPgPool>::new(None, session_config);
    session_store.initiate().await.unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(AxumSessionLayer::new(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

To use Axum_database_session in non_persistant mode Set the client to None and import AxumNullPool.
AxumNullPool is always loaded and can be used where you do not want to include any database within the build.
# Example

```rust ignore
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_database_sessions::{AxumSession, AxumNullPool, AxumSessionConfig, AxumSessionStore, AxumSessionLayer};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {
    let session_config = AxumSessionConfig::default()
        .with_table_name("test_table");

    let session_store = AxumSessionStore::<AxumNullPool>::new(None, session_config);

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(AxumSessionLayer::new(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn greet(session: AxumSession<AxumNullPool>) -> String {
    let mut count: usize = session.get("count").await.unwrap_or(0);

    count += 1;
    session.set("count", count).await;

    count.to_string()
}

```


To use Axum_database_session with session mode set as Storable.
# Example

```rust ignore
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_database_sessions::{AxumSession, AxumPgPool, AxumSessionConfig, AxumSessionStore, AxumSessionLayer, AxumSessionMode};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {
    let session_config = AxumSessionConfig::default()
        .with_table_name("test_table").with_mode(AxumSessionMode::Storable);

    let session_store = AxumSessionStore::<AxumPgPool>::new(None, session_config);
    session_store.initiate().await.unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(AxumSessionLayer::new(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

//No need to set the sessions accepted or not with gdpr mode disabled
async fn greet(session: AxumSession<AxumPgPool>) -> String {
    let mut count: usize = session.get("count").await.unwrap_or(0);

    // Allow the Session data to be keep in memory and the database for the lifetime.
    session.set_store(true).await;
    count += 1;
    session.set("count", count).await;

    count.to_string()
}

```