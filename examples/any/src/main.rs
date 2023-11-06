use axum::{routing::get, Router};
use axum_session::{
    SessionAnyPool, SessionConfig, SessionLayer, SessionPgSession, SessionSqlitePool, SessionStore,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let poll = connect_to_database().await;

    //This Defaults as normal Cookies.
    //To enable Private cookies for integrity, and authenticity please check the next Example.
    let session_config = SessionConfig::default().with_table_name("sessions_table");

    // create SessionStore and initiate the database tables
    let session_store =
        SessionStore::<SessionAnyPool>::new(Some(poll.clone().into()), session_config)
            .await
            .unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(SessionLayer::new(session_store));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn greet(session: SessionPgSession) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);

    count += 1;
    session.set("count", count);

    count.to_string()
}

async fn connect_to_database() -> SessionAnyPool {
    let connect_opts = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();

    let sqlite_pool = SessionSqlitePool::from(
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connect_opts)
            .await
            .unwrap(),
    );
    SessionAnyPool::new(Arc::new(sqlite_pool))
}
