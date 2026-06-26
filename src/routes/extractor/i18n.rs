use crate::utils::i18n::detect_lang;
use axum::extract::FromRequestParts;
use fluent_templates::LanguageIdentifier;
use http::request::Parts;

pub struct Locale {
    pub lang: LanguageIdentifier,
}

impl<S> FromRequestParts<S> for Locale
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let lang = detect_lang(parts);

        Ok(Locale { lang })
    }
}
