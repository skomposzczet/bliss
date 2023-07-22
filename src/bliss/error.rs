#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ReqwestError( #[from] reqwest::Error ),
    #[error(transparent)]
    IoError( #[from] std::io::Error ),
    #[error("Error: {0}")]
    BlissError(String),
}

