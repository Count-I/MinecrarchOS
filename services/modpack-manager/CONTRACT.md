# Service Contract: ModpackManager

**D-Bus bus name:** `org.minecrarch.ModpackManager`
**Object path:** `/org/minecrarch/ModpackManager`
**Interface:** `org.minecrarch.ModpackManager`
**IPC reference:** [`docs/ipc.md — ModpackManager`](../../docs/ipc.md)

---

## Owned Responsibilities

- Minecraft instance lifecycle: create, launch, stop, suspend, resume, remove.
- Modpack installation: download, checksum verification, extraction, Prism instance configuration.
- Prism Launcher integration: all Prism CLI interaction and instance directory management.
- Game process supervision: crash detection via systemd scope state, signal classification.
- `GameStarted`, `GameExited`, `GameCrashed`, `InstallProgress`, `InstallComplete`, `InstallFailed` signal emission.
- Instance state persistence: which instances exist, their configuration, their last known state.

## Explicit Non-Responsibilities

This service must NOT:

- Render any UI (belongs to `shell/` or `services/overlay`).
- Manage JVM installations (future: belongs to a dedicated JVM service).
- Parse Mojang version manifests (future: belongs to a dedicated asset service).
- Manage platform updates (belongs to `services/updater`).
- Manage log aggregation (belongs to `services/logging`).
- Directly communicate with other services via D-Bus (all service↔service communication goes through the shell or is event-driven via the bus; this service does not call other service methods directly).
- Access GPU or display resources.

## Lifecycle

**Startup:**
1. Register `org.minecrarch.ModpackManager` on the user session bus.
2. Load instance state from disk (`~/.local/share/minecrarch/instances/`).
3. Check for any orphaned game scopes from a previous session (e.g., unclean shutdown). Emit `GameCrashed` for any orphaned scope found.
4. Ready to accept method calls.

**Shutdown:**
1. Receive SIGTERM from systemd.
2. If a game is running: send SIGTERM to the game scope; wait up to 5s; send SIGKILL.
3. Flush instance state to disk.
4. Exit cleanly.

**Restart policy:** `Restart=on-failure` with `RestartSec=2s`. Maximum 5 restarts in 60s before systemd gives up and emits a failure to the system log.

## Failure Behavior

- If Prism Launcher CLI is unavailable: `LaunchInstance` and `InstallModpack` return `org.minecrarch.Error.LaunchFailed`. Service remains alive and responds to `ListInstances` and `GetInstanceStatus`.
- If instance state file is corrupted: log the corruption to journald, start with an empty instance list, do not crash.
- If a running game process cannot be found on service restart: treat it as crashed, emit `GameCrashed(id, -1, "UNKNOWN")`.

## State Ownership

This service owns:
- The list of all known instances and their metadata.
- The PID and scope name of the currently running game (if any).
- The result of the last launch (success, failure, crash reason).
- Download progress state for active modpack installations.

This service does NOT own:
- The visual state of the shell (shell owns that).
- Platform version information (updater owns that).
- Log routing configuration (logging service owns that).
