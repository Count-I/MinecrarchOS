# Minecrarch-OS

> A console-like Linux gaming platform built entirely around Minecraft.

<p align="center">
  <img src="./assets/banner.png" alt="Minecrarch-OS Banner" width="100%">
</p>

<p align="center">
  <a href="./LICENSE">
    <img src="https://img.shields.io/badge/license-TBD-blue.svg" alt="License">
  </a>
  <a href="./docs/roadmap.md">
    <img src="https://img.shields.io/badge/status-architecture_phase-orange.svg" alt="Status">
  </a>
  <a href="./docs/contributing.md">
    <img src="https://img.shields.io/badge/contributions-welcome-brightgreen.svg" alt="Contributions">
  </a>
</p>

---

# What is Minecrarch-OS?

Minecrarch-OS is an open source Linux gaming platform inspired by console-like experiences such as SteamOS, but fully centered around Minecraft as the primary workload of the system.

This is **NOT**:

- a Linux rice,
- a fullscreen launcher on top of a desktop,
- another generic gaming distro,
- a collection of startup scripts,
- a Minecraft launcher skin.

This **IS**:

- a dedicated gaming appliance platform,
- a controller-first Minecraft ecosystem,
- a Linux-native gaming runtime,
- a fullscreen integrated experience,
- a modular open source platform.

---

# Vision

Minecrarch-OS aims to make Minecraft feel like part of the operating system itself.

The intended experience is:

```text
Power On
  ↓
Bootloader
  ↓
Linux Kernel
  ↓
systemd
  ↓
Dedicated Wayland Session
  ↓
Minecrarch Shell
  ↓
Minecraft
```

Not:

```text
Desktop Environment
  ↓
Launcher App
  ↓
Minecraft
```

---

# Philosophy

Minecrarch-OS combines two core ideas:

## Console-Like Experience

- Fullscreen-first UX
- Gaming-oriented session model
- Stable and deterministic runtime
- Controller-first interaction
- Fast boot-to-game flow
- Minimal friction
- Appliance-style operation

## Real Open Source Platform

- Modular architecture
- Hackable internals
- Linux-native technologies
- Extensible services
- Transparent design
- Community contribution friendly
- Maintainable long-term structure

---

# Core Architecture Direction

Minecrarch-OS is designed as:

> A single-purpose gaming appliance platform.

Linux is treated as the infrastructure layer.

Minecraft is treated as a privileged workload.

---

# Planned Stack

| Layer               | Technology                 |
| ---------------------| ----------------------------|
| Base System         | Arch Linux                 |
| Kernel              | linux-zen                  |
| Init System         | systemd                    |
| Bootloader          | systemd-boot               |
| Filesystem          | btrfs                      |
| Graphics            | Wayland                    |
| Session Compositor  | Gamescope                  |
| Launcher Backend    | Prism Launcher (initially) |
| Session Model       | Dedicated Gaming Session   |
| Desktop Environment | None                       |

---

# System Overview

```text
UEFI
  ↓
systemd-boot
  ↓
linux-zen
  ↓
systemd
  ↓
Autologin Session
  ↓
Gamescope Session
  ↓
Minecrarch Shell
  ↓
Minecrarch Runtime Services
  ↓
Minecraft Runtime
```

---

# Minecrarch Shell

Minecrarch Shell is not simply a launcher UI.

It is the orchestration layer of the platform.

## Responsibilities

- User experience
- Navigation
- Session lifecycle
- Runtime orchestration
- Game launching
- Overlay management
- Recovery flows
- State coordination
- Controller UX

## Non-Responsibilities

The shell should **NOT**:

- manage Java internals,
- install modloaders directly,
- parse Mojang manifests,
- handle drivers,
- become a monolithic backend.

Those responsibilities belong to dedicated services.

---

# Design Goals

## Stability

The system should recover gracefully from:

- game crashes,
- launcher failures,
- suspend/resume events,
- compositor restarts,
- runtime interruptions.

---

## Modularity

Everything should be replaceable and isolated.

The platform should evolve without massive rewrites.

---

## Controller-First UX

Minecrarch-OS is primarily designed for:

- gamepads,
- couch gaming,
- fullscreen navigation,
- appliance-style interaction.

Keyboard and mouse remain secondary at the platform UX level.

This decision is not a rejection of Minecraft Java Edition.

Quite the opposite.

The long-term vision for Minecrarch-OS heavily includes:
- deep Java Edition support,
- modded ecosystems,
- custom runtimes,
- integrated mod management,
- launcher orchestration,
- instance isolation,
- potentially CurseForge-like or alternative integrated ecosystems.

Minecraft Java Edition is expected to become one of the core pillars of the platform over time.

However, from a platform engineering perspective, Bedrock currently offers a much faster path toward building the foundations of the operating system itself.

The reason is architectural pragmatism.

Bedrock naturally aligns better with:
- controller-first interaction,
- fullscreen appliance workflows,
- console-like UX,
- gamepad navigation,
- couch gaming scenarios,
- deterministic session behavior.

