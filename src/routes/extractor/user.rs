use crate::routes::SessionUser;
use axum::{extract::FromRequestParts, http::request::Parts};

impl<S> FromRequestParts<S> for SessionUser
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 从 request extensions 里取
        parts
            .extensions
            .get::<SessionUser>()
            .cloned()
            .ok_or_else(|| {
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    "Missing session user".to_string(),
                )
            })
    }
}
