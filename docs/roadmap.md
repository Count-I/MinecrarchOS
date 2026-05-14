# MinecrarchOS — Roadmap

## Phase 0 — Architecture ✅ Complete

Established the full architectural foundation of the platform.

**Delivered:**
- 12 Architecture Decision Records ([ADR Index](./adr/README.md))
- [System Architecture](./architecture/README.md)
- [Session Model](./session-model.md) — state machine, Wayland strategy, recovery flows
- [Runtime Definition](./runtime.md) — cgroup topology, supervision, crash taxonomy
- [IPC Strategy](./ipc.md) — D-Bus interface contracts for all services

**Key decisions:**
- Stack: Arch Linux + linux-zen + systemd-boot + btrfs + Gamescope
- Shell: Rust + GTK4/libadwaita + D-Bus (zbus)
- Initial game target: Minecraft Bedrock Edition
- No desktop environment

---

## Phase 1 — Prototype Runtime 🚧 In Progress

Validate the platform architecture with a running, playable system.

**Deliverables:**
- [ ] Bootable archiso with Gamescope session (no DE, autologin)
- [ ] Minecrarch Shell scaffold (Rust, GTK4, minimal menu, gamepad nav)
- [ ] ModpackManager service stub (D-Bus, `LaunchInstance`, crash detection)
- [ ] Controller navigation (d-pad focus traversal, system menu shortcut)
- [ ] Recovery handling (`GameCrashed` → recovery UI → restart or menu)

**Not in scope for Phase 1:** modpack installation UI, Update Orchestration service, Overlay service, Java Edition, ISO installer.

---

## Phase 2 — Runtime Services

Full implementation of all four platform services.

**Planned:**
- Modpack Manager (Prism Launcher integration, Modrinth + CurseForge pack formats, download management)
- Overlay System (wlr-layer-shell HUD, in-game system menu)
- Logging Infrastructure (structured journald routing, log level control)
- Update Orchestration (pacman + btrfs snapshot rollback, atomic updates)

---

## Phase 3 — Distribution Layer

MinecrarchOS as an installable product.

**Planned:**
- Installable ISO with graphical/TUI installer
- btrfs subvolume layout (`@`, `@home`, `@snapshots`)
- Atomic update + rollback via systemd-boot entry + btrfs snapshot pairs
- Custom pacman repository
- Recovery boot environment (separate boot entry)

---

## Phase 4 — Public Platform

Stable, community-ready platform.

**Planned:**
- Stable release versioning and release process
- Java Edition support (new ADRs: JVM selection, modloader strategy)
- Plugin/extension architecture
- Community contribution infrastructure
- Long-term maintenance model

---

*For architectural rationale behind each decision, see the [ADR Index](./adr/README.md).*
*For autonomous agent task guidance, see [AGENTS.md](../AGENTS.md).*
