use axum::{routing::get, Router};
use axum_session::{Session, SessionConfig, SessionLayer, SessionSqlitePool, SessionStore};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::{net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let poll = connect_to_database().await;
    // A premade saved and loaded Key.
    let key = [
        0, 6, 244, 144, 182, 219, 119, 30, 186, 208, 221, 180, 0, 206, 248, 7, 135, 27, 241, 0, 43,
        32, 128, 232, 76, 0, 40, 46, 1, 3, 220, 0, 42, 165, 66, 0, 36, 193, 19, 251, 196, 145, 38,
        0, 182, 195, 0, 64, 143, 0, 241, 0, 0, 228, 0, 0, 183, 128, 124, 0, 175, 62, 66, 30,
    ];

    //This Defaults as normal Cookies.
    //To enable Private cookies for integrity, and authenticity please check the next Example.
    let session_config = SessionConfig::default()
        .with_table_name("sessions_table")
        .with_key(axum_session::Key::from(&key));

    // create SessionStore and initiate the database tables
    let session_store =
        SessionStore::<SessionSqlitePool>::new(Some(poll.clone().into()), session_config)
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

async fn greet(session: Session<SessionSqlitePool>) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);

    count += 1;
    session.set("count", count);

    count.to_string()
}

async fn connect_to_database() -> SqlitePool {
    // to use sqlite in memory use sqlite::memory:
    let connect_opts = SqliteConnectOptions::from_str("sqlite:data.db")
        .unwrap()
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_opts)
        .await
        .unwrap()
}
