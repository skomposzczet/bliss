use crate::lemmy::LemmyError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    LemmyError( #[from] LemmyError ),
    #[error(transparent)]
    IoError( #[from] std::io::Error ),
    #[error("Error: {0}")]
    BlissError(String),
}

