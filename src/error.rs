use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Internal error:{0}")]
    Internal(String),
    #[error("Not Found:{0}")]
    NotFound(String),
    #[error("{0}")]
    InvalidArgument(String),
    #[error("InvalidAuth")]
    InvalidAuth,
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    InvalidPermission(String),
    #[error("{0}")]
    Validation(String),
    #[error("Need more fields")]
    ValidationFields(HashMap<String, Vec<String>>),
}

impl Error {
    pub fn internal(str: &str) -> Self {
        Error::Internal(str.to_string())
    }

    pub fn bad_request(str: &str) -> Self {
        Error::BadRequest(str.to_string())
    }

    pub fn invalid_arg(str: &str) -> Self {
        Error::InvalidArgument(str.to_string())
    }

    pub fn not_found(str: &str) -> Self {
        Error::NotFound(str.to_string())
    }

    pub fn invalid_permission(str: &str) -> Self {
        Error::InvalidPermission(str.to_string())
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Error::NotFound("row not exists".into()),
            _ => Error::Internal("Sqlx Error".into()),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Error::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message.to_string()),
            Error::InvalidArgument(message) => (StatusCode::BAD_REQUEST, message.to_string()),
            Error::NotFound(message) => (StatusCode::NOT_FOUND, message.to_string()),
            Error::InvalidAuth => (StatusCode::UNAUTHORIZED, self.to_string()),
            Error::BadRequest(message) => (StatusCode::BAD_REQUEST, message.to_string()),
            Error::InvalidPermission(message) => (StatusCode::FORBIDDEN, message.to_string()),
            Error::Validation(message) => (StatusCode::BAD_REQUEST, message.to_string()),
            Error::ValidationFields(_) => (StatusCode::BAD_REQUEST, "Need more fields".to_string()),
        };

        let body = serde_json::json!({
            "status_code":status.as_u16(),
            "timestamp": Utc::now(),
            "message": message,
        });
        (status, Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Internal(format!("Io Error:{:?}", value.to_string()))
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Internal(value.to_string())
    }
}
