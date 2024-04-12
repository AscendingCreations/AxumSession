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

- Cookies or Header Store of Generated Session UUID and a Store Boolean.
- Uses a DatabasePool Trait so you can implement your own Sub Storage Layer.
- Convenient API for `Session` no need to mark as Read or Write making Usage Easier. 
- Uses `dashmap` for internal memory lookup and storage to achieve high throughput.
- Uses Serdes for Data Serialization so it can store any Serdes supported type's into the Sessions data.
- Supports Redis, SurrealDB, MongoDB and SQLx optional Databases out of the Box.
- Supports Memory Only usage. No need to use a persistant database.
- Supports Cookie and Header Signing for integrity, and authenticity.
- Supports Database Session Data Encryption for confidentiality, integrity.
- Supports SessionID renewal for enhanced Security.
- Optional Fastbloom key storage for reduced Database lookups during new UUID generation. Boosting Bandwidth.
- Optional Rest Mode that Disables Cookies and uses the Header values instead.
- uses `#![forbid(unsafe_code)]` to ensure everything is implemented as safe rust.
- has an `advanced` API to allow further control of a session.
- uses IP address's and user agent to deter spoofing of signed cookies and headers.

## 🚨 Help

If you need help with this library or have suggestions please go to our [Discord Group](https://discord.gg/gVXNDwpS3Z)

## 📦 Install

Axum Session uses [`tokio`]. 
to your cargo include for Axum Session.

[`tokio`]: https://github.com/tokio-rs/tokio

```toml
# Cargo.toml
[dependencies]
# Postgres + rustls
axum_session = { version = "0.13.0" }
```

## 📱 Cargo Feature Flags

| Features                      | Description                                                                                    |
| ----------------------------- | ---------------------------------------------------------------------------------------------- |
| `advanced`                    | Enable functions allowing more direct control over the sessions.                               |
| `rest_mode`                   | Disables Cookie Handlering In place of Header only usage for Rest API Requests and Responses.  |
| `key-store`                   | Enabled the optional key storage. Will increase ram usage based on Fastbloom settings.         |

## 🔎 Example Default Setup

You can find examples within the [`Repository`](https://github.com/AscendingCreations/AxumSession/tree/main/examples) 

```rust ignore
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_session::{Session, SessionPgPool, SessionConfig, SessionStore, SessionLayer};
use axum::{
    Router,
    routing::get,
};
use tokio::net::TcpListener;

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
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    debug!("listening on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
