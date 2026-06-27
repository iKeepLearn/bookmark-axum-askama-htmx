use askama::Template;
use axum::response::{Html, IntoResponse, Response};
use chrono::{DateTime, Utc};
use chrono_tz::Hongkong;
use serde::Serialize;

const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Serialize, Template)]
#[template(path = "pages/500.html")]
pub struct ErrorMessage {
    pub error_msg: String,
}

#[derive(Debug, Serialize, Template)]
#[template(path = "pages/404.html")]
pub struct NotFoundMessage {
    pub msg: String,
}

pub fn e500(error_msg: impl Into<String>) -> Response {
    Html(
        ErrorMessage {
            error_msg: error_msg.into(),
        }
        .render()
        .unwrap(),
    )
    .into_response()
}

pub fn e404(msg: impl Into<String>) -> Response {
    Html(NotFoundMessage { msg: msg.into() }.render().unwrap()).into_response()
}

pub fn render_template<T: Template>(template: T) -> Response {
    template
        .render()
        .map(axum::response::Html)
        .map_err(|e| e500(e.to_string()))
        .into_response()
}

pub fn format_date_time(date_time: &DateTime<Utc>) -> String {
    date_time
        .with_timezone(&Hongkong)
        .format(FORMAT)
        .to_string()
}

pub fn now_timestamp() -> i64 {
    Utc::now().timestamp()
}
