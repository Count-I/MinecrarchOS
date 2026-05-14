# MinecrarchOS — Runtime Definition

## Overview

The MinecrarchOS runtime is the layer responsible for launching, supervising, containing, and recovering the Minecraft process. It sits between the `services/` layer (which coordinates launches via D-Bus) and the Minecraft process itself.

The core principle: **Minecraft always runs as a systemd transient unit**. It is never exec'd directly from a shell or service binary. This ensures it always has a managed cgroup scope, a predictable lifecycle observable by systemd, and clean termination behavior.

---

## Process Model

### Launch Sequence

```text
Shell ──D-Bus──► ModpackManager.LaunchInstance(id)
                        │
                        ▼
               ModpackManager (service)
               - validate instance
               - prepare working directory
               - resolve runtime environment (java binary, libraries, etc.)
               - hand off to runtime layer
                        │
                        ▼
               systemd-run --user \
                 --scope \
                 --unit=minecrarch-game@{id}.scope \
                 --slice=minecrarch-game.slice \
                 --property=CPUWeight=800 \
                 --property=MemoryMax=80% \
                 -- <game executable> [args]
                        │
                        ▼
               Minecraft process
               - PID assigned to cgroup scope
               - Wayland surface registered with Gamescope
               - Runtime.GameStarted(id, pid) emitted via D-Bus
```

### Cgroup Topology

```text
user.slice
└── minecrarch.slice               (all MinecrarchOS processes)
    ├── minecrarch-services.slice  (runtime services)
    │   ├── minecrarch-modpack-manager.service
    │   ├── minecrarch-overlay.service
    │   ├── minecrarch-updater.service
    │   └── minecrarch-logging.service
    ├── minecrarch-shell.service   (the shell)
    └── minecrarch-game.slice      (game processes — isolated)
        └── minecrarch-game@{id}.scope   (transient, one per game launch)
```

The game slice is separate from the services slice. This enables:
- Resource limits applied specifically to the game without affecting services.
- OOM killer priority: services should survive before the game if memory is constrained.
- Clean accounting: `systemctl status minecrarch-game.slice` shows game resource usage independently.

---

## Resource Management

### CPU

```ini
# minecrarch-game.slice
[Slice]
CPUWeight=800          # game gets ~80% of CPU weight vs. other slices
```

The shell and services run with default CPU weight. On linux-zen, the scheduler (CFS with zen tuning) already favors interactive/gaming workloads; the CPUWeight amplifies this for the game process specifically.

### Memory

```ini
# minecrarch-game@{id}.scope (applied via systemd-run --property)
MemoryMax=80%          # hard cap — OOM kills game before system
MemoryHigh=70%         # soft cap — triggers memory reclaim before hitting hard cap
OOMScoreAdjust=300     # game is preferred OOM victim over system processes
```

The remaining 20% of RAM is reserved for the shell, services, and kernel. On a system where MinecrarchOS is the only workload, this is a safe default that prevents a misbehaving game from OOM-killing the session.

### I/O

linux-zen ships with BFQ as the default I/O scheduler. No additional configuration is needed. BFQ provides per-process I/O fairness with latency guarantees, which prevents Minecraft's asset loading I/O from starving the shell and services.

### Wayland / GPU

The GPU is managed entirely by Gamescope. GPU resource allocation is implicit — Gamescope prioritizes the game surface for rendering. No cgroup-level GPU controls are applied (cgroup GPU controllers are not yet mature in mainline Linux).

---

## Process Supervision

### Crash Detection

The runtime monitors the game process via the systemd scope's state. When the game process exits, systemd marks the transient scope as failed (if exit code ≠ 0 or killed by signal). The runtime service subscribes to systemd's D-Bus interface for unit state changes:

```text
org.freedesktop.systemd1.Manager.SubscribeToUnitState
→ watch for minecrarch-game@{id}.scope PropertiesChanged
→ ActiveState: "failed" triggers crash handling
→ ActiveState: "inactive" with exit code 0 triggers clean exit handling
```

On crash detection, the runtime service emits `org.minecrarch.ModpackManager.GameCrashed(id, exit_code, signal_name)` on the D-Bus session bus.

### Signal Mapping

| Exit condition | systemd ActiveState | Signal emitted |
|---|---|---|
| Clean exit (exit 0) | `inactive` | `GameExited(id, 0)` |
| Non-zero exit | `failed` | `GameCrashed(id, exit_code, "")` |
| Killed by SIGSEGV | `failed` | `GameCrashed(id, -1, "SIGSEGV")` |
| Killed by SIGKILL | `failed` | `GameCrashed(id, -1, "SIGKILL")` |
| OOM killed | `failed` | `GameCrashed(id, -1, "OOM")` |

