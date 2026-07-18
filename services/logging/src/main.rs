mod logger;

use anyhow::Context;
use logger::LoggingService;
use tracing_subscriber::EnvFilter;
use zbus::connection::Builder;

const OBJECT_PATH: &str = "/org/minecrarch/Logging";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let service = LoggingService::new();

    let _conn = Builder::session()
        .context("D-Bus session bus unavailable")?
        .name("org.minecrarch.Logging")
        .context("failed to request bus name")?
        .serve_at(OBJECT_PATH, service)
        .context("failed to register object")?
        .build()
        .await
        .context("connection build failed")?;

    tracing::info!("Logging service running on session bus");

    // Block the main task indefinitely; the zbus executor drives the service.
    std::future::pending::<()>().await;

    Ok(())
}
