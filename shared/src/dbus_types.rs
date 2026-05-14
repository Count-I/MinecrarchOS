//! D-Bus type aliases and constants for MinecrarchOS IPC.
//! Full interface contracts are defined in docs/ipc.md.
//! Instance and snapshot data are passed as D-Bus dicts (a{sv}) per the IPC spec.

/// Field keys for an instance dict (`ListInstances` return type). D-Bus type: `a{sv}`.
pub mod instance_fields {
    pub const ID: &str = "id";
    pub const NAME: &str = "name";
    pub const EDITION: &str = "edition";
    pub const VERSION: &str = "version";
    pub const STATUS: &str = "status";
}

/// Field keys for a snapshot dict (ListSnapshots return type).
/// D-Bus type: a{sv} with these keys.
pub mod snapshot_fields {
    pub const ID: &str = "id";
    pub const TIMESTAMP: &str = "timestamp";
    pub const VERSION: &str = "version";
    pub const SIZE_BYTES: &str = "size_bytes";
}

/// D-Bus error name prefix for all MinecrarchOS errors.
pub const ERROR_NAMESPACE: &str = "org.minecrarch.Error";

/// D-Bus bus names for all platform services.
pub mod bus_names {
    pub const MODPACK_MANAGER: &str = "org.minecrarch.ModpackManager";
    pub const OVERLAY: &str = "org.minecrarch.Overlay";
    pub const UPDATER: &str = "org.minecrarch.Updater";
    pub const LOGGING: &str = "org.minecrarch.Logging";
}

/// D-Bus object paths for all platform services.
pub mod object_paths {
    pub const MODPACK_MANAGER: &str = "/org/minecrarch/ModpackManager";
    pub const OVERLAY: &str = "/org/minecrarch/Overlay";
    pub const UPDATER: &str = "/org/minecrarch/Updater";
    pub const LOGGING: &str = "/org/minecrarch/Logging";
}
