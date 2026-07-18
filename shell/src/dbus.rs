use async_channel::{Receiver, Sender};
use futures_util::StreamExt;

use crate::session::{ShellCommand, ShellEvent};

/// D-Bus proxy for org.minecrarch.ModpackManager.
/// Interface contract defined in docs/ipc.md.
#[zbus::proxy(
    interface = "org.minecrarch.ModpackManager",
    default_service = "org.minecrarch.ModpackManager",
    default_path = "/org/minecrarch/ModpackManager"
)]
trait ModpackManager {
    async fn launch_instance(&self, id: &str) -> zbus::Result<()>;
    async fn stop_instance(&self, id: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn game_started(&self, instance_id: &str, pid: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn game_exited(&self, instance_id: &str, exit_code: i32) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn game_crashed(
        &self,
        instance_id: &str,
        exit_code: i32,
        signal_name: &str,
    ) -> zbus::Result<()>;
}

/// Spawns the D-Bus background thread with a single-threaded tokio runtime.
/// The thread owns the zbus connection and bridges signals to the GTK main thread
/// via `event_tx`, and receives UI commands via `cmd_rx`.
pub fn spawn(event_tx: Sender<ShellEvent>, cmd_rx: Receiver<ShellCommand>) {
    std::thread::Builder::new()
        .name("dbus".into())
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime")
                .block_on(run(event_tx, cmd_rx));
        })
        .expect("D-Bus thread spawn failed");
}

async fn run(event_tx: Sender<ShellEvent>, cmd_rx: Receiver<ShellCommand>) {
    let conn = match zbus::Connection::session().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "D-Bus session bus unavailable — shell will not receive game events");
            return;
        }
    };

    let proxy = ModpackManagerProxy::new(&conn)
        .await
        .expect("proxy construction is infallible");

    let Ok(mut started) = proxy.receive_game_started().await else {
        tracing::error!("failed to subscribe to GameStarted signal");
        return;
    };
    let Ok(mut exited) = proxy.receive_game_exited().await else {
        tracing::error!("failed to subscribe to GameExited signal");
        return;
    };
    let Ok(mut crashed) = proxy.receive_game_crashed().await else {
        tracing::error!("failed to subscribe to GameCrashed signal");
        return;
    };

    tracing::info!("D-Bus ready, subscribed to ModpackManager signals");

    loop {
        tokio::select! {
            Ok(cmd) = cmd_rx.recv() => {
                handle_command(&proxy, cmd).await;
            }
            Some(sig) = started.next() => {
                if let Ok(args) = sig.args() {
                    let _ = event_tx.send(ShellEvent::GameStarted {
                        instance_id: args.instance_id.to_owned(),
                        pid: args.pid,
                    }).await;
                }
            }
            Some(sig) = exited.next() => {
                if let Ok(args) = sig.args() {
                    let _ = event_tx.send(ShellEvent::GameExited {
                        instance_id: args.instance_id.to_owned(),
                        exit_code: args.exit_code,
                    }).await;
                }
            }
            Some(sig) = crashed.next() => {
                if let Ok(args) = sig.args() {
                    let _ = event_tx.send(ShellEvent::GameCrashed {
                        instance_id: args.instance_id.to_owned(),
                        exit_code: args.exit_code,
                        signal: args.signal_name.to_owned(),
                    }).await;
                }
            }
            else => {
                tracing::warn!("D-Bus event loop ended (all channels closed)");
                break;
            }
        }
    }
}

async fn handle_command(proxy: &ModpackManagerProxy<'_>, cmd: ShellCommand) {
    match cmd {
        ShellCommand::LaunchInstance { id } => {
            tracing::info!(%id, "sending LaunchInstance");
            if let Err(e) = proxy.launch_instance(&id).await {
                tracing::warn!(error = %e, "LaunchInstance failed (service may not be running yet)");
            }
        }
        ShellCommand::StopInstance { id } => {
            tracing::info!(%id, "sending StopInstance");
            if let Err(e) = proxy.stop_instance(&id).await {
                tracing::warn!(error = %e, "StopInstance failed");
            }
        }
    }
}
