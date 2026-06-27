mod bookmark;
mod home;
mod i18n;
mod image;
mod user;

pub mod extractor;
pub mod middleware;

use self::bookmark::bookmark_import;
use self::i18n::set_lang;
use self::user::{get_token_submit, token_form};
use crate::app::state::AppState;
use crate::utils::render_template;
use askama::Template;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Router, middleware as axum_middleware};
pub use bookmark::{
    add_bookmark, add_bookmark_form, bookmark_import_page, delete_bookmark, edit_bookmark,
    get_edit_form,
};
pub use home::get_home;
pub use image::{get_image, upload_image};
use middleware::auth::{admin_middleware, auth_middleware};
pub use user::{SessionUser, changepwd_form, changepwd_submit, login_form, login_submit, logout};

pub enum PageResult<T: Template, S: Into<String>> {
    RenderTemplate(T),
    Redirect(S),
}

impl<T: Template, S: Into<String>> IntoResponse for PageResult<T, S> {
    fn into_response(self) -> axum::response::Response {
        match self {
            PageResult::RenderTemplate(t) => render_template(t),
            PageResult::Redirect(url) => Redirect::to(&url.into()).into_response(),
        }
    }
}

pub fn public_routes() -> axum::Router<AppState> {
    Router::new()
        .route("/token", get(token_form).post(get_token_submit))
        .route("/login", get(login_form).post(login_submit))
        .route("/i18n/switch", post(set_lang))
}

pub fn protected_routes() -> axum::Router<AppState> {
    let manage_routes = Router::new().route("/logout", post(logout));

    let admin_routes = Router::new()
        .route("/upload", post(upload_image))
        .route("/bookmark", get(add_bookmark_form).post(add_bookmark))
        .route("/bookmark/delete", post(delete_bookmark))
        .route("/bookmark/edit", get(get_edit_form).post(edit_bookmark))
        .route("/changepwd", get(changepwd_form).post(changepwd_submit))
        .route(
            "/bookmark/import",
            get(bookmark_import_page).post(bookmark_import),
        )
        .layer(axum_middleware::from_fn(admin_middleware));

    Router::new()
        .route("/", get(get_home))
        .route("/images/{id}", get(get_image))
        .nest("/manage", manage_routes)
        .nest("/manage", admin_routes)
        .layer(axum_middleware::from_fn(auth_middleware))
}
