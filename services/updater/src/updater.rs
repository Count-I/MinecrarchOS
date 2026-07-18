use std::sync::{Arc, Mutex};
use tokio::process::Command;
use tokio::sync::mpsc;
use zbus::{fdo, interface, SignalContext};

pub const OBJECT_PATH: &str = "/org/minecrarch/Updater";

#[derive(Debug)]
pub enum UpdaterEvent {
    Progress {
        stage: String,
        percent: u32,
        message: String,
    },
    Complete {
        new_version: String,
    },
    Failed {
        error: String,
    },
}

#[derive(Debug, Default, Clone, PartialEq)]
enum State {
    #[default]
    Idle,
    Checking,
    Snapshotting,
    Downloading,
    Applying,
    RollingBack,
    Error,
}

impl State {
    fn as_str(&self) -> &'static str {
        match self {
            State::Idle => "idle",
            State::Checking => "checking",
            State::Snapshotting => "snapshotting",
            State::Downloading => "downloading",
            State::Applying => "applying",
            State::RollingBack => "rolling-back",
            State::Error => "error",
        }
    }
}

#[derive(Default)]
struct UpdaterInner {
    state: State,
    pending_version: String,
    pending_url: String,
    update_available: bool,
}

pub struct Updater {
    event_tx: mpsc::Sender<UpdaterEvent>,
    current_version: String,
    inner: Arc<Mutex<UpdaterInner>>,
}

impl Updater {
    pub fn new(event_tx: mpsc::Sender<UpdaterEvent>) -> Self {
        Self {
            event_tx,
            current_version: read_current_version(),
            inner: Arc::new(Mutex::new(UpdaterInner::default())),
        }
    }
}

// ── helpers ────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct UpdateManifest {
    version: String,
    description: String,
    #[serde(default)]
    url: String,
}

fn read_current_version() -> String {
    std::fs::read_to_string("/etc/minecrarch/version")
        .ok()
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "0.1.0".to_string())
}

fn snapshots_dir() -> std::path::PathBuf {
    std::env::var("MINECRARCH_SNAPSHOTS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/.snapshots"))
}

fn snapshot_name_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("pre-update-{secs}")
}

/// Parses the subvolume ID from `btrfs subvolume show` output.
fn parse_subvolume_id(output: &[u8]) -> Option<u64> {
    let text = std::str::from_utf8(output).ok()?;
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("Subvolume ID:")
            .and_then(|rest| rest.trim().parse::<u64>().ok())
    })
}

// ── D-Bus interface ────────────────────────────────────────────────────────

#[interface(name = "org.minecrarch.Updater")]
impl Updater {
    /// Fetches the update manifest from MINECRARCH_UPDATE_URL and reports
    /// whether a newer version is available. Returns (available, version, description).
    async fn check_for_updates(&self) -> fdo::Result<(bool, String, String)> {
        let url = std::env::var("MINECRARCH_UPDATE_URL")
            .map_err(|_| fdo::Error::Failed("MINECRARCH_UPDATE_URL is not set".into()))?;

        self.inner.lock().unwrap().state = State::Checking;

        let output = Command::new("curl")
            .args(["-fsSL", "--max-time", "30", &url])
            .output()
            .await
            .map_err(|e| {
                self.inner.lock().unwrap().state = State::Error;
                fdo::Error::Failed(format!("curl failed: {e}"))
            })?;

        if !output.status.success() {
            self.inner.lock().unwrap().state = State::Error;
            return Err(fdo::Error::Failed("failed to fetch update manifest".into()));
        }

        let manifest: UpdateManifest = serde_json::from_slice(&output.stdout).map_err(|e| {
            self.inner.lock().unwrap().state = State::Error;
            fdo::Error::Failed(format!("invalid manifest JSON: {e}"))
        })?;

        let available = manifest.version != self.current_version;
        {
            let mut inner = self.inner.lock().unwrap();
            inner.state = State::Idle;
            inner.update_available = available;
            inner.pending_version = manifest.version.clone();
            inner.pending_url = manifest.url.clone();
        }

        tracing::info!(
            current = %self.current_version,
            pending = %manifest.version,
            available,
            "update check complete"
        );

        Ok((available, manifest.version, manifest.description))
    }

