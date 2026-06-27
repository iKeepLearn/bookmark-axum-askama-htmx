use crate::utils::i18n::{CURRENT_LANG, LOCALES, default_lang};
use fluent_templates::Loader;
use std::fmt::Display;

#[askama::filter_fn]
pub fn t(key: &str, _env: &dyn askama::Values) -> ::askama::Result<String> {
    let lang = CURRENT_LANG
        .try_with(|l| l.clone())
        .unwrap_or_else(|_| default_lang());
    Ok(LOCALES.lookup(&lang, key))
}

#[askama::filter_fn]
pub fn is_current_lang(value: impl Display, _env: &dyn askama::Values) -> ::askama::Result<bool> {
    let lang = CURRENT_LANG
        .try_with(|l| l.clone())
        .unwrap_or_else(|_| default_lang());

    let candidate = value.to_string();
    Ok(lang.to_string() == candidate || lang.language.as_str().eq_ignore_ascii_case(&candidate))
}
