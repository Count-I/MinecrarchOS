use std::collections::HashMap;
use std::sync::Mutex;
use tokio::process::Command;
use zbus::{fdo, interface};

const VALID_COMPONENTS: &[&str] = &["shell", "modpack-manager", "overlay", "updater", "runtime"];
const VALID_LEVELS: &[&str] = &["debug", "info", "warn", "error"];

/// Maps the canonical level name to the systemd/journald log priority used in
/// `systemctl set-log-level` for the corresponding service unit.
fn level_to_journald_priority(level: &str) -> &'static str {
    match level {
        "debug" => "debug",
        "info" => "info",
        "warn" => "warning",
        "error" => "err",
        _ => "info",
    }
}

/// Maps a component name to its systemd user service unit.
fn component_to_unit(component: &str) -> Option<&'static str> {
    match component {
        "shell" => Some("minecrarch-shell.service"),
        "modpack-manager" => Some("minecrarch-modpack-manager.service"),
        "overlay" => Some("minecrarch-overlay.service"),
        "updater" => Some("minecrarch-updater.service"),
        "runtime" => Some("minecrarch-runtime.service"),
        _ => None,
    }
}

pub struct LoggingService {
    levels: Mutex<HashMap<String, String>>,
}

impl LoggingService {
    pub fn new() -> Self {
        let mut defaults = HashMap::new();
        for &component in VALID_COMPONENTS {
            defaults.insert(component.to_owned(), "info".to_owned());
        }
        Self {
            levels: Mutex::new(defaults),
        }
    }
}

#[interface(name = "org.minecrarch.Logging")]
impl LoggingService {
    async fn set_log_level(&self, component: String, level: String) -> fdo::Result<()> {
        if !VALID_COMPONENTS.contains(&component.as_str()) {
            return Err(fdo::Error::Failed(
                "org.minecrarch.Error.InvalidComponent".into(),
            ));
        }
        if !VALID_LEVELS.contains(&level.as_str()) {
            return Err(fdo::Error::Failed(
                "org.minecrarch.Error.InvalidLevel".into(),
            ));
        }

        {
            let mut map = self.levels.lock().unwrap();
            map.insert(component.clone(), level.clone());
        }

        // Best-effort: set runtime log level on the systemd unit. Non-fatal if the
        // unit is not running — the level is stored and applied next time it starts.
        if let Some(unit) = component_to_unit(&component) {
            let priority = level_to_journald_priority(&level);
            let result = Command::new("systemctl")
                .args(["--user", "service-log-level", unit, priority])
                .status()
                .await;
            match result {
                Ok(s) if s.success() => {
                    tracing::info!(component, level, "runtime log level updated");
                }
                Ok(_) => {
                    tracing::debug!(
                        component,
                        level,
                        "unit not running; level stored for next start"
                    );
                }
                Err(e) => {
                    tracing::warn!(error = %e, component, "systemctl unavailable; level stored");
                }
            }
        }

        Ok(())
    }

    async fn get_log_level(&self, component: String) -> fdo::Result<String> {
        if !VALID_COMPONENTS.contains(&component.as_str()) {
            return Err(fdo::Error::Failed(
                "org.minecrarch.Error.InvalidComponent".into(),
            ));
        }
        let map = self.levels.lock().unwrap();
        Ok(map
            .get(&component)
            .cloned()
            .unwrap_or_else(|| "info".to_owned()))
    }

    /// Returns the journald cursor at the last GAME_CRASHED event for the given
    /// instance. Returns "" if no crash has been recorded.
    async fn get_last_crash_cursor(&self, instance_id: String) -> fdo::Result<String> {
        // journalctl exits 0 with no output when no entries match, and exits 1 on error.
        let output = Command::new("journalctl")
            .args([
                "--user",
                "--output=export",
                "--reverse",
                "--lines=1",
                "MINECRARCH_EVENT=GAME_CRASHED",
                &format!("MINECRARCH_INSTANCE={instance_id}"),
            ])
            .output()
            .await
            .map_err(|e| fdo::Error::Failed(format!("journalctl error: {e}")))?;

        if !output.status.success() || output.stdout.is_empty() {
            return Ok(String::new());
        }

        // The export format includes "__CURSOR=<cursor>" on one of the lines.
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if let Some(cursor) = line.strip_prefix("__CURSOR=") {
                return Ok(cursor.to_owned());
            }
        }

        Ok(String::new())
    }
}