    /// Creates a btrfs snapshot, downloads the update, and applies it via pacman.
    /// Returns immediately; progress is reported via UpdateProgress signals.
    async fn apply_update(&self) -> fdo::Result<()> {
        let (pending_version, pending_url) = {
            let inner = self.inner.lock().unwrap();
            if !inner.update_available {
                return Err(fdo::Error::Failed(
                    "no update available — run CheckForUpdates first".into(),
                ));
            }
            if inner.state != State::Idle {
                return Err(fdo::Error::Failed(format!(
                    "cannot apply update in state '{}'",
                    inner.state.as_str()
                )));
            }
            (inner.pending_version.clone(), inner.pending_url.clone())
        };

        let inner = Arc::clone(&self.inner);
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            macro_rules! progress {
                ($stage:expr, $pct:expr, $msg:expr) => {
                    event_tx
                        .send(UpdaterEvent::Progress {
                            stage: $stage.into(),
                            percent: $pct,
                            message: $msg.into(),
                        })
                        .await
                        .ok();
                };
            }
            macro_rules! fail {
                ($msg:expr) => {{
                    inner.lock().unwrap().state = State::Error;
                    event_tx
                        .send(UpdaterEvent::Failed { error: $msg.into() })
                        .await
                        .ok();
                    return;
                }};
            }

            // 1. Snapshot root before touching anything.
            inner.lock().unwrap().state = State::Snapshotting;
            progress!("snapshotting", 0, "Creating system snapshot...");

            let snap_name = snapshot_name_now();
            let snap_path = snapshots_dir().join(&snap_name);
            let snap = Command::new("btrfs")
                .args([
                    "subvolume",
                    "snapshot",
                    "/",
                    snap_path.to_str().unwrap_or("/.snapshots/pre-update"),
                ])
                .status()
                .await;

            match snap {
                Ok(s) if s.success() => {
                    tracing::info!(%snap_name, "btrfs snapshot created");
                    progress!(
                        "snapshotting",
                        20,
                        format!("Snapshot '{}' created", snap_name)
                    );
                }
                Ok(s) => {
                    tracing::warn!(code = ?s.code(), "btrfs snapshot non-zero — continuing");
                    progress!(
                        "snapshotting",
                        20,
                        "Snapshot failed (continuing without it)"
                    );
                }
                Err(e) => {
                    tracing::warn!(error = %e, "btrfs unavailable — continuing without snapshot");
                    progress!("snapshotting", 20, "Snapshot skipped (btrfs unavailable)");
                }
            }

            // 2. Download + apply (or system-wide upgrade when no URL given).
            if pending_url.is_empty() {
                inner.lock().unwrap().state = State::Applying;
                progress!(
                    "applying",
                    25,
                    "Running full system upgrade (pacman -Syu)..."
                );

                let upgrade = Command::new("pacman")
                    .args(["-Syu", "--noconfirm"])
                    .status()
                    .await;

                if !matches!(upgrade, Ok(s) if s.success()) {
                    tracing::error!("pacman -Syu failed");
                    fail!("system upgrade failed");
                }
            } else {
                // Download package.
                inner.lock().unwrap().state = State::Downloading;
                progress!("downloading", 25, "Downloading update package...");

                let tmp = "/tmp/minecrarch-update.pkg.tar.zst";
                let dl = Command::new("curl")
                    .args(["-fsSL", "--max-time", "600", "-o", tmp, &pending_url])
                    .status()
                    .await;

                if !matches!(dl, Ok(s) if s.success()) {
                    tracing::error!(%pending_url, "update download failed");
                    fail!("download failed");
                }

                progress!("downloading", 60, "Download complete");

                // Apply downloaded package.
                inner.lock().unwrap().state = State::Applying;
                progress!("applying", 65, "Installing update package...");

                let install = Command::new("pacman")
                    .args(["-U", "--noconfirm", tmp])
                    .status()
                    .await;

                let _ = tokio::fs::remove_file(tmp).await;

                if !matches!(install, Ok(s) if s.success()) {
                    tracing::error!("pacman -U failed");
                    fail!("package install failed");
                }
            }

            progress!("applying", 100, "Update applied successfully");
            inner.lock().unwrap().state = State::Idle;
            event_tx
                .send(UpdaterEvent::Complete {
                    new_version: pending_version,
                })
                .await
                .ok();
        });

        Ok(())
    }

    /// Stages a rollback to a previous snapshot. Writes a rollback-target marker
    /// and attempts to set the btrfs default subvolume for the next boot.
    /// Reboot required to complete.
    async fn rollback(&self, snapshot_name: String) -> fdo::Result<()> {
        {
            let inner = self.inner.lock().unwrap();
            if inner.state != State::Idle {
                return Err(fdo::Error::Failed(format!(
                    "cannot roll back in state '{}'",
                    inner.state.as_str()
                )));
            }
        }

        let snap_path = snapshots_dir().join(&snapshot_name);
        if !snap_path.exists() {
            return Err(fdo::Error::Failed(format!(
                "snapshot '{}' not found",
                snapshot_name
            )));
        }

        let inner = Arc::clone(&self.inner);
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            inner.lock().unwrap().state = State::RollingBack;

            macro_rules! progress {
                ($pct:expr, $msg:expr) => {
                    event_tx
                        .send(UpdaterEvent::Progress {
                            stage: "rolling-back".into(),
                            percent: $pct,
                            message: $msg.into(),
                        })
                        .await
                        .ok();
                };
            }

            progress!(0, format!("Staging rollback to '{}'...", snapshot_name));

            // Write marker so boot tooling knows which snapshot to activate.
            let marker_dir = std::path::Path::new("/var/lib/minecrarch");
            let _ = tokio::fs::create_dir_all(marker_dir).await;
            if let Err(e) =
                tokio::fs::write(marker_dir.join("rollback-target"), snapshot_name.as_bytes()).await
            {
                tracing::warn!(error = %e, "could not write rollback-target marker");
            }

            // Attempt to set the btrfs default subvolume so the next boot uses
            // the snapshot directly, without needing the marker file.
            let show = Command::new("btrfs")
                .args(["subvolume", "show", snap_path.to_str().unwrap_or("")])
                .output()
                .await;

            if let Ok(out) = show {
                if let Some(id) = parse_subvolume_id(&out.stdout) {
                    let set = Command::new("btrfs")
                        .args(["subvolume", "set-default", &id.to_string(), "/"])
                        .status()
                        .await;
                    match set {
                        Ok(s) if s.success() => {
                            tracing::info!(id, %snapshot_name, "btrfs default subvolume set");
                        }
                        _ => {
                            tracing::warn!(
                                %snapshot_name,
                                "btrfs set-default failed — relying on rollback-target marker"
                            );
                        }
                    }
                }
            }

            progress!(100, "Rollback staged — reboot to complete");
            inner.lock().unwrap().state = State::Idle;
        });

        Ok(())
    }

    /// Lists available btrfs snapshots in the snapshots directory.
    async fn list_snapshots(&self) -> Vec<String> {
        let dir = snapshots_dir();
        let mut snapshots = Vec::new();
        if let Ok(mut entries) = tokio::fs::read_dir(&dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
                if is_dir {
                    if let Some(name) = entry.file_name().to_str() {
                        if !name.starts_with('.') {
                            snapshots.push(name.to_owned());
                        }
                    }
                }
            }
        }
        snapshots.sort();
        snapshots
    }

    #[zbus(property)]
    async fn current_version(&self) -> String {
        self.current_version.clone()
    }

    #[zbus(property)]
    async fn update_available(&self) -> bool {
        self.inner.lock().unwrap().update_available
    }

    #[zbus(property)]
    async fn state(&self) -> String {
        self.inner.lock().unwrap().state.as_str().to_owned()
    }

    #[zbus(property)]
    async fn pending_version(&self) -> String {
        self.inner.lock().unwrap().pending_version.clone()
    }

    #[zbus(signal)]
    pub async fn update_progress(
        ctxt: &SignalContext<'_>,
        stage: &str,
        percent: u32,
        message: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn update_complete(ctxt: &SignalContext<'_>, new_version: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn update_failed(ctxt: &SignalContext<'_>, error: &str) -> zbus::Result<()>;
}
