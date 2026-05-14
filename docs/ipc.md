# MinecrarchOS — IPC Strategy

## Overview

All inter-process communication between the Minecrarch Shell and Runtime Services uses D-Bus on the systemd user session bus. See [ADR-0012](./adr/0012-ipc-mechanism.md) for the rationale.

This document defines:
- The D-Bus naming conventions used across the platform.
- The interface contracts for each service.
- The error taxonomy.
- Patterns for async notifications, state queries, and high-frequency data.

---

## Naming Conventions

| Concept | Convention | Example |
|---|---|---|
| Bus name | `org.minecrarch.<Service>` | `org.minecrarch.ModpackManager` |
| Object path | `/org/minecrarch/<Service>` | `/org/minecrarch/ModpackManager` |
| Interface | `org.minecrarch.<Service>` | `org.minecrarch.ModpackManager` |
| Error namespace | `org.minecrarch.Error.<Name>` | `org.minecrarch.Error.InstanceNotFound` |
| Signal | PascalCase verb/noun | `GameCrashed`, `UpdateAvailable` |
| Method | PascalCase verb/noun | `LaunchInstance`, `CheckForUpdates` |
| Property | PascalCase noun | `ActiveInstance`, `CurrentVersion` |

All services register on the **user session bus** (`DBUS_SESSION_BUS_ADDRESS`), not the system bus. The session bus is started by systemd as part of the user session and is available to all user processes in the Gamescope session.

---

## Shell Integration Pattern

The shell connects to all services on startup during `INITIALIZING` state:

```rust
// Shell startup (zbus, Rust)
let conn = Connection::session().await?;

// Verify all critical services are present
conn.call_method(
    Some("org.minecrarch.ModpackManager"),
    "/org/minecrarch/ModpackManager",
    Some("org.freedesktop.DBus.Peer"),
    "Ping",
    &(),
).await?;

// Subscribe to signals from all services
let modpack_proxy = ModpackManagerProxy::new(&conn).await?;
let mut game_started = modpack_proxy.receive_game_started().await?;
let mut game_crashed = modpack_proxy.receive_game_crashed().await?;

// Drive UI updates from async signal stream
tokio::spawn(async move {
    while let Some(signal) = game_crashed.next().await {
        // transition shell state machine to RECOVERING
    }
});
```

---

## Service Interfaces

### `org.minecrarch.ModpackManager`

Object path: `/org/minecrarch/ModpackManager`

The central coordination service for game instances. Owns the lifecycle of the Minecraft process.

#### Methods

```xml
<interface name="org.minecrarch.ModpackManager">

  <!-- Returns all known instances -->
  <method name="ListInstances">
    <arg direction="out" name="instances" type="aa{sv}"/>
    <!-- Each dict contains: id(s), name(s), edition(s), version(s), status(s) -->
  </method>

  <!-- Launch a game instance. Emits GameStarted or GameCrashed. -->
  <method name="LaunchInstance">
    <arg direction="in" name="id" type="s"/>
    <!-- Errors: InstanceNotFound, AlreadyRunning, LaunchFailed -->
  </method>

  <!-- Request orderly game termination. Emits GameExited. -->
  <method name="StopInstance">
    <arg direction="in" name="id" type="s"/>
    <!-- Errors: InstanceNotFound, NotRunning -->
  </method>

  <!-- Suspend game (SIGSTOP). Used during system suspend. -->
  <method name="SuspendInstance">
    <arg direction="in" name="id" type="s"/>
  </method>

  <!-- Resume game (SIGCONT). Used on system resume. -->
  <method name="ResumeInstance">
    <arg direction="in" name="id" type="s"/>
  </method>

  <!-- Install a modpack into a new instance. Emits InstallProgress* signals. -->
  <method name="InstallModpack">
    <arg direction="in" name="source_url" type="s"/>
    <arg direction="in" name="instance_id" type="s"/>
    <!-- Errors: InvalidSource, InsufficientSpace, InstallFailed -->
  </method>

  <!-- Remove an instance and its data. -->
  <method name="RemoveInstance">
    <arg direction="in" name="id" type="s"/>
    <!-- Errors: InstanceNotFound, InstanceRunning -->
  </method>

  <!-- Get detailed status of one instance. -->
  <method name="GetInstanceStatus">
    <arg direction="in" name="id" type="s"/>
    <arg direction="out" name="status" type="a{sv}"/>
    <!-- Returns: state(s), pid(u), uptime_secs(u), last_crash_reason(s) -->
  </method>

</interface>
```

#### Signals

