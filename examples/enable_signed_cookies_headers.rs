use axum::{routing::get, Router};
use axum_session::{Key, SessionConfig, SessionLayer};
use axum_session_sqlx::{SessionPgSession, SessionPgSessionStore};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let poll = connect_to_database().await;

    //This Defaults as normal Cookies.
    //To enable Signed cookies for integrity, and authenticity you just need to use with_key as shown below.
    let session_config = SessionConfig::default()
        .with_table_name("sessions_table")
        // 'Key::generate()' will generate a new key each restart of the server.
        // If you want it to be more permanent then generate and set it to a config file.
        // If with_key() is used it will set all cookies or headers as signed, which guarantees integrity, and authenticity.
        // We dont need Private cookies since we do not Send any sensitive data to store inside the cookie.
        .with_key(Key::generate())
        // This is how we would Set a Database Key to encrypt as store our per session data.
        .with_database_key(Key::generate());

    // create SessionStore and initiate the database tables
    let session_store = SessionPgSessionStore::new(Some(poll.clone().into()), session_config)
        .await
        .unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(SessionLayer::new(session_store));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn greet(session: SessionPgSession) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);

    count += 1;
    session.set("count", count);

    count.to_string()
}

async fn connect_to_database() -> PgPool {
    let mut connect_opts = PgConnectOptions::new();
    connect_opts = connect_opts
        .database("test")
        .username("test")
        .password("password")
        .host("127.0.0.1")
        .port(5432);

    PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connect_opts)
        .await
        .unwrap()
}