The runtime reads `/proc/{pid}/status` or the cgroup exit event to determine whether the kill was OOM-triggered.

### Watchdog

For hung game processes (process alive but unresponsive — common in Java out-of-memory soft-lock scenarios):

```text
Runtime service:
  Every 30s: check that the game's Wayland surface is still rendering
  (query Gamescope for last frame timestamp on game surface)
  If no frame rendered for > 60s while game process is alive:
    → consider the game hung
    → emit GameCrashed(id, -1, "WATCHDOG")
    → send SIGKILL to game scope
```

The frame timestamp check uses Gamescope's socket IPC. An alternative (simpler) implementation for Phase 1: check that the game process is not in D state (uninterruptible sleep) for more than 60s via `/proc/{pid}/status`.

---

## Termination

### Orderly Termination (user-initiated quit)

```text
1. User selects "Quit Game" in shell overlay
2. Shell sends SIGTERM to the game's cgroup scope:
   systemctl kill --user --kill-who=all --signal=SIGTERM minecrarch-game@{id}.scope
3. Wait up to 10 seconds for clean exit (GameExited signal)
4. If timeout: SIGKILL
   systemctl kill --user --kill-who=all --signal=SIGKILL minecrarch-game@{id}.scope
5. Transient scope is automatically cleaned up by systemd
```

### Forced Termination (suspend / shutdown)

During suspend or shutdown (see session-model.md), the runtime must terminate the game before the system suspends:

```text
1. SIGTERM → 5s wait → SIGKILL (tighter timeout than orderly quit)
2. Runtime emits GameExited or GameCrashed as appropriate
3. Releases inhibitor lock (managed at shell level, not runtime level)
```

---

## Bedrock Edition Runtime

### Process Model

Minecraft Bedrock for Linux is distributed via community packaging (mcpelauncher). The actual process tree under `systemd-run`:

```text
minecrarch-game@{id}.scope
└── mcpelauncher-client [args]
    └── Minecraft (Bedrock) native binary
```

mcpelauncher is a compatibility launcher that handles Bedrock's Android-derived runtime. The runtime layer treats the top-level `mcpelauncher-client` process as the game process for supervision purposes.

### Gamepad Input

Bedrock Edition has native gamepad support. No additional input remapping is needed. Gamescope routes raw gamepad events to the Bedrock Wayland surface while in `IN_GAME` state. The shell must not intercept gamepad events intended for the game, except for the reserved system shortcut (Guide/Home long-press → in-game system menu).

---

## Java Edition Runtime (Phase 2+)

Java Edition introduces additional runtime layers. These are not implemented in Phase 1 but must be considered in the runtime architecture to avoid structural rework.

### JVM Management

- JVM selection: multiple JVM versions may be needed (Java 17 for modern Minecraft, Java 8 for legacy). The runtime layer must be able to exec different JVM binaries based on instance configuration.
- Prism Launcher manages JVM selection internally; in Phase 1, this is delegated entirely to Prism. In Phase 2+, the runtime layer may take direct control.

### JVM Tuning

- Heap size: configurable per instance (Xmx/Xms).
- GC tuning: ZGC or G1GC recommended for low pause times on gaming hardware.
- These settings live in instance configuration, not in the runtime layer's global config.

### Process Tree

```text
minecrarch-game@{id}.scope
└── java [JVM args] -jar minecraft.jar [args]
    └── Minecraft Java Edition (main class)
        └── (optional) mod loader init (Fabric/Forge/NeoForge)
```

### Modloader Considerations

The runtime layer must wait for modloader initialization to complete before considering the game "running". A game that exits during modloader init should be treated as a launch failure, not a crash, and reported with the modloader log as the relevant context.

---

## Instance Isolation

Each game instance runs in its own transient scope (`minecrarch-game@{id}.scope`). Two instances cannot run simultaneously in the current design — `LaunchInstance()` must fail if any game scope is currently active.

Future work (Phase 2+) may explore:
- Running two instances simultaneously in separate Gamescope sub-sessions.
- Namespace-based filesystem isolation for instance data directories.

---

## Runtime Error Taxonomy

| Error | Category | Recovery |
|---|---|---|
| Game process never started | Launch failure | Show error in recovery UI |
| Clean exit (exit 0) | Normal | Return to MENU |
| Non-zero exit within 10s of launch | Launch failure | Recovery UI with launch logs |
| Non-zero exit after stable running | Crash | Recovery UI with crash logs |
| OOM killed | Crash (OOM) | Recovery UI with OOM warning |
| Watchdog timeout | Crash (hang) | Recovery UI with hang warning |
| SIGKILL from user | Normal termination | Return to MENU |
