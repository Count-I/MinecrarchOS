use std::sync::{Arc, Mutex};
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use libadwaita as adw;

use crate::iface::OverlayStatus;
use crate::{OverlayCmd, OverlayEvent};

pub fn build(
    app: &adw::Application,
    cmd_rx: async_channel::Receiver<OverlayCmd>,
    event_tx: async_channel::Sender<OverlayEvent>,
    status: Arc<Mutex<OverlayStatus>>,
) {
    // Layer shell windows use gtk4::Window directly — ApplicationWindow is not needed.
    let window = gtk4::Window::builder()
        .application(app)
        .name("minecrarch-overlay")
        .build();

    // Configure wlr-layer-shell: OVERLAY layer, all edges anchored, full-screen.
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 0);
    window.set_margin(Edge::Bottom, 0);
    window.set_margin(Edge::Left, 0);
    window.set_margin(Edge::Right, 0);
    // Start with no keyboard interaction — only capture input during system menu.
    window.set_keyboard_mode(KeyboardMode::None);

    // Root overlay: a GtkFixed so individual widgets can be positioned freely.
    let root = gtk4::Fixed::new();
    root.set_size_request(1, 1); // minimal size when nothing is shown

    // Notification bar: shown at the top of the screen.
    let notification_label = gtk4::Label::builder()
        .wrap(true)
        .halign(gtk4::Align::Center)
        .css_classes(vec!["overlay-notification"])
        .build();
    let notification_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(vec!["overlay-notification-box"])
        .visible(false)
        .build();
    notification_box.append(&notification_label);
    root.put(&notification_box, 0.0, 0.0);

    // System menu: a vertical box of action buttons centered on screen.
    let system_menu = build_system_menu(&event_tx);
    system_menu.set_visible(false);
    root.put(&system_menu, 0.0, 0.0);

    window.set_child(Some(&root));

    let hide_all = {
        let notification_box = notification_box.clone();
        let system_menu = system_menu.clone();
        let window = window.clone();
        let status = status.clone();
        move || {
            notification_box.set_visible(false);
            system_menu.set_visible(false);
            window.set_keyboard_mode(KeyboardMode::None);
            let mut s = status.lock().unwrap();
            s.visible = false;
            s.has_input_focus = false;
        }
    };

    // Drive overlay commands from the D-Bus thread.
    let window_clone = window.clone();
    let status_clone = status.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(cmd) = cmd_rx.recv().await {
            match cmd {
                OverlayCmd::ShowNotification {
                    text,
                    duration_ms,
                    level: _,
                } => {
                    notification_label.set_text(&text);
                    notification_box.set_visible(true);
                    {
                        let mut s = status_clone.lock().unwrap();
                        s.visible = true;
                    }
                    window_clone.set_visible(true);

                    // Auto-dismiss after duration_ms.
                    let nb = notification_box.clone();
                    let st = status_clone.clone();
                    let wc = window_clone.clone();
                    glib::timeout_add_local_once(
                        Duration::from_millis(duration_ms as u64),
                        move || {
                            nb.set_visible(false);
                            let mut s = st.lock().unwrap();
                            s.visible = false;
                            drop(s);
                            wc.set_visible(false);
                        },
                    );
                }

                OverlayCmd::ShowCrashOverlay {
                    reason,
                    instance_id,
                } => {
                    let text = format!("Game crashed ({}): {}", instance_id, reason);
                    notification_label.set_text(&text);
                    notification_box.set_visible(true);
                    window_clone.set_visible(true);
                    {
                        let mut s = status_clone.lock().unwrap();
                        s.visible = true;
                    }
                    // Crash overlay is persistent — the shell calls HideAll when it takes over.
                }

                OverlayCmd::ShowSystemMenu => {
                    system_menu.set_visible(true);
                    window_clone.set_keyboard_mode(KeyboardMode::OnDemand);
                    window_clone.set_visible(true);
                    {
                        let mut s = status_clone.lock().unwrap();
                        s.visible = true;
                        s.has_input_focus = true;
                    }
                }

                OverlayCmd::HideAll => hide_all(),
            }
        }
    });

    window.present();
}

fn build_system_menu(event_tx: &async_channel::Sender<OverlayEvent>) -> gtk4::Box {
    let menu = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .css_classes(vec!["overlay-system-menu"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    let actions = [
        ("Return to Game", "return"),
        ("Quit Game", "quit_game"),
        ("Suspend", "suspend"),
        ("Shut Down", "shutdown"),
    ];

    for (label, action_id) in actions {
        let btn = gtk4::Button::builder()
            .label(label)
            .css_classes(vec!["pill"])
            .width_request(200)
            .build();
        let tx = event_tx.clone();
        let action = action_id.to_owned();
        btn.connect_clicked(move |_| {
            if let Err(e) = tx.try_send(OverlayEvent::SystemMenuAction {
                action: action.clone(),
            }) {
                tracing::warn!(error = %e, "event channel full on system menu action");
            }
        });
        menu.append(&btn);
    }

    menu
}
