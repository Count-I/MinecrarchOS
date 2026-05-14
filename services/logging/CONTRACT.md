# Service Contract: Logging

**D-Bus bus name:** `org.minecrarch.Logging`
**Object path:** `/org/minecrarch/Logging`
**Interface:** `org.minecrarch.Logging`
**IPC reference:** [`docs/ipc.md — Logging`](../../docs/ipc.md)

---

## Owned Responsibilities

- Log level configuration for all platform components.
- `GetLastCrashCursor`: returning the journald cursor for the most recent game crash of a given instance (used by the shell to display crash logs in recovery UI).
- Structured log routing policy: ensuring all platform components write structured JSON logs to journald with consistent field names.
- `SetLogLevel` and `GetLogLevel` D-Bus method handling.

## Explicit Non-Responsibilities

This service must NOT:

- Implement a custom log aggregation daemon. journald IS the log system. This service configures it, not replaces it.
- Buffer, store, or re-emit logs through its own pipeline. All logs flow to journald directly from each component.
- Be on the critical path for any functional operation. A logging service failure must never prevent game launches, UI rendering, or crash recovery.
- Parse or process log content. It stores and retrieves journal cursors but does not analyze log entries.

## Lifecycle

**Startup:**
1. Register `org.minecrarch.Logging` on the user session bus.
2. Apply configured log levels to the appropriate systemd service units (via journald runtime configuration or unit overrides).
3. Ready to accept method calls.

**Shutdown:**
1. Exit cleanly. No state flush needed (configuration is applied to systemd units, not held in memory).

**Restart policy:** `Restart=on-failure` with `RestartSec=5s`. This service is non-critical — the platform operates normally without it. Log levels fall back to defaults if the service is unavailable.

## Failure Behavior

- If journald is unavailable (unusual — journald is a core systemd component): log to stderr, return errors from all method calls, but do not crash the service.
- If a requested component name is unknown in `SetLogLevel`: return `org.minecrarch.Error.InvalidComponent` but do not crash.
- All callers must treat logging service calls as non-blocking and tolerate failures silently.

## Log Field Conventions

All platform components must write structured logs with the following fields, so journald queries work consistently:

| Field | Value | Required by |
|---|---|---|
| `MINECRARCH_COMPONENT` | `shell`, `modpack-manager`, `overlay`, `updater`, `runtime` | All components |
| `MINECRARCH_INSTANCE` | instance ID (if log is instance-specific) | modpack-manager, runtime |
| `MINECRARCH_EVENT` | `GAME_STARTED`, `GAME_CRASHED`, `INSTALL_COMPLETE`, etc. | modpack-manager |

This allows targeted journald queries:

```bash
journalctl _MINECRARCH_COMPONENT=modpack-manager _MINECRARCH_EVENT=GAME_CRASHED -n 50
```

## State Ownership

This service owns:
- Current log level configuration per component (in-memory; applied to systemd units).
- A mapping of instance ID → last crash journal cursor (in-memory cache, populated from journald on demand).

This service does NOT own:
- The log data itself (journald owns this).
- Any game or session state.
