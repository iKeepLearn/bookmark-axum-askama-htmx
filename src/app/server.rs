use crate::routes::middleware::i18n::locale_middleware;
use crate::routes::{protected_routes, public_routes};
use crate::utils::e404;
use axum::{Router, middleware};
use http::{Response, StatusCode};
use std::time::Duration as StdDuration;
use time::Duration;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::{DefaultOnRequest, TraceLayer},
};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tracing::Level;

use super::config::Settings;
use super::state::AppState;

pub async fn create_app(config: Settings) -> Router {
    let app_state = AppState::new(config);
    let session_store = MemoryStore::default();
    let static_directory = app_state.static_directory.0.clone();
    let assets = ServeDir::new(static_directory);

    let secure: bool = env!("APP_SECURE_COOKIES")
        .parse()
        .expect("APP_SECURE_COOKIES 应该是 build.rs 生成的合法 bool 字符串");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(secure)
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &http::Request<_>| {
            let request_id = request
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown");

            let matched_path = request
                .extensions()
                .get::<axum::extract::MatchedPath>()
                .map(|p| p.as_str().to_string())
                .unwrap_or_else(|| request.uri().path().to_string());

            tracing::info_span!(
                "http_request",
                method = %request.method(),
                path = %matched_path,
                request_id = %request_id,
            )
        })
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
            |response: &Response<_>, latency: StdDuration, span: &tracing::Span| {
                let status = response.status().as_u16();
                span.record("status", status);
                tracing::info!(
                    status = status,
                    latency = ?latency,
                    "finished processing request"
                );
            },
        );

    Router::new()
        .nest_service("/static", assets)
        .merge(public_routes())
        .merge(protected_routes(app_state.clone()))
        .fallback(|| async { e404("页面不存在") })
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(trace_layer)
                .layer(PropagateRequestIdLayer::x_request_id())
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    StdDuration::from_secs(10),
                ))
                .layer(CorsLayer::permissive())
                .layer(session_layer)
                .layer(middleware::from_fn(locale_middleware)),
        )
        .with_state(app_state)
}