```xml
  <!-- Game process started and Wayland surface registered. -->
  <signal name="GameStarted">
    <arg name="instance_id" type="s"/>
    <arg name="pid" type="u"/>
  </signal>

  <!-- Game exited cleanly (exit code 0). -->
  <signal name="GameExited">
    <arg name="instance_id" type="s"/>
    <arg name="exit_code" type="i"/>
  </signal>

  <!-- Game exited unexpectedly. -->
  <signal name="GameCrashed">
    <arg name="instance_id" type="s"/>
    <arg name="exit_code" type="i"/>
    <arg name="signal_name" type="s"/>  <!-- "SIGSEGV", "OOM", "WATCHDOG", "" -->
  </signal>

  <!-- Modpack install progress (rate-limited to max 2/s). -->
  <signal name="InstallProgress">
    <arg name="instance_id" type="s"/>
    <arg name="percent" type="d"/>
    <arg name="stage" type="s"/>  <!-- "downloading", "verifying", "extracting" -->
    <arg name="bytes_done" type="t"/>
    <arg name="bytes_total" type="t"/>
  </signal>

  <!-- Modpack install completed. -->
  <signal name="InstallComplete">
    <arg name="instance_id" type="s"/>
  </signal>

  <!-- Modpack install failed. -->
  <signal name="InstallFailed">
    <arg name="instance_id" type="s"/>
    <arg name="reason" type="s"/>
  </signal>
```

#### Properties

```xml
  <!-- ID of the currently running instance, or "" if none. -->
  <property name="ActiveInstance" type="s" access="read"/>

  <!-- Number of known instances. -->
  <property name="InstanceCount" type="u" access="read"/>
```

---

### `org.minecrarch.Overlay`

Object path: `/org/minecrarch/Overlay`

Renders HUD overlays above the game surface using `wlr-layer-shell`. The shell calls this service to display notifications without directly owning the overlay surface.

#### Methods

```xml
<interface name="org.minecrarch.Overlay">

  <!-- Display a transient notification. Auto-dismisses after duration_ms. -->
  <method name="ShowNotification">
    <arg direction="in" name="text" type="s"/>
    <arg direction="in" name="duration_ms" type="u"/>
    <arg direction="in" name="level" type="s"/>  <!-- "info", "warning", "error" -->
  </method>

  <!-- Display the crash notification overlay (persistent until dismissed). -->
  <method name="ShowCrashOverlay">
    <arg direction="in" name="reason" type="s"/>
    <arg direction="in" name="instance_id" type="s"/>
  </method>

  <!-- Display the in-game system menu overlay (interactive). -->
  <method name="ShowSystemMenu">
  </method>

  <!-- Hide all active overlays immediately. -->
  <method name="HideAll">
  </method>

</interface>
```

#### Signals

```xml
  <!-- User interacted with system menu (e.g., chose "Quit Game"). -->
  <signal name="SystemMenuAction">
    <arg name="action" type="s"/>  <!-- "quit_game", "suspend", "shutdown", "return" -->
  </signal>
```

#### Properties

```xml
  <!-- True if any overlay surface is currently visible. -->
  <property name="Visible" type="b" access="read"/>

  <!-- True if an interactive overlay (system menu) has input focus. -->
  <property name="HasInputFocus" type="b" access="read"/>
```

---

### `org.minecrarch.Updater`

Object path: `/org/minecrarch/Updater`

Manages platform updates via pacman, btrfs snapshots for rollback safety, and the update UI flow.

#### Methods

```xml
<interface name="org.minecrarch.Updater">

  <!-- Check for available platform updates. Emits UpdateAvailable if found. -->
  <method name="CheckForUpdates">
    <arg direction="out" name="update_available" type="b"/>
  </method>

  <!-- Apply the pending update. Creates btrfs snapshot before applying. -->
  <!-- Emits UpdateProgress and then UpdateApplied or UpdateFailed. -->
  <method name="ApplyUpdate">
    <!-- Errors: NoUpdateAvailable, SnapshotFailed, UpdateFailed -->
  </method>

  <!-- Roll back to the last snapshot. -->
  <method name="Rollback">
    <!-- Errors: NoSnapshotAvailable, RollbackFailed -->
  </method>

  <!-- Get current platform version string. -->
  <method name="GetCurrentVersion">
    <arg direction="out" name="version" type="s"/>
  </method>

  <!-- List available rollback snapshots. -->
  <method name="ListSnapshots">
    <arg direction="out" name="snapshots" type="aa{sv}"/>
    <!-- Each dict: id(s), timestamp(t), version(s), size_bytes(t) -->
  </method>

</interface>
```

#### Signals

```xml
  <!-- Update is available (emitted after CheckForUpdates or on periodic check). -->
  <signal name="UpdateAvailable">
    <arg name="new_version" type="s"/>
    <arg name="changelog" type="s"/>
    <arg name="size_bytes" type="t"/>
  </signal>

  <!-- Update application progress (rate-limited to max 1/s). -->
  <signal name="UpdateProgress">
    <arg name="percent" type="d"/>
    <arg name="stage" type="s"/>  <!-- "snapshot", "downloading", "installing", "verifying" -->
  </signal>

  <!-- Update applied successfully. Reboot required. -->
  <signal name="UpdateApplied">
    <arg name="new_version" type="s"/>
    <arg name="reboot_required" type="b"/>
  </signal>

  <!-- Update failed. Snapshot preserved for safety. -->
  <signal name="UpdateFailed">
    <arg name="reason" type="s"/>
    <arg name="snapshot_preserved" type="b"/>
  </signal>

  <!-- Rollback completed. Reboot required to activate. -->
  <signal name="RollbackComplete">
    <arg name="target_version" type="s"/>
  </signal>
```

