# MinecrarchOS — Claude Context Reference

This document gives Claude the per-component technical context needed to work on MinecrarchOS without requiring the user to re-explain the stack, constraints, or architectural decisions on every prompt. Read alongside CLAUDE.md and the relevant ADR(s) before proposing or implementing anything.

---

## Settled Decisions — Never Contradict These

These are Accepted ADRs. Do not suggest alternatives unless the user explicitly opens the decision.

| Decision | Settled Choice | ADR |
|---|---|---|
| Base system | Arch Linux | ADR-0001 |
| Kernel | linux-zen | ADR-0002 |
| Bootloader | systemd-boot (UEFI only) | ADR-0003 |
| Filesystem | btrfs (with subvolumes) | ADR-0004 |
| Session compositor | Gamescope | ADR-0005 |
| Desktop environment | None | ADR-0006 |
| Initial game target | Minecraft Bedrock Edition | ADR-0007 |
| Launcher backend | Prism Launcher (initial, replaceable) | ADR-0008 |
| Shell model | Orchestration only — no platform logic in the shell | ADR-0009 |
| Primary input | Controller/gamepad — all UI must be fully gamepad-operable | ADR-0010 |
| Shell language | Rust · GTK4 + libadwaita · Cargo | ADR-0011 |
| IPC mechanism | D-Bus (user session bus) · zbus (Rust client) | ADR-0012 |
| Shell UI framework | GTK4 + libadwaita (`gtk4-rs` + `libadwaita-rs`) | ADR-0013 |

**All decisions are settled. No open ADRs remain. Phase 1 implementation can begin.**

---

## Cross-Cutting Rules

Apply these in every component, every prompt:

- **Shell never implements platform logic.** Game launching, modpack management, download management, Java/JVM concerns, manifest parsing — all belong in `services/`. The shell signals services; it does not act directly.
- **All service integration is bounded.** Prism Launcher calls, launcher APIs, and edition-specific logic must live in `services/`, never in `shell/`. The service interface is the only coupling point.
- **Everything must be gamepad-operable.** Any UI element, overlay, or navigation flow proposed for `shell/` must be fully operable without keyboard or mouse. If a UI framework or pattern doesn't support focus traversal, reject it.
- **Java Edition is the long-term goal.** Bedrock is Phase 1 pragmatism, not the destination. When designing `services/` and `runtime/` interfaces, ensure they are edition-agnostic so Java can be added without architectural rework.
- **Components must be replaceable.** Prism Launcher is "initial". Gamescope is chosen, not mandatory forever. Design service interfaces as contracts, not as thin wrappers around a specific tool.
- **UEFI only.** No legacy BIOS support anywhere in the stack.
- **No desktop.** Never suggest adding a DE, a window manager as a fallback, or desktop-style UX patterns.

---

## Component Context

### `shell/`

**What it is:** The user-facing orchestration layer. Renders the fullscreen UI, handles navigation, dispatches commands to services, manages overlays, coordinates recovery flows.

**What it is not:** A launcher, a download manager, a Java runtime manager, a mod installer.

**Tech context:**
- Runs as a Wayland client inside Gamescope's nested compositor session.
- Draws to screen via GTK4 + libadwaita (`AdwApplicationWindow` fullscreen, `AdwNavigationView` for menu stack navigation — ADR-0013). Does not touch DRM/KMS directly.
- Receives gamepad input via the input stack (libinput or SDL2). Must handle d-pad/thumbstick navigation as the primary flow.
- Communicates with `services/` over D-Bus (user session bus, ADR-0012) using `zbus`. Uses D-Bus signals from services to update displayed state asynchronously.
- Session lifecycle events come from systemd (suspend, resume, shutdown) — the shell must respond to these gracefully.
- Overlay surfaces are separate Wayland surfaces layered above the game, rendered by the overlay component.

**Key constraints:**
- Every interactive element must have a defined focus order reachable by gamepad.
- The shell must display recovery UI if a service crashes — it cannot crash with the service.
- No blocking calls in the UI thread. Service communication is async.

