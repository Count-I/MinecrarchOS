# MinecrarchOS — State Machines

These are the canonical state machine definitions for all major runtime flows. Implementation must conform to these diagrams. Any deviation requires an ADR.

---

## 1. Session Lifecycle

The top-level state of the platform from boot to shutdown.

```mermaid
stateDiagram-v2
    [*] --> BOOTING: power on

    BOOTING --> INITIALIZING: all services registered on D-Bus

    INITIALIZING --> MENU: shell UI rendered
    INITIALIZING --> BOOTING: critical service unavailable (restart)

    MENU --> LAUNCHING: user selects game (LaunchInstance called)

    LAUNCHING --> IN_GAME: GameStarted signal received
    LAUNCHING --> RECOVERING: GameCrashed signal or launch timeout

    IN_GAME --> MENU: GameExited(exit_code=0) signal
    IN_GAME --> RECOVERING: GameCrashed signal
    IN_GAME --> PAUSED: PrepareForSleep(true) signal from logind

    PAUSED --> IN_GAME: PrepareForSleep(false) + game process alive (SIGCONT)
    PAUSED --> MENU: PrepareForSleep(false) + game process dead

    RECOVERING --> MENU: user dismisses recovery UI
    RECOVERING --> LAUNCHING: user selects restart

    MENU --> [*]: user initiates shutdown
    IN_GAME --> [*]: shutdown during game (orderly termination first)
```

**State ownership:** Minecrarch Shell holds and transitions the session state. Services report events; the shell decides transitions.

---

## 2. Game Process Lifecycle

The lifecycle of a single Minecraft process instance, managed by the runtime layer.

```mermaid
stateDiagram-v2
    [*] --> IDLE: service starts

    IDLE --> PREPARING: LaunchInstance(id) received via D-Bus

    PREPARING --> LAUNCHING: systemd-run scope created
    PREPARING --> FAILED: scope creation failed

    LAUNCHING --> RUNNING: process alive + Wayland surface registered
    LAUNCHING --> FAILED: process exits before Wayland surface (launch failure)
    LAUNCHING --> FAILED: launch timeout (30s)

    RUNNING --> STOPPING: StopInstance(id) received
    RUNNING --> CRASHED: process exits non-zero or killed by signal
    RUNNING --> HUNG: watchdog timeout (no Wayland frame for 60s)
    RUNNING --> SUSPENDED: SuspendInstance(id) received (SIGSTOP)

    SUSPENDED --> RUNNING: ResumeInstance(id) received (SIGCONT)
    SUSPENDED --> CRASHED: process killed during suspend (OOM etc.)

    STOPPING --> TERMINATED: clean exit within 10s
    STOPPING --> TERMINATED: SIGKILL after 10s timeout

    HUNG --> CRASHED: SIGKILL sent after watchdog fires

    CRASHED --> IDLE: GameCrashed signal emitted, scope cleaned up
    TERMINATED --> IDLE: GameExited signal emitted, scope cleaned up
    FAILED --> IDLE: GameCrashed(launch_failure) signal emitted
```

**State ownership:** `services/modpack-manager` owns game process state. The runtime layer supervises the process; the service translates process events to D-Bus signals.

**Exit signals emitted:**

| Terminal state | D-Bus signal |
|---|---|
| `TERMINATED` (exit 0) | `GameExited(id, 0)` |
| `CRASHED` (non-zero exit) | `GameCrashed(id, exit_code, signal_name)` |
| `CRASHED` (OOM) | `GameCrashed(id, -1, "OOM")` |
| `CRASHED` (hung) | `GameCrashed(id, -1, "WATCHDOG")` |
| `FAILED` (launch) | `GameCrashed(id, -1, "LAUNCH_FAILURE")` |

---

## 3. Crash Recovery Flow

The recovery flow from `RECOVERING` session state back to a stable state.

```mermaid
stateDiagram-v2
    [*] --> DETECTING: GameCrashed signal received by shell

    DETECTING --> RECLAIMING_FOCUS: shell requests Wayland focus from Gamescope
    RECLAIMING_FOCUS --> SHOWING_RECOVERY_UI: focus granted
    SHOWING_RECOVERY_UI --> AWAITING_USER: recovery screen rendered

    AWAITING_USER --> RESTARTING: user selects "Restart Game"
    AWAITING_USER --> RETURNING: user selects "Return to Menu"
    AWAITING_USER --> VIEWING_LOGS: user selects "View Crash Log"

    VIEWING_LOGS --> AWAITING_USER: user closes log view

    RESTARTING --> [*]: session transitions to LAUNCHING
    RETURNING --> [*]: session transitions to MENU
```