This allows Minecrarch-OS to focus first on solving the hardest platform-level problems:

- session ownership,
- compositor lifecycle,
- fullscreen orchestration,
- input routing,
- suspend/resume behavior,
- game recovery flows,
- overlay systems,
- runtime isolation,
- controller navigation,
- appliance-style UX consistency.

Those problems exist independently of Java Edition itself.

Starting with Bedrock allows the project to validate the operating system architecture before tackling the significantly larger complexity of the Java ecosystem.

Because Java Edition introduces additional layers such as:
- Java runtime management,
- modloader orchestration,
- modpack dependency resolution,
- JVM tuning,
- asset synchronization,
- compatibility management,
- sandboxing concerns,
- large-scale instance management.

Minecrarch-OS therefore treats Bedrock as the most practical starting point for platform maturation — not as the final destination.

The long-term ambition remains a deeply integrated Minecraft platform where Java Edition, modded gameplay, and advanced runtime orchestration become first-class citizens of the operating system.

---

## Performance Consistency

The system should prioritize:

- low input latency,
- stable frametimes,
- predictable session behavior,
- proper fullscreen ownership.

---

# Repository Structure (Planned)

```text
minecrarch-os/
├── assets/
├── branding/
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── roadmap.md
│   ├── contributing.md
│   ├── session-model.md
│   ├── runtime.md
│   └── ipc.md
├── shell/
├── services/
├── runtime/
├── packaging/
├── scripts/
├── infra/
├── iso/
├── tools/
└── tests/
```

---

# Documentation

## Architecture

- [System Architecture](./docs/architecture/README.md)
- [Session Model](./docs/session-model.md)
- [IPC Strategy](./docs/ipc.md)
- [Runtime Architecture](./docs/runtime.md)

## Architecture Decision Records (ADR)

- [ADR Index](./docs/adr/README.md)

## Project Governance

- [Roadmap](./docs/roadmap.md)
- [Contributing Guide](./docs/contributing.md)
- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [License](./LICENSE)

---

# Development Status

Minecrarch-OS is currently in:

# Phase 1 — Prototype Runtime

Current focus areas:

- Gamescope session (archiso, autologin, no DE)
- Minecrarch Shell scaffold (Rust, GTK4, gamepad navigation)
- ModpackManager D-Bus service stub
- Controller navigation and focus traversal
- Recovery handling (crash detection, recovery UI)

Phase 0 (Formal Architecture Design) is complete. Full architecture documentation is available in [docs/](./docs/).

---

# Planned Phases

## Phase 0 — Architecture ✅

- Formal architecture
- Session model
- IPC strategy
- Runtime definition
- Service boundaries
- Project organization

## Phase 1 — Prototype Runtime 🚧

- Dedicated Gamescope session
- Minimal shell
- Prism integration
- Controller navigation
- Recovery handling

## Phase 2 — Runtime Services

- Runtime manager
- Modpack manager
- Overlay system
- Logging infrastructure
- Update orchestration

## Phase 3 — Distribution Layer

- Installable ISO
- Recovery environment
- Rollback support
- Repository infrastructure
- Public update system

## Phase 4 — Public Platform

- Stable releases
- Community ecosystem
- Plugin architecture
- Long-term maintenance

---

# Non-Goals

Minecrarch-OS is intentionally NOT trying to become:

- a generic Linux desktop,
- another Wayland rice,
- a traditional distro spin,
- a generic Minecraft launcher,
- a mod manager only.

---

# Inspirations

Architectural inspirations include:

- SteamOS
- Gamescope session model
- Console operating systems
- Gaming appliance platforms
- Embedded fullscreen systems

---

# Contributing

Phase 0 is complete. Contributions are now open for Phase 1 work.

Active areas:

- Shell development (Rust, GTK4, Wayland, gamepad input)
- Runtime services (D-Bus, systemd, Prism Launcher integration)
- Linux integration (Gamescope, session lifecycle, cgroups)
- Packaging (PKGBUILD, archiso)
- Documentation and testing infrastructure

See:

- [Contributing Guide](./docs/contributing.md)

---

# License

License is currently undecided.

Potential options include:

- GPLv3
- MIT
- Apache-2.0

Final decision will be made during governance definition.

---

# Project Disclaimer

Minecrarch-OS is currently an architecture and research initiative.

APIs, naming, stack choices, and internal structure may evolve significantly during early development phases.

---

# Future Goals

Long-term goals include:

- Installable ISO
- Immutable deployments
- Atomic updates
- Recovery environments
- Rollback support
- Dedicated repositories
- Plugin ecosystem
- Custom runtime services
- Integrated Minecraft lifecycle management

---

# Contact

Project communication structure is not finalized yet.

Future channels may include:

- GitHub Discussions
- Discord
- Matrix
- Documentation Portal

---

# Status

🚧 Phase 1 — Prototype Runtime