**Gamescope specifics:**
- Gamescope exposes a socket-based IPC for shell integration (separate from the standard Wayland protocol). Study Gamescope source before implementing any shell↔compositor feature.
- The shell runs in Gamescope's "overlay" or "main window" role — not as a free-floating Wayland client. This affects surface creation and lifecycle.

---

### `services/`

**What it is:** Long-running background processes that implement platform logic. Four planned services:

1. **Modpack Manager** — Minecraft instance lifecycle, modpack install/update, Prism Launcher integration.
2. **Overlay System** — renders HUD overlays on top of the game surface on behalf of the shell.
3. **Logging Infrastructure** — structured log collection and routing from all platform components.
4. **Update Orchestration** — platform package updates, btrfs snapshot management, rollback safety.

**Tech context:**
- Each service is a separate process, managed by systemd.
- Services expose their interface over D-Bus (user session bus, ADR-0012). Each service registers a well-known bus name (e.g., `org.minecrarch.ModpackManager`).
- Services must not depend on the shell being alive — they are independent processes.
- Prism Launcher integration: Prism has a CLI mode and a known instance directory format (`~/.local/share/PrismLauncher/`). Interaction should be via CLI or file-system-level, not GUI automation.
- Update Orchestration uses btrfs snapshots: `btrfs subvolume snapshot` before any update, verify, then commit or rollback. The pacman libalpm library or pacman CLI is used for package operations.

**Key constraints:**
- Service interfaces are contracts. Do not design a service API that is inherently Prism-specific or Bedrock-specific — it must be replaceable.
- A service crash must not crash the shell. systemd RestartPolicy must be configured for each service.
- Logging service receives logs from all other services — it must be started first in the systemd dependency chain.

---

### `runtime/`

**What it is:** Defines the process model and resource containment for the Minecraft runtime — how the game process is launched, supervised, contained, and recovered.

**Tech context:**
- Minecraft (Bedrock or Java) runs as a systemd transient unit launched via `systemd-run`. This gives it a cgroup scope for resource management and clean lifecycle tracking.
- cgroups v2: resource limits (CPU shares, memory limits) are set on the game's cgroup scope. MinecrarchOS can apply gaming-profile resource limits here.
- Process supervision: crash detection via exit code and signal; recovery flows are coordinated back to the shell via IPC.
- `sd_notify` WATCHDOG can be used to detect a hung game process that hasn't crashed but isn't responding.
- Namespace isolation (PID, mount) may be used for instance isolation — this is a future decision, not yet settled.

**Key constraints:**
- Game launch is initiated by the shell signaling a service, which signals the runtime — never a direct shell → game exec.
- Recovery from a game crash must return the user to the shell UI cleanly. The shell must handle the service notification for "game exited unexpectedly."

---

### `packaging/`

**What it is:** Builds MinecrarchOS as installable Arch-based packages and a distributable ISO.

**Tech context:**
- ISO build uses `archiso` (`mkarchiso`). Reference the official `releng` profile as the starting point.
- All MinecrarchOS-specific packages are written as PKGBUILDs. Build with `makepkg`; lint with `namcap`.
- Custom pacman repository: `repo-add` to build the database; packages and database must be GPG-signed.
- systemd-boot ESP layout: loader entries reference linux-zen vmlinuz and initramfs paths (not default `linux` paths).
- btrfs partition setup: at minimum subvolumes `@` (root) and `@home`; a `@snapshots` subvolume is expected for Phase 3 rollback.

**Key constraints:**
- Never suggest GRUB or syslinux — systemd-boot is the settled bootloader (ADR-0003).
- Never suggest ext4 or XFS for the root partition — btrfs is the settled filesystem (ADR-0004).
- linux-zen, not linux or linux-lts, must be the kernel in all ISO and PKGBUILD configurations.

---

### `iso/`

**What it is:** archiso profile configuration for the live/install ISO.

**Tech context:**
- `profiledef.sh`: defines ISO name, architecture, bootmodes.
- `airootfs/`: overlay applied to the live filesystem — place systemd unit overrides, config files here.
- Autologin: override `getty@tty1.service` to auto-login the `minecrarch` user; configure PAM accordingly.
- Gamescope session must start via a systemd user service or `~/.profile` invocation — not a display manager.
- mkinitcpio preset must target linux-zen, not linux.

