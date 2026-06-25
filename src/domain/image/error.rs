use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageError {
    #[error("file not found")]
    NotFound,
    #[error("invalid image format")]
    InvalidFormat,
    #[error(transparent)]
    Unkown(#[from] anyhow::Error),
    #[error("EncodeFailed")]
    EncodeFailed,
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("file not found")]
    NotFound,
    #[error("file save failed")]
    SaveFailed,
    #[error(transparent)]
    Unkown(#[from] anyhow::Error),
}
