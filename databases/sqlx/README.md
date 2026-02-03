<h1 align="center">
Axum Session Sqlx
</h1>

[![https://crates.io/crates/axum_session_sqlx](https://img.shields.io/crates/v/axum_session_sqlx?style=plastic)](https://crates.io/crates/axum_session_sqlx)
[![Docs](https://docs.rs/axum_session_sqlx/badge.svg)](https://docs.rs/axum_session_sqlx)
[![Discord Server](https://img.shields.io/discord/81844480201728000?label=&labelColor=6A7EC2&logo=discord&logoColor=ffffff&color=7389D8)](https://discord.gg/gVXNDwpS3Z)

## 📑 Overview

<p align="center">
`axum_session_sqlx` provide's a Persistent SQL Database Storage for Axum Session.
</p>

## 🚨 Help

If you need help with this library or have suggestions please go to our [Discord Group](https://discord.gg/gVXNDwpS3Z)

## 📦 Install

Axum Session uses [`tokio`]. 
By Default Axum Session Sqlx uses `postgres` and `tls-rustls` so if you need tokio native TLS please add `default-features = false` 
to your cargo include for Axum Session Sqlx.

```toml
# Cargo.toml
[dependencies]
axum_session = { version = "0.18.0" }
# Postgres + rustls
axum_session_sqlx = { version = "0.7.0", features = [ "postgres", "tls-rustls"] }
```

## 📱 Cargo Feature Flags

You must choose a Database and a tls mode if you disable defaults.

| Features                      | Description                                                        |
| ----------------------------- | ------------------------------------------------------------------ |
| `default`                     | `postgres-rustls` and `tls-rustls`                                 |
| `sqlite`                      | Enables sqlite usage.                                              |
| `postgres`                    | Enables postgres usage.                                            |
| `mysql`                       | Enables mysql usage.                                               |
| `tls-rustls`                  | Uses `rustls` for TLS encryption. Must choose one of these.        |
| `tls-native-tls`              | Uses `native-tls` for TLS encryption. Must choose one of these.    |

## 🔎 Examples

You can locate the example files within the [`Repository`](https://github.com/AscendingCreations/AxumSession/tree/main/examples)  