---

### `infra/`

**What it is:** CI/CD pipelines and build infrastructure.

**Tech context:**
- GitHub Actions for CI. ISO builds and package builds should run in a clean Arch Linux container (Docker or Podman) to ensure reproducibility.
- GPG signing: packages and the pacman repository database must be signed. Key management must be documented.
- Artifact output: ISO files hosted via GitHub Releases or S3-compatible storage.

---

### `tools/`

**What it is:** Development helper scripts and tooling for the local development workflow.

**Tech context:**
- Bash scripts with `set -euo pipefail`.
- QEMU/KVM for running the full MinecrarchOS stack locally: boot the ISO, test the session model. Use VNC or Wayland passthrough for display.
- Gamescope can be tested headless via `wlheadless` or Xvfb for CI scenarios.

---

### `tests/`

**What it is:** Integration and system test infrastructure.

**Tech context:**
- QEMU/KVM for full-stack integration tests: boot the OS, exercise the session lifecycle, assert on behavior.
- `systemd-nspawn` for lightweight service isolation tests (test a single service without a full VM).
- Gamepad emulation via `uinput` kernel module and `python-evdev` for synthesizing controller events in tests.
- IPC mock/stub tooling for testing shell and services independently across the IPC boundary.

---

### `docs/`

**What it is:** Architecture documentation, ADRs, and contributor guides.

**When editing docs:**
- New architectural decisions → new ADR in `docs/adr/` following the format in `docs/adr/README.md`. Number sequentially from 0013 onward.
- Decisions that overturn an Accepted ADR → new ADR with status Superseded referencing the old one; update the old ADR's status field.
- `docs/ipc.md`, `docs/session-model.md`, `docs/runtime.md` are planned but not yet created — they should be written once ADR-0011 and ADR-0012 are resolved.
- Use Mermaid for inline diagrams (sequence diagrams, flowcharts). GitHub renders Mermaid in Markdown natively.

---

## Boot Sequence Reference

When reasoning about where something belongs or what can fail at what point:

```text
UEFI
  → systemd-boot           # loads linux-zen vmlinuz + initramfs
  → linux-zen              # kernel init
  → systemd (PID 1)        # unit activation begins
  → autologin              # getty override, PAM autologin
  → Gamescope session      # systemd user service; owns the Wayland display
  → Minecrarch Shell       # Wayland client inside Gamescope; orchestration layer
  → Runtime Services       # systemd user services; background processes
  → Minecraft (Bedrock)    # systemd transient unit via systemd-run
```

Each layer is a potential failure point with its own recovery path. When proposing recovery flows, trace which layer failed and what the layer above it can observe and do.

---

## ADR Quick Links

- [ADR-0001](./adr/0001-base-system-arch-linux.md) — Arch Linux
- [ADR-0002](./adr/0002-kernel-linux-zen.md) — linux-zen
- [ADR-0003](./adr/0003-bootloader-systemd-boot.md) — systemd-boot
- [ADR-0004](./adr/0004-filesystem-btrfs.md) — btrfs
- [ADR-0005](./adr/0005-session-compositor-gamescope.md) — Gamescope
- [ADR-0006](./adr/0006-no-desktop-environment.md) — No desktop
- [ADR-0007](./adr/0007-bedrock-edition-initial-target.md) — Bedrock first
- [ADR-0008](./adr/0008-launcher-backend-prism.md) — Prism Launcher
- [ADR-0009](./adr/0009-shell-as-orchestration-layer.md) — Shell boundary
- [ADR-0010](./adr/0010-controller-first-ux.md) — Controller-first UX
- [ADR-0011](./adr/0011-shell-implementation-language.md) — Shell language: Rust
- [ADR-0012](./adr/0012-ipc-mechanism.md) — IPC: D-Bus (user session bus)
- [ADR-0013](./adr/0013-shell-ui-framework-gtk4-libadwaita.md) — Shell UI: GTK4 + libadwaita
