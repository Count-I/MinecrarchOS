# MinecrarchOS — Session Model

## Overview

The MinecrarchOS session is a single Gamescope Wayland session that persists for the entire uptime of the system. There is no login screen, no desktop, and no session switching. The session starts on boot and ends on shutdown.

Within this persistent session, the platform transitions through defined states driven by user actions, service events, and system signals (suspend, shutdown, crash). All state transitions are coordinated by the Minecrarch Shell via D-Bus.

---

## Session State Machine

```text
                    ┌─────────┐
                    │ BOOTING │  (systemd bringing up services)
                    └────┬────┘
                         │ all services registered on D-Bus
                    ┌────▼────────┐
                    │ INITIALIZING│  (shell connecting to services)
                    └────┬────────┘
                         │ shell UI ready
                    ┌────▼────┐
               ┌───►│  MENU   │◄──────────────────────┐
               │    └────┬────┘                        │
               │         │ user selects game            │ user quits / GameExited
               │    ┌────▼──────┐                      │
               │    │ LAUNCHING │                      │
               │    └────┬──────┘                      │
               │         │ GameStarted signal     ┌────┴────────┐
               │    ┌────▼────┐   GameCrashed ───►│  RECOVERING │
               │    │ IN_GAME │   signal            └────┬────────┘
               │    └────┬────┘                         │ user: restart
               │         │ PrepareForSleep               └──────────────►LAUNCHING
               │    ┌────▼──────┐
               │    │  PAUSED   │
               │    └────┬──────┘
               │         │ system resumed
               └─────────┘ (return to MENU or resume game)
```

### States

| State | Description |
|---|---|
| `BOOTING` | systemd starting system and user services. Shell not yet running. |
| `INITIALIZING` | Shell process started. Connecting to all services via D-Bus. Verifying service health. |
| `MENU` | Shell UI active and visible in fullscreen. No game running. User is navigating. |
| `LAUNCHING` | Game launch in progress. Shell shows launch screen. Waiting for `GameStarted` signal. |
| `IN_GAME` | Game running. Shell moves to overlay mode. Gamescope routes input to game. |
| `PAUSED` | System suspend in progress. Game is stopped or saved. Shell holds systemd inhibitor lock. |
| `RECOVERING` | Game exited unexpectedly. Shell has regained focus. User sees recovery UI. |

### Transitions

| From | To | Trigger |
|---|---|---|
| `BOOTING` | `INITIALIZING` | Shell service unit started by systemd |
| `INITIALIZING` | `MENU` | All required D-Bus services available and shell UI rendered |
| `MENU` | `LAUNCHING` | User selects a game/instance; shell calls `ModpackManager.LaunchInstance()` |
| `LAUNCHING` | `IN_GAME` | D-Bus signal: `ModpackManager.GameStarted(id, pid)` |
| `LAUNCHING` | `RECOVERING` | D-Bus signal: `ModpackManager.GameCrashed(id, ...)` or timeout |
| `IN_GAME` | `RECOVERING` | D-Bus signal: `ModpackManager.GameCrashed(id, exit_code, signal)` |
| `IN_GAME` | `MENU` | D-Bus signal: `ModpackManager.GameExited(id, 0)` (clean exit) |
| `IN_GAME` | `PAUSED` | logind `PrepareForSleep` signal received |
| `PAUSED` | `MENU` | System resumed; game was terminated before suspend |
| `PAUSED` | `IN_GAME` | System resumed; game was paused (SIGSTOP) and is resumed (SIGCONT) |
| `RECOVERING` | `MENU` | User dismisses recovery UI |
| `RECOVERING` | `LAUNCHING` | User chooses "Restart Game" in recovery UI |

---

## Wayland Session Architecture

### Surface Hierarchy

Gamescope manages a surface stack. The shell and game both run as Wayland clients within the same Gamescope session.

```text
Gamescope (compositor — owns DRM/KMS)
│
├── Game Surface          (Minecraft Wayland window)
│   └── Owns input when IN_GAME state
│
├── Shell Surface         (Minecrarch Shell — GTK4 fullscreen window)
│   └── Owns input when MENU / RECOVERING / LAUNCHING states
│   └── Hidden (z-order below game) when IN_GAME
│
└── Overlay Surface       (wlr-layer-shell — rendered by Overlay service)
    └── Always on top — HUD notifications, system status
    └── Input passthrough when not actively capturing (e.g., in-game HUD)
```

### Surface Lifecycle

**MENU state:**
- Shell surface is fullscreen and has Wayland keyboard/pointer/gamepad focus.
- Game surface does not exist.
- Overlay surface exists but is idle (no active notifications).

**LAUNCHING state:**
- Shell surface still has focus, showing launch screen.
- Game process started but Wayland surface not yet registered.
- Overlay may show "Launching…" notification.

**IN_GAME state:**
- Game surface registered with Gamescope and receives input focus.
- Shell surface is sent to the back of the z-order (still rendered, not destroyed).
- Overlay surface remains on top — shell can push notifications via D-Bus to Overlay service.
- Shell receives no direct input — only D-Bus signals from services.

**Transition IN_GAME → MENU/RECOVERING:**
- Game surface destroyed (process exited).
- Gamescope routes input focus back to the shell surface.
- Shell surface moves to front.

### Wayland Protocols in Use

