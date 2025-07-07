use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::{routing::get, Router};
use axum_session::{ReadOnlySession, SessionConfig, SessionLayer, SessionStore};
use axum_session_surreal::{SessionSurrealPool, SessionSurrealSession};
use hyper::StatusCode;
use std::net::SocketAddr;
use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Create the Surreal connection.
    let db = connect("ws://localhost:8000").await.unwrap();

    // sign in as our account.
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .unwrap();

    // Set the database and namespace we will function within.
    db.use_ns("test").use_db("test").await.unwrap();

    // No need here to specify a table name because redis does not support tables
    let session_config = SessionConfig::default();

    // create SessionStore and initiate the database tables
    let session_store =
        SessionStore::new(Some(SessionSurrealPool::new(db.clone())), session_config)
            .await
            .unwrap();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(root))
        // `POST /users` goes to `counter`
        .route("/counter", get(counter))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(SessionLayer::new(session_store)); // adding the crate plugin ( layer ) to the project

    // run it with hyper on localhost:3000
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn auth_middleware(
    session: ReadOnlySession<SessionSurrealPool<Any>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let count: usize = session.get("count").unwrap_or(0);
    println!("count: {count}");
    // e.g: check if user is logged in
    // let login: Option<String> = session.get("login");
    // match login {
    //     None => Err(StatusCode::UNAUTHORIZED),
    //     Some(_) => Ok(next.run(request).await),
    // }

    // assume logged-in
    Ok(next.run(request).await)
}

async fn counter(session: SessionSurrealSession<Any>) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);
    count += 1;
    session.set("count", count);
    let sessions_count = session.count().await;
    // consider use better Option handling here instead of expect
    let new_count = session.get::<usize>("count").expect("error setting count");
    format!("We have set the counter to surreal: {new_count}, Sessions Count: {sessions_count}")
}
