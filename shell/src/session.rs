/// Top-level session states — matches the state machine in docs/state-machines.md.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum SessionState {
    #[default]
    Initializing,
    Menu,
    Launching {
        instance_id: String,
    },
    InGame {
        instance_id: String,
        pid: u32,
    },
    Recovering {
        instance_id: String,
        exit_code: i32,
        signal: String,
    },
}

/// Events sent from the D-Bus background thread to the GTK main thread.
/// Variant names mirror the D-Bus signal names from docs/ipc.md.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum ShellEvent {
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
}

/// Commands sent from the GTK main thread to the D-Bus background thread.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ShellCommand {
    LaunchInstance { id: String },
    StopInstance { id: String },
}
