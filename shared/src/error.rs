use thiserror::Error;

#[derive(Debug, Error)]
pub enum MinecrarchError {
    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Launch failed: {0}")]
    Launch(String),

    #[error("Install failed: {0}")]
    Install(String),

    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
