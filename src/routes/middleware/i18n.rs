use crate::routes::extractor::i18n::Locale;
use crate::utils::i18n::{CURRENT_LANG, detect_lang};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn locale_middleware(req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let lang = detect_lang(&parts);
    parts.extensions.insert(Locale { lang: lang.clone() });
    let req = Request::from_parts(parts, body);

    CURRENT_LANG.scope(lang, next.run(req)).await
}
