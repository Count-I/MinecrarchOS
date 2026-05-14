# Service Contract: Overlay

**D-Bus bus name:** `org.minecrarch.Overlay`
**Object path:** `/org/minecrarch/Overlay`
**Interface:** `org.minecrarch.Overlay`
**IPC reference:** [`docs/ipc.md — Overlay`](../../docs/ipc.md)

---

## Owned Responsibilities

- Rendering all HUD overlays above the game surface using `wlr-layer-shell`.
- Transient notifications: appear, display, auto-dismiss.
- Crash overlay: persistent display until the shell transitions to RECOVERING and takes over.
- In-game system menu: interactive overlay surface that captures gamepad input when active.
- `SystemMenuAction` signal emission when user interacts with the system menu.
- `Visible` and `HasInputFocus` D-Bus property management.

## Explicit Non-Responsibilities

This service must NOT:

- Own or transition session state (belongs to the shell).
- Decide when to show overlays (the shell or other services trigger overlays via D-Bus methods).
- Implement game-launching logic.
- Access the game process directly.
- Own input focus in any state other than when an interactive overlay (system menu) is explicitly active.
- Render the main shell UI (the shell renders its own GTK4 surface; the overlay service only renders overlay surfaces above the game).

## Lifecycle

**Startup:**
1. Initialize Wayland connection to Gamescope session.
2. Create `wlr-layer-shell` overlay surface (initially invisible, zero size or zero opacity).
3. Register `org.minecrarch.Overlay` on the user session bus.
4. Ready to receive method calls.

**Shutdown:**
1. Destroy Wayland surfaces.
2. Exit cleanly.

**Restart policy:** `Restart=on-failure` with `RestartSec=1s`. The overlay service is non-critical — the shell degrades gracefully when it is unavailable. The shell must not block on overlay responses.

## Failure Behavior

- If Gamescope Wayland surface cannot be created: service logs the error to journald and retries every 5s. It does not crash; it reports `Visible=false` and all method calls return without error (no-op).
- If `wlr-layer-shell` is unavailable (compositor does not support it): log and enter no-op mode. All overlays silently fail but do not return errors to callers.
- The shell must treat all overlay calls as fire-and-forget. An overlay failure must never block shell state transitions.

## Input Ownership Rules

The overlay surface must NEVER intercept gamepad or pointer input unless `ShowSystemMenu` has been called and the system menu is actively displayed.

When the system menu is active:
- The overlay surface captures all gamepad input.
- The overlay emits `SystemMenuAction` when the user makes a selection.
- Input is returned to the game surface (or shell) only after `HideAll` is called or the system menu is dismissed.

## State Ownership

This service owns:
- The Wayland overlay surface and its visibility state.
- Whether the system menu is currently active.
- The `Visible` and `HasInputFocus` D-Bus properties.

This service does NOT own:
- Session state (the shell owns this).
- Which notification content to show (callers provide content via method arguments).
- Whether the game surface is focused (Gamescope owns this).
