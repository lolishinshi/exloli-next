use thiserror::Error;

pub type Result<T> = std::result::Result<T, EhError>;

#[derive(Debug, Error)]
pub enum EhError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("invalid exhentai URL: {0}")]
    InvalidURL(String),
    #[error("tokio join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}
