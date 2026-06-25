use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("User already exists")]
    Exists,
    #[error("Invalid password")]
    InvalidPassword,
    #[error(transparent)]
    Unkown(#[from] anyhow::Error),
}
