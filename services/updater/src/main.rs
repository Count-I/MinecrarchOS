mod updater;

use anyhow::Context;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use updater::{Updater, UpdaterEvent, OBJECT_PATH};
use zbus::connection::Builder;
use zbus::SignalContext;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let (tx, mut rx) = mpsc::channel::<UpdaterEvent>(32);
    let svc = Updater::new(tx);

    let conn = Builder::session()
        .context("D-Bus session bus unavailable")?
        .name("org.minecrarch.Updater")
        .context("failed to request bus name")?
        .serve_at(OBJECT_PATH, svc)
        .context("failed to register object")?
        .build()
        .await
        .context("connection build failed")?;

    tracing::info!("Updater service running on session bus");

    while let Some(event) = rx.recv().await {
        let ctxt =
            SignalContext::new(&conn, OBJECT_PATH).context("SignalContext construction failed")?;

        match event {
            UpdaterEvent::Progress {
                stage,
                percent,
                message,
            } => {
                tracing::info!(%stage, percent, %message, "emitting UpdateProgress");
                Updater::update_progress(&ctxt, &stage, percent, &message)
                    .await
                    .ok();
            }
            UpdaterEvent::Complete { new_version } => {
                tracing::info!(%new_version, "emitting UpdateComplete");
                Updater::update_complete(&ctxt, &new_version).await.ok();
            }
            UpdaterEvent::Failed { error } => {
                tracing::error!(%error, "emitting UpdateFailed");
                Updater::update_failed(&ctxt, &error).await.ok();
            }
        }
    }

    Ok(())
}
