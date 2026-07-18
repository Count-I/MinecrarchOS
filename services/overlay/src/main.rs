mod iface;
mod window;

use std::sync::{Arc, Mutex};

use anyhow::Context;
use iface::{OverlayIface, OverlayStatus};
use libadwaita as adw;
use libadwaita::prelude::*;
use tracing_subscriber::EnvFilter;
use zbus::{connection::Builder, SignalContext};

const OBJECT_PATH: &str = "/org/minecrarch/Overlay";

/// Commands sent from the D-Bus thread to the GTK main thread.
#[derive(Debug)]
pub enum OverlayCmd {
    ShowNotification {
        text: String,
        duration_ms: u32,
        level: String,
    },
    ShowCrashOverlay {
        reason: String,
        instance_id: String,
    },
    ShowSystemMenu,
    HideAll,
}

/// Events sent from the GTK main thread to the D-Bus thread.
#[derive(Debug)]
pub enum OverlayEvent {
    SystemMenuAction { action: String },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let display_available =
        std::env::var("WAYLAND_DISPLAY").is_ok() || std::env::var("DISPLAY").is_ok();

    if display_available {
        run_with_display()
    } else {
        tracing::warn!("no display available — starting in no-op mode");
        run_noop()
    }
}

/// No Wayland display: register the D-Bus service and serve all methods as no-ops.
fn run_noop() -> anyhow::Result<()> {
    let status = Arc::new(Mutex::new(OverlayStatus::default()));
    let service = OverlayIface::new(None, status);

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let _conn = Builder::session()
                .context("D-Bus session bus unavailable")?
                .name("org.minecrarch.Overlay")
                .context("failed to request bus name")?
                .serve_at(OBJECT_PATH, service)
                .context("failed to register object")?
                .build()
                .await
                .context("connection build failed")?;

            tracing::info!("Overlay service running in no-op mode");
            std::future::pending::<()>().await;
            Ok::<(), anyhow::Error>(())
        })
}

/// Wayland display available: run as a GTK4 app with a layer shell surface.
fn run_with_display() -> anyhow::Result<()> {
    let (cmd_tx, cmd_rx) = async_channel::bounded::<OverlayCmd>(16);
    let (event_tx, event_rx) = async_channel::bounded::<OverlayEvent>(16);
    let status = Arc::new(Mutex::new(OverlayStatus::default()));

    // Spawn the D-Bus bridge on a background thread with its own tokio runtime.
    let status_dbus = status.clone();
    std::thread::Builder::new()
        .name("overlay-dbus".into())
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime")
                .block_on(run_dbus(cmd_tx, event_rx, status_dbus));
        })
        .context("D-Bus thread spawn failed")?;

    let app = adw::Application::builder()
        .application_id("org.minecrarch.Overlay")
        .build();

    let status_gtk = status.clone();
    app.connect_activate(move |app| {
        window::build(app, cmd_rx.clone(), event_tx.clone(), status_gtk.clone());
    });

    let code = app.run();
    if code != 0.into() {
        anyhow::bail!("overlay application exited with code {}", i32::from(code));
    }
    Ok(())
}

/// D-Bus bridge: register the service and forward events as D-Bus signals.
async fn run_dbus(
    cmd_tx: async_channel::Sender<OverlayCmd>,
    event_rx: async_channel::Receiver<OverlayEvent>,
    status: Arc<Mutex<OverlayStatus>>,
) {
    let service = OverlayIface::new(Some(cmd_tx), status);

    let conn = match Builder::session()
        .and_then(|b| b.name("org.minecrarch.Overlay"))
        .and_then(|b| b.serve_at(OBJECT_PATH, service))
    {
        Ok(builder) => match builder.build().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(error = %e, "overlay D-Bus connection failed");
                return;
            }
        },
        Err(e) => {
            tracing::error!(error = %e, "overlay D-Bus setup failed");
            return;
        }
    };

    tracing::info!("Overlay D-Bus service registered");

    while let Ok(event) = event_rx.recv().await {
        let ctxt = match SignalContext::new(&conn, OBJECT_PATH) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "SignalContext failed");
                continue;
            }
        };
        match event {
            OverlayEvent::SystemMenuAction { action } => {
                tracing::info!(%action, "emitting SystemMenuAction");
                OverlayIface::system_menu_action(&ctxt, &action).await.ok();
            }
        }
    }
}