**Invariant:** The shell must regain Wayland focus before rendering recovery UI. If Gamescope does not release focus within 2s, the shell logs the failure and forces a session restart.

---

## 4. Suspend/Resume Flow

```mermaid
stateDiagram-v2
    [*] --> ACTIVE: system running normally

    ACTIVE --> INHIBITING: PrepareForSleep(true) received
    note right of INHIBITING: shell acquires systemd inhibitor lock

    INHIBITING --> GAME_STOPPING: session state is IN_GAME
    INHIBITING --> READY_TO_SLEEP: session state is MENU or other

    GAME_STOPPING --> GAME_SUSPENDED: SuspendInstance(id) sent → SIGSTOP
    GAME_STOPPING --> GAME_TERMINATED: game process does not support suspend
    GAME_SUSPENDED --> READY_TO_SLEEP: inhibitor lock released
    GAME_TERMINATED --> READY_TO_SLEEP: inhibitor lock released

    READY_TO_SLEEP --> [*]: system suspends (kernel takes over)

    [*] --> RESUMING: PrepareForSleep(false) received

    RESUMING --> CHECKING_GAME: was game running before suspend?

    CHECKING_GAME --> RESUMING_GAME: game process still alive
    CHECKING_GAME --> CRASH_RECOVERY: game process dead

    RESUMING_GAME --> ACTIVE: SIGCONT sent → session returns to IN_GAME
    CRASH_RECOVERY --> ACTIVE: session transitions to RECOVERING
```

**Invariant:** The inhibitor lock is always released before returning from the INHIBITING state, regardless of what happens to the game process. A failed SIGSTOP must not block system suspend.

---

## 5. Service Degradation

When a non-critical runtime service crashes and restarts.

```mermaid
stateDiagram-v2
    [*] --> HEALTHY: all services registered on D-Bus

    HEALTHY --> DEGRADED: service bus name disappears (NameOwnerChanged)

    DEGRADED --> DEGRADED: service restarting (systemd restart policy)
    note right of DEGRADED: shell disables UI features that\ndepend on the crashed service

    DEGRADED --> RECOVERING_SERVICE: service bus name reappears
    RECOVERING_SERVICE --> HEALTHY: shell re-subscribes to signals, restores UI

    DEGRADED --> CRITICAL_FAILURE: critical service (ModpackManager) down for > 30s
    CRITICAL_FAILURE --> [*]: shell forces session restart
```

**Critical services:** `org.minecrarch.ModpackManager`, `org.minecrarch.Logging`
**Non-critical services:** `org.minecrarch.Overlay`, `org.minecrarch.Updater`

The shell must never crash because a service crashed. Service failure is an expected operational event, not an exception.

---

## 6. Modpack Install Flow

```mermaid
stateDiagram-v2
    [*] --> IDLE: no install in progress

    IDLE --> VALIDATING: InstallModpack(url, id) called
    VALIDATING --> DOWNLOADING: source URL valid, disk space available
    VALIDATING --> FAILED: invalid source or insufficient space

    DOWNLOADING --> VERIFYING: all files downloaded
    DOWNLOADING --> FAILED: download error (network, checksum mismatch)

    VERIFYING --> EXTRACTING: checksums pass
    VERIFYING --> FAILED: checksum mismatch

    EXTRACTING --> CONFIGURING: files extracted to instance directory
    EXTRACTING --> FAILED: extraction error

    CONFIGURING --> COMPLETE: Prism instance configured
    CONFIGURING --> FAILED: configuration error

    COMPLETE --> IDLE: InstallComplete signal emitted
    FAILED --> IDLE: InstallFailed signal emitted, partial files cleaned up
```

**Progress signals:** `InstallProgress(id, percent, stage, bytes_done, bytes_total)` emitted at max 2/s during DOWNLOADING and EXTRACTING stages.

**Invariant:** On FAILED, all partially-written files must be removed. The instance directory must either be complete and valid, or absent. No partial state.

---

## Diagram Maintenance

When a state machine changes due to new requirements or implementation discoveries:

1. Update the diagram in this file first.
2. Update the corresponding prose in `docs/session-model.md` or `docs/runtime.md`.
3. Reference this file in the PR description.
4. If the change affects an Accepted ADR, open a new ADR.

The diagrams in this file are authoritative. Implementation that diverges from them is a bug, not the diagram.
