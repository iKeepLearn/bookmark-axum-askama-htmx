use crate::{
    Error, app::state::AppState, domain::user::traits::AuthProvider, routes::user::SessionUser,
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{Redirect, Response},
};
use http::header;
use tower_sessions::Session;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, Redirect> {
    if let Some(session) = req.extensions().get::<Session>().cloned() {
        if let Ok(Some(user)) = session.get::<SessionUser>("user").await {
            req.extensions_mut().insert(user);
            return Ok(next.run(req).await);
        }
    }

    let token = match extract_bearer_token(&req) {
        Some(token) => token,
        None => return Err(Redirect::to("/login")),
    };

    let claims = match state.auth_provider.verify_token(token) {
        Ok(c) => c,
        Err(_) => return Err(Redirect::to("/login")),
    };

    let user = match state
        .user_service
        .get_user_by_username(&claims.username)
        .await
    {
        Ok(u) => u,
        Err(_) => return Err(Redirect::to("/login")),
    };

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

pub async fn admin_middleware(req: Request, next: Next) -> Result<Response, Redirect> {
    let user = req.extensions().get::<SessionUser>();
    match user {
        Some(user) => {
            if user.is_admin {
                Ok(next.run(req).await)
            } else {
                Err(Redirect::to("/login"))
            }
        }
        None => Err(Redirect::to("/login")),
    }
}

pub async fn api_auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, Error> {
    let token = extract_bearer_token(&req).ok_or(Error::InvalidAuth)?;
    let claims = state.auth_provider.verify_token(token)?;
    let user = state
        .user_service
        .get_user_by_username(&claims.username)
        .await?;
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

fn extract_bearer_token(req: &Request) -> Option<&str> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}
