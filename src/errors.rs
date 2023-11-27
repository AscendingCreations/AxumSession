use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Decode(#[from] base64::DecodeError),
    #[cfg(any(
        feature = "postgres-rustls",
        feature = "postgres-native",
        feature = "sqlite-rustls",
        feature = "sqlite-native",
        feature = "mysql-rustls",
        feature = "mysql-native"
    ))]
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[cfg(any(feature = "redis-db", feature = "redis-clusterdb"))]
    #[error(transparent)]
    RedisPool(#[from] redis_pool::errors::RedisPoolError),
    #[cfg(any(feature = "redis-db", feature = "redis-clusterdb"))]
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[cfg(feature = "mongodb")]
    #[error(transparent)]
    Mongodb(#[from] mongodb::error::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::error::Error),
    #[error(transparent)]
    HTTP(#[from] http::Error),
    #[error(transparent)]
    UUID(#[from] uuid::Error),
    #[error(transparent)]
    UTF8(#[from] std::string::FromUtf8Error),
    #[cfg(feature = "surreal")]
    #[error(transparent)]
    SurrealDBError(#[from] surrealdb::Error),
    #[cfg(feature = "surreal")]
    #[error(transparent)]
    SurrealDBDatabaseError(#[from] surrealdb::error::Db),
    #[error("unknown Session store error")]
    Unknown,
    #[error("Generic Database insert error {0}")]
    GenericInsertError(String),
    #[error("Generic Database select error {0}")]
    GenericSelectError(String),
    #[error("Generic Database create error {0}")]
    GenericCreateError(String),
    #[error("Generic Database delete error {0}")]
    GenericDeleteError(String),
    #[error("{0}")]
    GenericNotSupportedError(String),
    #[error("Session was not found. Either the session was unloaded or was never created.")]
    NoSessionError,
    #[error(
        "The Session Exists but is outdated, either renew it or remove it. \n
    Session will get removed on next Session request purge update if no changes are done."
    )]
    OldSessionError,
}
