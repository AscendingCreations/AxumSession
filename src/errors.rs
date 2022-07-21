use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::error::Error),
    #[error(transparent)]
    HTTP(#[from] http::Error),
    #[error("unknown Session store error")]
    Unknown,
    #[error("Generic Database insert error")]
    GenericInsertError,
    #[error("Generic Database select error")]
    GenericSelectError,
    #[error("Generic Database create error")]
    GenericCreateError,
}
