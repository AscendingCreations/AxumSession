use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Router};
use axum_session::{
    Key, SessionConfig, SessionLayer, SessionStore, SessionSurrealPool, SessionSurrealSession,
};
use http_body_util::BodyExt;
use surrealdb::engine::any::{connect, Any};
use surrealdb::opt::auth::Root;
use tower::{Service, ServiceExt};

#[tokio::main]
async fn main() {
    // Create the Surreal connection.
    let db = connect("ws://localhost:8080").await.unwrap();

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
    // We are also generating a encryption key for storage.
    let session_config = SessionConfig::default().with_key(Key::generate());

    // create SessionStore and initiate the database tables
    let session_store =
        SessionStore::new(Some(SessionSurrealPool::new(db.clone())), session_config)
            .await
            .unwrap();

    // build our application with a single route
    let mut app = Router::new()
        .route("/", get(root))
        // `POST /users` goes to `counter`
        .route("/counter", get(counter))
        .layer(SessionLayer::new(session_store)); // adding the crate plugin ( layer ) to the project

    // Lets build our first Request so we can get our UUID key.
    let request = Request::builder()
        .uri("/")
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    // Get the Response back.
    let response = <axum::Router as tower::ServiceExt<Request<Body>>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    // Get our Session ID from the Response
    let mut sessionid = response
        .headers()
        .get("session")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    for _ in 0..10 {
        // Do some more Requests and get the Responses for counter
        let response = <axum::Router as tower::ServiceExt<Request<Body>>>::ready(&mut app)
            .await
            .unwrap();
        let response = response
            .call(
                Request::builder()
                    .uri("/counter")
                    .method(Method::GET)
                    .header("session", sessionid)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // We pull out and reset the SessionID as this can Change based on user end designs for Security.
        // If you also use Per session keys and storage/manual you will also need to get and return those as well.
        sessionid = response
            .headers()
            .get("session")
            .cloned()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        // print our Status
        println!("Status: {:?}", response.status());

        // Get and print our returned Body String.
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        println!("Body: {:?}", body);
    }

    println!("Finished!")
}

async fn root() -> Response {
    StatusCode::OK.into_response()
}

async fn counter(session: SessionSurrealSession<Any>) -> Response {
    let mut count: usize = session.get("count").unwrap_or(0);
    count += 1;
    session.set("count", count);
    let sessions_count = session.count().await;
    // consider use better Option handling here instead of expect
    let new_count = session.get::<usize>("count").expect("error setting count");
    (
        StatusCode::OK,
        format!(
            "We have set the counter to surreal: {new_count}, Sessions Count: {sessions_count}"
        ),
    )
        .into_response()
}
