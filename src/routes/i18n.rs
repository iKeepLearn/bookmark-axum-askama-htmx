use crate::utils::i18n::pick_supported;
use axum::http::header;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;
use http::HeaderMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct I18nForm {
    lang: String,
    redirect: Option<String>,
}

pub async fn set_lang(headers: HeaderMap, Form(form): Form<I18nForm>) -> Response {
    let back = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/")
        .to_string();

    if let Some(supported) = pick_supported(&form.lang) {
        let cookie = format!("lang={supported}; Path=/; Max-Age=31536000; SameSite=Lax");
        let mut response = Redirect::to(&form.redirect.unwrap_or(back)).into_response();
        response
            .headers_mut()
            .insert(header::SET_COOKIE, cookie.parse().unwrap());
        response
    } else {
        Redirect::to(&back).into_response()
    }
}
