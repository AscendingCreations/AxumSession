use axum::{routing::get, Router};
use axum_session::{SessionAnyPool, SessionAnySession, SessionConfig, SessionLayer, SessionStore};
use axum_session_sqlx::{SessionPgPool, SessionSqlitePool};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let poll = connect_to_database(false).await;

    //This Defaults as normal Cookies.
    //To enable Private cookies for integrity, and authenticity please check the next Example.
    let session_config = SessionConfig::default().with_table_name("sessions_table");

    // create SessionStore and initiate the database tables
    let session_store = SessionStore::<SessionAnyPool>::new(Some(poll), session_config)
        .await
        .unwrap();

    // build our application with some routes
    let app = Router::new()
        .route("/greet", get(greet))
        .layer(SessionLayer::new(session_store));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn greet(session: SessionAnySession) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);

    count += 1;
    session.set("count", count);

    count.to_string()
}

async fn connect_to_database(is_production: bool) -> SessionAnyPool {
    if is_production {
        let mut connect_opts = PgConnectOptions::new();

        connect_opts = connect_opts
            .database("test")
            .username("test")
            .password("password")
            .host("127.0.0.1")
            .port(5432);

        SessionAnyPool::new(SessionPgPool::from(
            PgPoolOptions::new()
                .max_connections(5)
                .connect_with(connect_opts)
                .await
                .unwrap(),
        ))
    } else {
        let connect_opts = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();

        SessionAnyPool::new(SessionSqlitePool::from(
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(connect_opts)
                .await
                .unwrap(),
        ))
    }
}
