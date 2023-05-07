use axum::{routing::get, Router};
use axum_session::{Session, SessionConfig, SessionLayer, SessionRedisPool, SessionStore};

#[tokio::main]
async fn main() {
    // please consider using dotenvy to get this
    // please check the docker-compose file included for the redis image used here
    let redis_url = "redis://default:YourSecretPassWord@127.0.0.1:6379/0";

    let client =
        redis::Client::open(redis_url).expect("Error while tryiong to open the redis connection");

    // No need here to specify a table name because redis does not support tables
    let session_config = SessionConfig::default();

    let session_store =
        SessionStore::<SessionRedisPool>::new(Some(client.clone().into()), session_config);

    // build our application with a single route
    let app = Router::new()
        .route("/", get(root))
        // `POST /users` goes to `counter`
        .route("/counter", get(counter))
        .layer(SessionLayer::new(session_store)); // adding the crate plugin ( layer ) to the project

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn counter(session: Session<SessionRedisPool>) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);
    count += 1;
    session.set("count", count);
    // consider use better Option handling here instead of expect
    let new_count = session.get::<usize>("count").expect("error setting count");
    format!("We have set the counter to redis {new_count}")
}
