mod dbus;
mod session;
mod ui;

use adw::prelude::*;
use libadwaita as adw;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let app = adw::Application::builder()
        .application_id("org.minecrarch.Shell")
        .build();

    app.connect_activate(ui::build);

    std::process::exit(app.run().into());
}
