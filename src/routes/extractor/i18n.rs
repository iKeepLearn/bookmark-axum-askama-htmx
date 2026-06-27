use crate::utils::i18n::default_lang;
use axum::extract::FromRequestParts;
use fluent_templates::LanguageIdentifier;
use http::request::Parts;

#[derive(Debug, Clone)]
pub struct Locale {
    pub lang: LanguageIdentifier,
}

impl<S> FromRequestParts<S> for Locale
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let locale = parts
            .extensions
            .get::<Locale>()
            .cloned()
            .unwrap_or_else(|| Locale {
                lang: default_lang(),
            });
        Ok(locale)
    }
}
