use crate::routes::user::SessionUser;
use axum::{
    extract::Request,
    middleware::Next,
    response::{Redirect, Response},
};
use tower_sessions::Session;

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, Redirect> {
    let session = req.extensions().get::<Session>().cloned();
    let user: Option<SessionUser> = match session {
        Some(session) => session.get("user").await.ok().flatten(),
        None => None,
    };

    match user {
        Some(user) => {
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        }
        None => Err(Redirect::to("/login")),
    }
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
