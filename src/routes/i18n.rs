use crate::utils::i18n::pick_supported;
use axum::extract::Path;
use axum::http::header;
use axum::response::{IntoResponse, Redirect, Response};
use http::HeaderMap;

pub async fn set_lang(Path(lang): Path<String>, headers: HeaderMap) -> Response {
    let back = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/")
        .to_string();

    if let Some(supported) = pick_supported(&lang) {
        let cookie = format!("lang={supported}; Path=/; Max-Age=31536000; SameSite=Lax");
        let mut response = Redirect::to(&back).into_response();
        response
            .headers_mut()
            .insert(header::SET_COOKIE, cookie.parse().unwrap());
        response
    } else {
        Redirect::to(&back).into_response()
    }
}
