mod manager;

use anyhow::Context;
use manager::{GameEvent, ModpackManager, OBJECT_PATH};
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use zbus::connection::Builder;
use zbus::SignalContext;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let (tx, mut rx) = mpsc::channel::<GameEvent>(32);
    let mgr = ModpackManager::new(tx);

    let conn = Builder::session()
        .context("D-Bus session bus unavailable")?
        .name("org.minecrarch.ModpackManager")
        .context("failed to request bus name")?
        .serve_at(OBJECT_PATH, mgr)
        .context("failed to register object")?
        .build()
        .await
        .context("connection build failed")?;

    tracing::info!("ModpackManager running on session bus");

    while let Some(event) = rx.recv().await {
        let ctxt =
            SignalContext::new(&conn, OBJECT_PATH).context("SignalContext construction failed")?;

        match event {
            GameEvent::Started { instance_id, pid } => {
                tracing::info!(%instance_id, pid, "emitting GameStarted");
                ModpackManager::game_started(&ctxt, &instance_id, pid)
                    .await
                    .ok();
            }
            GameEvent::Exited {
                instance_id,
                exit_code,
            } => {
                tracing::info!(%instance_id, exit_code, "emitting GameExited");
                ModpackManager::game_exited(&ctxt, &instance_id, exit_code)
                    .await
                    .ok();
            }
            GameEvent::Crashed {
                instance_id,
                exit_code,
                signal,
            } => {
                tracing::warn!(%instance_id, exit_code, %signal, "emitting GameCrashed");
                ModpackManager::game_crashed(&ctxt, &instance_id, exit_code, &signal)
                    .await
                    .ok();
            }
        }
    }

    Ok(())
}
