pub mod api;
mod image;

#[derive(thiserror::Error, Debug)]
pub enum LemmyError {
    #[error(transparent)]
    ReqwestError( #[from] reqwest::Error ),
    #[error(transparent)]
    IoError( #[from] std::io::Error ),
    #[error("ResponeError: {0}")]
    ResponeError(String),
}
