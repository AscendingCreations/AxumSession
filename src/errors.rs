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
    #[cfg(feature = "redis")]
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::error::Error),
    #[error(transparent)]
    HTTP(#[from] http::Error),
    #[error(transparent)]
    UUID(#[from] uuid::Error),
    #[cfg(feature = "surrealdb_tag")]
    #[error(transparent)]
    SurrealDBError(#[from] surrealdb::Error),
    #[cfg(feature = "surrealdb_tag")]
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
}