| Protocol | Used by | Purpose |
|---|---|---|
| `xdg-shell` + `xdg-toplevel` | Shell, Game | Standard application window surface |
| `wlr-layer-shell` | Overlay service | Always-on-top overlay surface for HUD |
| `xdg-output` | Shell | Query display resolution and geometry |
| Gamescope socket IPC | Shell | Shell↔compositor coordination (focus, surface hints) |

The Gamescope socket IPC is separate from the standard Wayland protocol. It provides mechanisms for the shell to signal Gamescope about surface priority, HDR preferences, and frame pacing hints. Study Gamescope source (`src/steamcompmgr.cpp`) before implementing this integration.

---

## Service Startup and Ordering

systemd dependency ordering for the user session:

```text
minecrarch-logging.service          (starts first — all others log to it)
        │
        ▼ (After=)
minecrarch-modpack-manager.service  ─┐
minecrarch-overlay.service          ─┤ (start in parallel)
minecrarch-updater.service          ─┘
        │
        ▼ (After=)
minecrarch-shell.service            (starts last — depends on all services)
```

The shell's `INITIALIZING` state explicitly verifies that all required D-Bus bus names are registered before rendering the UI. If a non-critical service (e.g., updater) is unavailable, the shell must start anyway and disable the relevant UI elements rather than failing to start.

**Critical services** (shell cannot start without them):
- `org.minecrarch.ModpackManager` — required for any game launching
- `org.minecrarch.Logging` — required for structured log routing

**Non-critical services** (shell degrades gracefully):
- `org.minecrarch.Overlay` — HUD notifications disabled if unavailable
- `org.minecrarch.Updater` — update UI hidden if unavailable

---

## Suspend and Resume

### Suspend Flow

```text
1. logind emits PrepareForSleep(true) on the system bus
2. Shell receives signal via logind D-Bus subscription
3. Shell acquires a systemd inhibitor lock (delay inhibitor)
4. Shell determines current state:
   - IN_GAME: signal Runtime to SIGSTOP the game process
   - MENU / other: no game process to handle
5. Shell calls ModpackManager.SuspendInstance(id) — flushes game state if possible
6. Shell releases inhibitor lock — system proceeds to suspend
```

### Resume Flow

```text
1. logind emits PrepareForSleep(false) on resume
2. Shell receives signal
3. Shell checks if game process still exists (via pid from GameStarted signal):
   - If alive: send SIGCONT, transition back to IN_GAME
   - If dead: transition to RECOVERING
4. Shell re-renders UI as appropriate
```

The game process may be killed by the kernel during suspend on memory-constrained hardware. The shell must treat a missing game process on resume as a crash, not an error.

---

## Shutdown Flow

```text
1. User triggers shutdown (via shell UI or power button)
2. Shell calls systemd Shutdown via logind D-Bus
3. Before systemd proceeds:
   - Shell acquires inhibitor lock
   - If IN_GAME: orderly game termination (SIGTERM → wait 5s → SIGKILL)
   - Shell tells all services to flush state
   - Shell releases inhibitor lock
4. systemd stops all user units in reverse dependency order
5. System powers off
```

---

## Recovery Flows

### Game Crash Recovery

When `GameCrashed(id, exit_code, signal)` is received:

1. Shell transitions to `RECOVERING`.
2. Shell re-acquires Wayland focus from Gamescope (game surface destroyed automatically when process dies).
3. Overlay service is signaled to show crash notification overlay.
4. Shell renders recovery screen with:
   - Crash reason (exit code / signal name if available)
   - "Restart Game" button
   - "Return to Menu" button
   - Link to crash log (journald entry for the game's cgroup scope)
5. User choice drives transition:
   - Restart → `LAUNCHING`
   - Return to menu → `MENU`

### Service Crash Recovery

If a runtime service (not the game) crashes:

- systemd's `Restart=on-failure` restarts the service automatically.
- The shell detects the service's D-Bus name disappearing (via `NameOwnerChanged` signal on the bus).
- Shell enters a "service degraded" mode: disables UI features that depend on the crashed service, shows a warning overlay, and waits for the service to re-register on D-Bus.
- Once the service re-registers, the shell re-subscribes to its signals and restores full functionality.

The shell must never crash because a service crashed. Service failures are expected events, not exceptional ones.

### Launch Failure

If `GameCrashed` is received while in `LAUNCHING` state (game never started successfully):

1. Shell transitions directly to `RECOVERING`.
2. Recovery UI shows launch failure reason.
3. No SIGCONT/game process cleanup needed (game didn't reach a running state).

---

## Controller Input Routing

Input routing is managed by Gamescope at the compositor level.

| State | Input routed to |
|---|---|
| `MENU` / `INITIALIZING` / `RECOVERING` / `LAUNCHING` | Shell surface (GTK4 handles gamepad focus traversal) |
| `IN_GAME` | Game surface (Minecraft handles gamepad natively) |
| Overlay active during `IN_GAME` | Overlay captures input only if the overlay is an interactive modal (e.g., system menu invoked by a dedicated button) |

The shell must register a gamepad shortcut for opening the in-game system menu (e.g., a long-press of the Guide/Home button) that recaptures focus from the game surface to the overlay/shell layer. This is implemented by intercepting the button event at the libinput level before it reaches the game, using Gamescope's input filtering API.
