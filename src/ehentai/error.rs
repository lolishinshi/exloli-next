use thiserror::Error;

use crate::utils::xpath;

pub type Result<T> = std::result::Result<T, EhError>;

#[derive(Debug, Error)]
pub enum EhError {
    #[error("xpath error: {0}")]
    XPathError(#[from] xpath::XpathError),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}
