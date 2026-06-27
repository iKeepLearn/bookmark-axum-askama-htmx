use fluent_templates::{LanguageIdentifier, Loader, static_loader};
use http::request::Parts;
use http::{HeaderMap, header};
use std::sync::LazyLock;

static_loader! {
    pub static LOCALES = {
        locales: "./locales",
        fallback_language: "zh-CN",
    };
}

static SUPPORTED: LazyLock<Vec<LanguageIdentifier>> =
    LazyLock::new(|| vec!["zh-CN".parse().unwrap(), "en-US".parse().unwrap()]);

pub fn default_lang() -> LanguageIdentifier {
    "zh-CN".parse().unwrap()
}

tokio::task_local! {
   pub static CURRENT_LANG: LanguageIdentifier;
}

pub fn t_for(lang: &LanguageIdentifier, key: &str) -> String {
    LOCALES.lookup(lang, key)
}

pub fn pick_supported(candidate: &str) -> Option<LanguageIdentifier> {
    let id: LanguageIdentifier = candidate.parse().ok()?;
    SUPPORTED.iter().find(|s| **s == id).cloned()
}

pub fn detect_lang(parts: &Parts) -> LanguageIdentifier {
    if let Some(q) = parts.uri.query() {
        if let Some(lang) = parse_query_lang(q) {
            return lang;
        }
    }

    if let Some(lang) = parse_cookie_lang(&parts.headers) {
        return lang;
    }

    if let Some(lang) = parse_accept_language(&parts.headers) {
        return lang;
    }

    default_lang()
}

fn parse_query_lang(query: &str) -> Option<LanguageIdentifier> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == "lang")
        .and_then(|(_, v)| pick_supported(v))
}

fn parse_cookie_lang(headers: &HeaderMap) -> Option<LanguageIdentifier> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .filter_map(|c| c.trim().split_once('='))
        .find(|(k, _)| *k == "lang")
        .and_then(|(_, v)| pick_supported(v))
}

fn parse_accept_language(headers: &HeaderMap) -> Option<LanguageIdentifier> {
    let raw = headers.get(header::ACCEPT_LANGUAGE)?.to_str().ok()?;
    // "zh-CN,zh;q=0.9,en-US;q=0.8" -> try each tag, in order, until one matches.
    raw.split(',')
        .filter_map(|part| part.split(';').next())
        .map(|tag| tag.trim())
        .find_map(pick_supported)
}

pub fn current_lang() -> String {
    let lang = CURRENT_LANG
        .try_with(|l| l.clone())
        .unwrap_or_else(|_| default_lang());

    lang.to_string()
}
