use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};
use zbus::{fdo, interface, SignalContext};

pub const OBJECT_PATH: &str = "/org/minecrarch/ModpackManager";

#[derive(Debug)]
pub enum ManagerEvent {
    GameStarted {
        instance_id: String,
        pid: u32,
    },
    GameExited {
        instance_id: String,
        exit_code: i32,
    },
    GameCrashed {
        instance_id: String,
        exit_code: i32,
        signal: String,
    },
    InstallProgress {
        instance_id: String,
        percent: u32,
        status: String,
    },
}

pub struct ModpackManager {
    active: Arc<Mutex<Option<String>>>,
    event_tx: mpsc::Sender<ManagerEvent>,
}

impl ModpackManager {
    pub fn new(event_tx: mpsc::Sender<ManagerEvent>) -> Self {
        Self {
            active: Arc::new(Mutex::new(None)),
            event_tx,
        }
    }

    fn game_binary() -> String {
        std::env::var("MINECRARCH_GAME_BINARY")
            .unwrap_or_else(|_| "/usr/bin/fake-game".to_string())
    }
}

fn prism_instances_dir() -> std::path::PathBuf {
    std::env::var("PRISM_INSTANCES_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            std::path::PathBuf::from(home).join(".local/share/PrismLauncher/instances")
        })
}

async fn read_prism_instances() -> Vec<String> {
    let dir = prism_instances_dir();
    let mut instances = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(&dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            if is_dir {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.starts_with('.') {
                        instances.push(name.to_owned());
                    }
                }
            }
        }
    }
    instances
}

#[interface(name = "org.minecrarch.ModpackManager")]
impl ModpackManager {
    async fn launch_instance(&self, id: String) -> fdo::Result<()> {
        {
            let active = self.active.lock().await;
            if active.is_some() {
                return Err(fdo::Error::Failed(
                    "org.minecrarch.Error.AlreadyRunning".into(),
                ));
            }
        }

        let binary = Self::game_binary();
        let scope_unit = format!("minecrarch-game@{}.scope", id);

        let mut child = Command::new("systemd-run")
            .args([
                "--user",
                "--scope",
                "--collect",
                &format!("--unit={}", scope_unit),
                "--",
                &binary,
            ])
            .spawn()
            .map_err(|e| fdo::Error::Failed(format!("spawn failed: {e}")))?;

        let pid = child.id().unwrap_or(0);
        *self.active.lock().await = Some(id.clone());

        tracing::info!(%id, pid, %scope_unit, "game launched");

        let _ = self
            .event_tx
            .send(ManagerEvent::GameStarted {
                instance_id: id.clone(),
                pid,
            })
            .await;

        let id_clone = id.clone();
        let active_clone = Arc::clone(&self.active);
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let event = match child.wait().await {
                Ok(status) => {
                    *active_clone.lock().await = None;
                    if status.success() {
                        ManagerEvent::GameExited {
                            instance_id: id_clone,
                            exit_code: 0,
                        }
                    } else {
                        let exit_code = status.code().unwrap_or(-1);
                        let signal = if status.code().is_none() {
                            "SIGKILL".to_string()
                        } else {
                            String::new()
                        };
                        ManagerEvent::GameCrashed {
                            instance_id: id_clone,
                            exit_code,
                            signal,
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to wait for game process");
                    *active_clone.lock().await = None;
                    ManagerEvent::GameCrashed {
                        instance_id: id_clone,
                        exit_code: -1,
                        signal: "UNKNOWN".to_string(),
                    }
                }
            };
            let _ = event_tx.send(event).await;
        });

        Ok(())
    }

    async fn stop_instance(&self, id: String) -> fdo::Result<()> {
        let active = self.active.lock().await;
        if active.as_deref() != Some(&id) {
            return Err(fdo::Error::Failed("org.minecrarch.Error.NotRunning".into()));
        }

        let scope_unit = format!("minecrarch-game@{}.scope", id);
        let status = Command::new("systemctl")
            .args(["--user", "kill", "--signal=SIGTERM", &scope_unit])
            .status()
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        if !status.success() {
            return Err(fdo::Error::Failed("systemctl kill failed".into()));
        }

        Ok(())
    }

    /// Downloads a modpack from `source_url` and imports it into Prism Launcher.
    ///
    /// Progress is reported via `InstallProgress` signals:
    ///   0%   — downloading
    ///  50%   — importing (Prism Launcher handoff)
    /// 100%   — complete
    ///
    /// Returns immediately; installation runs in the background.
    async fn install_modpack(&self, source_url: String, instance_id: String) -> fdo::Result<()> {
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let send = |percent: u32, status: &str| {
                let tx = event_tx.clone();
                let id = instance_id.clone();
                let st = status.to_owned();
                async move {
                    tx.send(ManagerEvent::InstallProgress {
                        instance_id: id,
                        percent,
                        status: st,
                    })
                    .await
                    .ok();
                }
            };

            send(0, "downloading").await;

            let tmp = format!("/tmp/minecrarch-install-{}.mrpack", instance_id);
            let dl = Command::new("curl")
                .args(["-fsSL", "-o", &tmp, &source_url])
                .status()
                .await;

            if !matches!(dl, Ok(s) if s.success()) {
                tracing::error!(%instance_id, %source_url, "modpack download failed");
                send(0, "download_failed").await;
                return;
            }

            send(50, "importing").await;

            let import = Command::new("prismlauncher")
                .args(["--import", &tmp])
                .status()
                .await;

            let _ = tokio::fs::remove_file(&tmp).await;

            match import {
                Ok(s) if s.success() => {
                    tracing::info!(%instance_id, "modpack import complete");
                }
                Ok(s) => {
                    tracing::warn!(%instance_id, code = ?s.code(), "prismlauncher --import returned non-zero");
                }
                Err(e) => {
                    tracing::warn!(%instance_id, error = %e, "prismlauncher not available or failed");
                }
            }

            send(100, "complete").await;
        });

        Ok(())
    }

    /// Lists all Prism Launcher instance IDs by reading the instances directory.
    async fn list_instances(&self) -> Vec<String> {
        read_prism_instances().await
    }

    #[zbus(property)]
    async fn active_instance(&self) -> String {
        self.active.lock().await.clone().unwrap_or_default()
    }

    #[zbus(property)]
    async fn instance_count(&self) -> u32 {
        read_prism_instances().await.len() as u32
    }

    #[zbus(signal)]
    pub async fn game_started(
        ctxt: &SignalContext<'_>,
        instance_id: &str,
        pid: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn game_exited(
        ctxt: &SignalContext<'_>,
        instance_id: &str,
        exit_code: i32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn game_crashed(
        ctxt: &SignalContext<'_>,
        instance_id: &str,
        exit_code: i32,
        signal_name: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn install_progress(
        ctxt: &SignalContext<'_>,
        instance_id: &str,
        percent: u32,
        status: &str,
    ) -> zbus::Result<()>;
}
