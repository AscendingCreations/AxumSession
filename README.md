<h1 align="center">
Axum Session
</h1>

<p>
`axum_session` provide's a Session management middleware that stores all session data within a MemoryStore internally. It can also save data to an optional persistent database.
It uses a Cookie inserted UUID to sync back to the memory store. Formally known as Axum Database Sessions.
</p>

[![https://crates.io/crates/axum_session](https://img.shields.io/crates/v/axum_session?style=plastic)](https://crates.io/crates/axum_session)
[![Docs](https://docs.rs/axum_session/badge.svg)](https://docs.rs/axum_session)

- Cookies only Store a Generated Session UUID and a Storable Boolean.
- Uses a DatabasePool Trait so you can implement your own Sub Storage Layer.
- Convenient API for `Session` no need to mark as Read or Write making Usage Easier. 
- Uses `dashmap` for internal memory lookup and storage to achieve high throughput.
- Uses Serdes for Data Serialization so it can store any Serdes supported type's into the Sessions data.
- Supports Redis, SurrealDB and SQLx optional Databases out of the Box.
- Supports Memory Only usage. No need to use a persistant database.
- Supports Per Session SessionID cookie Encryption for enhanced Security.

## Help

If you need help with this library or have suggestions please go to our [Discord Group](https://discord.gg/gVXNDwpS3Z)

## Install

Axum Session uses [`tokio`]. 
By Default Axum Session uses `postgres-rustls` so if you need tokio native TLS please add `default-features = false` 
to your cargo include for Axum Session.

[`tokio`]: https://github.com/tokio-rs/tokio

```toml
# Cargo.toml
[dependencies]
# Postgres + rustls
axum_session = { version = "0.2.0", features = [ "postgres-rustls"] }
```

#### Cargo Feature Flags
`default`: [`postgres-rustls`]

`sqlite-rustls`: `Sqlx 0.6.0` support for the self-contained [SQLite](https://sqlite.org/) database engine and `rustls`.

`sqlite-native`: `Sqlx 0.6.0` support for the self-contained [SQLite](https://sqlite.org/) database engine and `native-tls`.

`postgres-rustls`: `Sqlx 0.6.0` support for the Postgres database server and `rustls`.

`postgres-native`: `Sqlx 0.6.0` support for the Postgres database server and `native-tls`.

`mysql-rustls`: `Sqlx 0.6.0` support for the MySQL/MariaDB database server and `rustls`.

`mysql-native`: `Sqlx 0.6.0` support for the MySQL/MariaDB database server and `native-tls`.

`redis-db`:  `redis 0.23.0` session support.

`surrealdb-rocksdb`: `surrealdb 1.0.0-beta.9+20230402` support for rocksdb.

`surrealdb-tikv` : `surrealdb 1.0.0-beta.9+20230402` support for tikv.

`surrealdb-indxdb` : `surrealdb 1.0.0-beta.9+20230402` support for indxdb.

`surrealdb-fdb-?_?` : `surrealdb 1.0.0-beta.9+20230402` support for fdb versions 5_1, 5_2, 6_0, 6_1, 6_2, 6_3, 7_0, 7_1. Replace ?_? with version.

`surrealdb-mem` : `surrealdb 1.0.0-beta.9+20230402` support for mem.

# Example

```rust ignore
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_session::{Session, SessionPgPool, SessionConfig, SessionStore, SessionLayer};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {

    let poll = connect_to_database().await.unwrap();

    //This Defaults as normal Cookies.
    //To enable Private cookies for integrity, and authenticity please check the next Example.
    let session_config = SessionConfig::default()
        .with_table_name("test_table");

    let session_store = SessionStore::<SessionPgPool>::new(Some(poll.clone().into()), session_config);

    //Create the Database table for storing our Session Data.
    session_store.initiate().await.unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(SessionLayer::new(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn greet(session: Session<SessionPgPool>) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);

    count += 1;
    session.set("count", count);

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
use axum_session::{Session, SessionPgPool, SessionConfig, SessionStore, SessionLayer, SessionMode, Key, SecurityMode};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {
    let session_config = SessionConfig::default()
        .with_table_name("test_table")
        // 'Key::generate()' will generate a new key each restart of the server.
        // If you want it to be more permanent then generate and set it to a config file.
        // If with_key() is used it will set all cookies as private, which guarantees integrity, and authenticity.
        .with_key(Key::generate())
        // This is how we would Set a Database Key to encrypt as store our per session keys. 
        // This MUST be set in order to use SecurityMode::PerSession.
        .with_database_key(Key::generate())
        // This is How you will enable PerSession SessionID Private Cookie Encryption. When enabled it will
        // Encrypt the SessionID and Storage with an Encryption key generated and stored per session.
        // This allows for Key renewing without needing to force the entire Session from being destroyed.
        // This Also helps prevent impersonation attempts. 
        .with_security_mode(SecurityMode::PerSession);

    let session_store = SessionStore::<SessionPgPool>::new(None, session_config);
    session_store.initiate().await.unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(SessionLayer::new(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

To use axum_session in non_persistant mode Set the client to None and import SessionNullPool.
SessionNullPool is always loaded and can be used where you do not want to include any database within the build.
# Example

```rust ignore
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_session::{Session, SessionNullPool, SessionConfig, SessionStore, SessionLayer};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {
    let session_config = SessionConfig::default()
        .with_table_name("test_table");

    let session_store = SessionStore::<SessionNullPool>::new(None, session_config);

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(SessionLayer::new(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn greet(session: Session<SessionNullPool>) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);

    count += 1;
    session.set("count", count);

    count.to_string()
}

```


To use axum_session with session mode set as Storable.
# Example

```rust ignore
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_session::{Session, SessionPgPool, SessionConfig, SessionStore, SessionLayer, SessionMode};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {
    let session_config = SessionConfig::default()
        .with_table_name("test_table").with_mode(SessionMode::Storable);

    let session_store = SessionStore::<SessionPgPool>::new(None, session_config);
    session_store.initiate().await.unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(SessionLayer::new(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

//No need to set the sessions accepted or not with gdpr mode disabled
async fn greet(session: Session<SessionPgPool>) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);

    // Allow the Session data to be keep in memory and the database for the lifetime.
    session.set_store(true);
    count += 1;
    session.set("count", count);

    count.to_string()
}

```

## Session Login and Authentication via `axum_session_auth`

For user login, login caching and authentication please see [`axum_session_auth`](https://github.com/AscendingCreations/AxumSessionsAuth).
