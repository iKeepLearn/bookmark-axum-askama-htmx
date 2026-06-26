use crate::utils::i18n::{CURRENT_LANG, detect_lang};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn locale_middleware(req: Request, next: Next) -> Response {
    let (parts, body) = req.into_parts();
    let lang = detect_lang(&parts);
    CURRENT_LANG
        .scope(lang, next.run(Request::from_parts(parts, body)))
        .await
}
