<h1 align="center">
Axum Session
</h1>

[![https://crates.io/crates/axum_session](https://img.shields.io/crates/v/axum_session?style=plastic)](https://crates.io/crates/axum_session)
[![Docs](https://docs.rs/axum_session/badge.svg)](https://docs.rs/axum_session)

## 📑 Overview

<p align="center">
`axum_session` provide's a Session management middleware that stores all session data within a MemoryStore internally. 
Optionally it can save data to a persistent database for long term storage.
Uses Cookie or Header stored UUID's to sync back to the session store.
</p>

- Cookies or Header Store of Generated Session UUID and a Storable Boolean.
- Uses a DatabasePool Trait so you can implement your own Sub Storage Layer.
- Convenient API for `Session` no need to mark as Read or Write making Usage Easier. 
- Uses `dashmap` for internal memory lookup and storage to achieve high throughput.
- Uses Serdes for Data Serialization so it can store any Serdes supported type's into the Sessions data.
- Supports Redis, SurrealDB, MongoDB and SQLx optional Databases out of the Box.
- Supports Memory Only usage. No need to use a persistant database.
- Supports Per Session SessionID cookie Encryption for enhanced Security.
- Supports SessionID renewal for enhanced Security.
- Optional Fastbloom key storage for reduced Database lookups during new UUID generation. Boosting Bandwidth.
- Optional Rest Mode that Disables Cookies and uses the Header values instead.
- uses `#![forbid(unsafe_code)]` to ensure everything is implemented as safe rust.

## 🚨 Help

If you need help with this library or have suggestions please go to our [Discord Group](https://discord.gg/gVXNDwpS3Z)

## 📦 Install

Axum Session uses [`tokio`]. 
By Default Axum Session uses `postgres-rustls` so if you need tokio native TLS please add `default-features = false` 
to your cargo include for Axum Session.

[`tokio`]: https://github.com/tokio-rs/tokio

```toml
# Cargo.toml
[dependencies]
# Postgres + rustls
axum_session = { version = "0.6.0", features = [ "postgres-rustls"] }
```

## 📱 Cargo Feature Flags
`default`: [`postgres-rustls`]

`rest_mode`: Disables Cookie Handlering In place of Header only usage for Rest API Requests and Responses.

`key-store`: Enabled the optional key storage. Will increase ram usage based on Fastbloom settings.

`sqlite-rustls`: `Sqlx 0.7.0` support for the self-contained [SQLite](https://sqlite.org/) database engine and `rustls`.

`sqlite-native`: `Sqlx 0.7.0` support for the self-contained [SQLite](https://sqlite.org/) database engine and `native-tls`.

`postgres-rustls`: `Sqlx 0.7.0` support for the Postgres database server and `rustls`.

`postgres-native`: `Sqlx 0.7.0` support for the Postgres database server and `native-tls`.

`mysql-rustls`: `Sqlx 0.7.0` support for the MySQL/MariaDB database server and `rustls`.

`mysql-native`: `Sqlx 0.7.0` support for the MySQL/MariaDB database server and `native-tls`.

`redis-db`:  `redis_pool 0.2.1` session support. Enables Redis Client Pool

`redis-clusterdb`:  `redis_pool 0.2.0` session support. Enabled Redis ClusterClient Pool.

`surrealdb-rocksdb`: `surrealdb 1.0.0` support for rocksdb.

`surrealdb-tikv` : `surrealdb 1.0.0` support for tikv.

`surrealdb-fdb-?_?` : `surrealdb 1.0.0` support for fdb versions 6_1, 6_2, 6_3, 7_0, 7_1. Replace ?_? with version.

`surrealdb-mem` : `surrealdb 1.0.0` support for mem.

`mongo` : `mongodb 2.6.1` support for mongo.

## 🔎 Example Default Setup

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
        .with_table_name("sessions_table");

    // create SessionStore and initiate the database tables
    let session_store = SessionStore::<SessionPgPool>::new(Some(poll.clone().into()), session_config).await.unwrap();

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

## 🔐 Example Encryption.
### Enable Cookie and Header UUID Encryption and Database Key encryption for Per_session Keys.

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
        .with_table_name("sessions_table")
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

    // create SessionStore and initiate the database tables
    let session_store = SessionStore::<SessionPgPool>::new(None, session_config).await.unwrap();

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

## 💿 Example SessionNullPool for non_persistant Memory store only.

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
        .with_table_name("sessions_table");

    // create SessionStore and initiate the database tables
    let session_store = SessionStore::<SessionNullPool>::new(None, session_config).await.unwrap();

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

## 📒 Example session mode set as Storable

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
        .with_table_name("sessions_table").with_mode(SessionMode::Storable);

    // create SessionStore and initiate the database tables
    let session_store = SessionStore::<SessionPgPool>::new(None, session_config).await.unwrap();

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
## 🔑 Key Store Details

To enable and use fastbloom key storage for less database lookups. 
Add the feature `"key-store"` to the crate’s features. This feature will increase the ram usage server side.
but will heavily improve the bandwidth limitations and reduce latency of returns from the server. 
This is based on how much the `filter_expected_elements` and `filter_false_positive_probability` are set too.
The higher they are the more ram is used. You will also need to Enable the bloom filter in the config for it to be used. By default, 
the `use_bloom_filters` is enabled and these config options exist whither or not the feature is enabled.
Please refer to `with_filter_expected_elements` and `with_filter_false_positive_probability` within the documents to set the options.
Otherwise stick with the default settings which should work in most situations. Just do note these options provide on how many False positives
could possibly occur when comparing a UUID to what currently exists, which means it will keep trying till it finds none that match. 
Higher values decrease the chance of a false positive but increase ram usage.

## 😎 Session Login and Authentication via `axum_session_auth`

For user login, login caching and authentication please see [`axum_session_auth`](https://github.com/AscendingCreations/AxumSessionAuth).
