use anyhow::Context;
use bookmark::app::config::get_configurations;
use bookmark::app::server::create_app;
use bookmark::shutdown_signal;
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
    let config = get_configurations().expect("failed to read configuration");
    let host = config.application.host.clone();
    let port = config.application.port;
    let app = create_app(config).await;
    let address = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&address).await?;
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("failed to run server")?;

    Ok(())
}
