use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};
use zbus::{fdo, interface, SignalContext};

pub const OBJECT_PATH: &str = "/org/minecrarch/ModpackManager";

#[derive(Debug)]
pub enum GameEvent {
    Started {
        instance_id: String,
        pid: u32,
    },
    Exited {
        instance_id: String,
        exit_code: i32,
    },
    Crashed {
        instance_id: String,
        exit_code: i32,
        signal: String,
    },
}

pub struct ModpackManager {
    active: Arc<Mutex<Option<String>>>,
    event_tx: mpsc::Sender<GameEvent>,
}

impl ModpackManager {
    pub fn new(event_tx: mpsc::Sender<GameEvent>) -> Self {
        Self {
            active: Arc::new(Mutex::new(None)),
            event_tx,
        }
    }

    fn game_binary() -> String {
        std::env::var("MINECRARCH_GAME_BINARY").unwrap_or_else(|_| "/usr/bin/fake-game".to_string())
    }
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
            .send(GameEvent::Started {
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
                        GameEvent::Exited {
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
                        GameEvent::Crashed {
                            instance_id: id_clone,
                            exit_code,
                            signal,
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to wait for game process");
                    *active_clone.lock().await = None;
                    GameEvent::Crashed {
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

    /// Returns an empty list — full instance management is Phase 2.
    async fn list_instances(&self) -> Vec<String> {
        vec![]
    }

    #[zbus(property)]
    async fn active_instance(&self) -> String {
        self.active.lock().await.clone().unwrap_or_default()
    }

    #[zbus(property)]
    async fn instance_count(&self) -> u32 {
        0
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
}
