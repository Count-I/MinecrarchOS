# MinecrarchOS — Architectural MVP

## What the MVP Is Not

The MVP is not:
- a polished UI
- a full Minecraft launcher
- a modpack manager
- feature completeness of any kind
- a stable product

## What the MVP Is

The MVP is proof that the platform architecture works end-to-end. It validates the foundation, not the features.

**MVP definition:**

```
boot
  → linux-zen kernel
  → systemd user session
  → Gamescope session (no DE, autologin)
  → Minecrarch Shell alive (Rust, GTK4, fullscreen, gamepad nav)
  → Runtime Services alive on D-Bus
  → IPC operational (shell sends command, service responds)
  → fake game launch (any process: sleep 30, a test binary)
  → fake game crash (kill the process, detect via cgroup exit)
  → GameCrashed D-Bus signal received by shell
  → recovery UI rendered in shell
  → user navigates recovery UI with gamepad
  → user selects restart or return to menu
  → shell returns to stable MENU state
```

The fake game does not need to be Minecraft. It does not need to render anything. A simple `sleep 30` or a purpose-built test binary that exits with a non-zero code is sufficient.

## What This Validates

| Platform Claim | Validated By |
|---|---|
| Gamescope owns the session | Shell runs as Wayland client inside Gamescope |
| Shell is an orchestrator | Shell calls D-Bus, does not exec game directly |
| IPC is operational | Shell sends `LaunchInstance`, receives `GameStarted` and `GameCrashed` |
| systemd owns process lifecycle | Game runs as transient scope via `systemd-run` |
| Crash recovery works | `GameCrashed` signal → RECOVERING state → recovery UI |
| Gamepad navigation works | Recovery UI navigable without keyboard |
| Service fault isolation | Crashing ModpackManager does not crash the shell |
| Session returns to stable state | Shell in MENU after recovery, ready for relaunch |

## What the MVP Does Not Validate

- Minecraft compatibility
- Modpack installation
- Prism Launcher integration
- Update/rollback
- Overlay system
- Multi-instance support
- Java Edition runtime
- Controller UX completeness (only recovery UI needs to work)
- Production-quality error messages

## MVP Success Criteria

The MVP is complete when:

1. The system boots to Gamescope session without a desktop environment.
2. The Minecrarch Shell appears fullscreen in Gamescope within 10 seconds of boot.
3. All four services are registered on the D-Bus user session bus (`busctl --user list | grep minecrarch` shows all four).
4. The shell's "Launch Game" button sends `LaunchInstance` via D-Bus and receives `GameStarted`.
5. A fake game process runs as a systemd transient scope (visible in `systemctl --user status`).
6. Killing the fake game process causes the shell to receive `GameCrashed` and enter RECOVERING state.
7. The recovery UI renders and is navigable with a gamepad (or gamepad emulation via uinput).
8. Selecting "Return to Menu" returns the shell to MENU state.
9. The shell does not crash when ModpackManager is killed and restarted by systemd.

## MVP Test Procedure

```bash
# 1. Boot the system (or test in QEMU)
# 2. Verify Gamescope session
systemctl --user status gamescope-session.service

# 3. Verify all services registered
busctl --user list | grep org.minecrarch

# 4. Verify shell is running
systemctl --user status minecrarch-shell.service

# 5. Trigger launch via D-Bus (manual test, or use shell UI)
busctl --user call org.minecrarch.ModpackManager \
  /org/minecrarch/ModpackManager \
  org.minecrarch.ModpackManager \
  LaunchInstance "s" "test-instance"

# 6. Verify game scope exists
systemctl --user list-units 'minecrarch-game*'

# 7. Kill the fake game process
systemctl --user kill minecrarch-game@test-instance.scope

# 8. Observe shell transitions to RECOVERING state (visually)

# 9. Navigate recovery UI with gamepad and select "Return to Menu"

# 10. Verify shell is in MENU state (no crash, no freeze)
systemctl --user status minecrarch-shell.service

# 11. Kill ModpackManager, verify shell degrades gracefully
systemctl --user stop minecrarch-modpack-manager.service
# Shell should show degraded state, not crash
# ModpackManager restarts automatically

# 12. Verify ModpackManager restart restores shell to HEALTHY state
systemctl --user start minecrarch-modpack-manager.service
```

## Relationship to Phase 1

The MVP is the exit criteria for Phase 1. Phase 1 is not complete until the MVP is validated on real hardware or in a QEMU VM with the full stack.

Phase 1 may include additional features (basic modpack listing, better UI, etc.) but none of that matters unless the MVP passes first.

## Fake Game Binary

For MVP testing, a minimal fake game is needed. It should:
- Start as a Wayland client (or just a process — Wayland surface optional for MVP)
- Run for a configurable duration
- Exit with exit code 0 (normal) or non-zero (crash) based on a flag

A minimal Rust or bash implementation in `tools/fake-game/` is sufficient for Phase 1 testing.