#### Properties

```xml
  <property name="CurrentVersion" type="s" access="read"/>
  <property name="UpdateAvailable" type="b" access="read"/>
  <property name="PendingUpdateVersion" type="s" access="read"/>  <!-- "" if none -->
  <property name="LastSnapshotTimestamp" type="t" access="read"/>  <!-- Unix epoch, 0 if none -->
```

---

### `org.minecrarch.Logging`

Object path: `/org/minecrarch/Logging`

Structured log configuration. Actual log data flows through journald (each service writes to its own journal stream) — this service only controls log level configuration and log retrieval queries.

#### Methods

```xml
<interface name="org.minecrarch.Logging">

  <!-- Set the log level for a named component. -->
  <method name="SetLogLevel">
    <arg direction="in" name="component" type="s"/>  <!-- "shell", "modpack", "overlay", "updater", "runtime" -->
    <arg direction="in" name="level" type="s"/>       <!-- "debug", "info", "warn", "error" -->
  </method>

  <!-- Get the current log level for a component. -->
  <method name="GetLogLevel">
    <arg direction="in" name="component" type="s"/>
    <arg direction="out" name="level" type="s"/>
  </method>

  <!-- Get the journald cursor for the last game crash (for the shell's "View Logs" UI). -->
  <method name="GetLastCrashCursor">
    <arg direction="in" name="instance_id" type="s"/>
    <arg direction="out" name="cursor" type="s"/>  <!-- journald cursor string, "" if none -->
  </method>

</interface>
```

---

## Error Taxonomy

All D-Bus method calls may return errors. Errors follow the `org.minecrarch.Error.<Name>` namespace.

| Error | Meaning |
|---|---|
| `org.minecrarch.Error.InstanceNotFound` | The requested instance ID does not exist |
| `org.minecrarch.Error.AlreadyRunning` | A game instance is already running; only one at a time |
| `org.minecrarch.Error.NotRunning` | Operation requires a running game, but none is active |
| `org.minecrarch.Error.LaunchFailed` | Game process could not be started |
| `org.minecrarch.Error.InstallFailed` | Modpack installation failed |
| `org.minecrarch.Error.InvalidSource` | The modpack source URL is invalid or unsupported |
| `org.minecrarch.Error.InsufficientSpace` | Not enough disk space for the operation |
| `org.minecrarch.Error.NoUpdateAvailable` | `ApplyUpdate` called when no update is pending |
| `org.minecrarch.Error.SnapshotFailed` | btrfs snapshot could not be created before update |
| `org.minecrarch.Error.UpdateFailed` | pacman update failed; snapshot preserved |
| `org.minecrarch.Error.NoSnapshotAvailable` | `Rollback` called but no snapshot exists |
| `org.minecrarch.Error.RollbackFailed` | Rollback operation failed |
| `org.minecrarch.Error.InstanceRunning` | `RemoveInstance` called while the instance is running |

---

## High-Frequency Data Patterns

D-Bus is not suitable for high-frequency data streams. Two patterns are used for data that updates frequently:

### Rate-Limited Signals

Signals that represent progress (install, update) must be rate-limited to a maximum of **2 signals/second** on the service side before emitting. The service accumulates state internally and emits only when a meaningful change has occurred or the rate limit timer fires.

### Property Polling

For data the shell needs to display continuously (e.g., download speed, CPU usage of the game):

- The service exposes the value as a D-Bus property that changes and emits `PropertiesChanged`.
- The shell subscribes to `PropertiesChanged` on the service's interface.
- `PropertiesChanged` is still rate-limited to max 2/s.

Never design a flow where the shell polls via method calls in a tight loop. Use signals or property change notifications exclusively.

---

## Versioning and Compatibility

All interfaces are at version 1 during Phase 1. Interface versioning strategy:

- Interface names do not include a version number in the initial release.
- Breaking changes to an interface require a new interface name (e.g., `org.minecrarch.ModpackManager2`).
- The old interface is kept and marked deprecated; removed only in a major version bump.
- Additive changes (new methods, new signals, new properties) do not require a version bump — D-Bus introspection allows the shell to check for method existence before calling.

---

## Debugging

```bash
# List all registered MinecrarchOS services on the session bus
busctl --user list | grep minecrarch

# Introspect a service interface
busctl --user introspect org.minecrarch.ModpackManager /org/minecrarch/ModpackManager

# Monitor all signals from all minecrarch services
busctl --user monitor org.minecrarch.ModpackManager org.minecrarch.Updater

# Call a method manually (useful for testing services without the shell)
busctl --user call org.minecrarch.ModpackManager \
  /org/minecrarch/ModpackManager \
  org.minecrarch.ModpackManager \
  ListInstances

# Watch property changes in real time
busctl --user monitor --match "interface='org.freedesktop.DBus.Properties',member='PropertiesChanged'"
```

For a graphical D-Bus inspector, use `d-spy` (available in Arch repositories).
