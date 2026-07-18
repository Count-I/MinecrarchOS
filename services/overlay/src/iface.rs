use std::sync::{Arc, Mutex};
use zbus::{fdo, interface, SignalContext};

use crate::OverlayCmd;

/// Shared state readable by both the D-Bus thread and the GTK main thread.
#[derive(Default)]
pub struct OverlayStatus {
    pub visible: bool,
    pub has_input_focus: bool,
}

pub struct OverlayIface {
    /// None when running in no-op mode (no Wayland display).
    cmd_tx: Option<async_channel::Sender<OverlayCmd>>,
    pub status: Arc<Mutex<OverlayStatus>>,
}

impl OverlayIface {
    pub fn new(
        cmd_tx: Option<async_channel::Sender<OverlayCmd>>,
        status: Arc<Mutex<OverlayStatus>>,
    ) -> Self {
        Self { cmd_tx, status }
    }

    fn send_cmd(&self, cmd: OverlayCmd) {
        if let Some(tx) = &self.cmd_tx {
            if let Err(e) = tx.try_send(cmd) {
                tracing::warn!(error = %e, "overlay command channel full or closed");
            }
        }
    }
}

#[interface(name = "org.minecrarch.Overlay")]
impl OverlayIface {
    async fn show_notification(
        &self,
        text: String,
        duration_ms: u32,
        level: String,
    ) -> fdo::Result<()> {
        tracing::info!(%text, duration_ms, %level, "ShowNotification");
        self.send_cmd(OverlayCmd::ShowNotification {
            text,
            duration_ms,
            level,
        });
        Ok(())
    }

    async fn show_crash_overlay(&self, reason: String, instance_id: String) -> fdo::Result<()> {
        tracing::info!(%reason, %instance_id, "ShowCrashOverlay");
        self.send_cmd(OverlayCmd::ShowCrashOverlay {
            reason,
            instance_id,
        });
        Ok(())
    }

    async fn show_system_menu(&self) -> fdo::Result<()> {
        tracing::info!("ShowSystemMenu");
        self.send_cmd(OverlayCmd::ShowSystemMenu);
        Ok(())
    }

    async fn hide_all(&self) -> fdo::Result<()> {
        tracing::info!("HideAll");
        self.send_cmd(OverlayCmd::HideAll);
        Ok(())
    }

    #[zbus(property)]
    async fn visible(&self) -> bool {
        self.status.lock().unwrap().visible
    }

    #[zbus(property)]
    async fn has_input_focus(&self) -> bool {
        self.status.lock().unwrap().has_input_focus
    }

    #[zbus(signal)]
    pub async fn system_menu_action(ctxt: &SignalContext<'_>, action: &str) -> zbus::Result<()>;
}
