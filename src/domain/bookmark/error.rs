use thiserror::Error;

#[derive(Error, Debug)]
pub enum BookmarkError {
    #[error("{0} not found")]
    NotFound(String),
    #[error("{0} already exists")]
    Exists(String),
    #[error(transparent)]
    Unkown(#[from] anyhow::Error),
}
