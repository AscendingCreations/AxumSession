use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Decode(#[from] base64::DecodeError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::error::Error),
    #[error(transparent)]
    HTTP(#[from] http::Error),
    #[error(transparent)]
    UUID(#[from] uuid::Error),
    #[error(transparent)]
    UTF8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    DatabaseError(#[from] crate::DatabaseError),
    #[error("unknown Session store error")]
    Unknown,
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
