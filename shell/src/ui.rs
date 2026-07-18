use adw::prelude::*;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use crate::dbus;
use crate::session::{SessionState, ShellCommand, ShellEvent};

pub fn build(app: &adw::Application) {
    let (event_tx, event_rx) = async_channel::bounded::<ShellEvent>(32);
    let (cmd_tx, cmd_rx) = async_channel::bounded::<ShellCommand>(8);

    dbus::spawn(event_tx, cmd_rx);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .fullscreened(true)
        .build();

    let nav_view = adw::NavigationView::new();
    let state = Rc::new(RefCell::new(SessionState::Menu));

    let menu_page = build_main_menu(&state, cmd_tx);
    nav_view.push(&menu_page);

    window.set_content(Some(&nav_view));
    window.present();

    // Drive state transitions from D-Bus events on the GLib main loop.
    let nav_clone = nav_view.clone();
    let state_clone = state.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = event_rx.recv().await {
            handle_event(event, &nav_clone, &state_clone);
        }
    });
}

fn build_main_menu(
    state: &Rc<RefCell<SessionState>>,
    cmd_tx: async_channel::Sender<ShellCommand>,
) -> adw::NavigationPage {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    let title = gtk::Label::builder()
        .label("Minecrarch")
        .css_classes(vec!["title-1"])
        .build();

    let launch_btn = gtk::Button::builder()
        .label("Launch Game")
        .css_classes(vec!["suggested-action", "pill"])
        .width_request(240)
        .can_focus(true)
        .build();

    let state_for_click = state.clone();
    launch_btn.connect_clicked(move |_| {
        let current = state_for_click.borrow().clone();
        if current == SessionState::Menu {
            *state_for_click.borrow_mut() = SessionState::Launching {
                instance_id: "test-instance".to_owned(),
            };
            tracing::info!("requesting game launch");
            if let Err(e) = cmd_tx.try_send(ShellCommand::LaunchInstance {
                id: "test-instance".to_owned(),
            }) {
                tracing::warn!(error = %e, "command channel full");
            }
        }
    });

    content.append(&title);
    content.append(&launch_btn);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    toolbar.set_content(Some(&content));

    adw::NavigationPage::builder()
        .title("Minecrarch")
        .child(&toolbar)
        .build()
}

fn handle_event(
    event: ShellEvent,
    _nav_view: &adw::NavigationView,
    state: &Rc<RefCell<SessionState>>,
) {
    match event {
        ShellEvent::GameStarted { instance_id, pid } => {
            tracing::info!(%instance_id, pid, "game started — transitioning to IN_GAME");
            *state.borrow_mut() = SessionState::InGame { instance_id, pid };
        }
        ShellEvent::GameExited {
            instance_id,
            exit_code,
        } => {
            tracing::info!(%instance_id, exit_code, "game exited cleanly — returning to MENU");
            *state.borrow_mut() = SessionState::Menu;
        }
        ShellEvent::GameCrashed {
            instance_id,
            exit_code,
            signal,
        } => {
            tracing::warn!(%instance_id, exit_code, %signal, "game crashed — entering RECOVERING");
            *state.borrow_mut() = SessionState::Recovering {
                instance_id,
                exit_code,
                signal,
            };
            // D6 pushes the recovery UI page here.
        }
    }
}
