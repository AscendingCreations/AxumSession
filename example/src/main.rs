use axum::{routing::get, Router};
use axum_session::{Session, SessionConfig, SessionLayer, SessionSqlitePool, SessionStore};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::{net::SocketAddr, str::FromStr};

#[tokio::main]
async fn main() {
    let poll = connect_to_database().await;

    //This Defaults as normal Cookies.
    //To enable Private cookies for integrity, and authenticity please check the next Example.
    let session_config = SessionConfig::default().with_table_name("sessions_table");

    let session_store =
        SessionStore::<SessionSqlitePool>::new(Some(poll.clone().into()), session_config);

    //Create the Database table for storing our Session Data.
    session_store.initiate().await.unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(SessionLayer::new(session_store));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
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
    let connect_opts = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_opts)
        .await
        .unwrap()
}
