# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Extended Context

For per-component technical context, settled decisions, and cross-cutting rules that apply when working on any specific area: read **[`docs/skills.md`](docs/skills.md)** before proposing or implementing anything component-specific.

For the full rationale behind each architectural decision: read the relevant ADR in **[`docs/adr/`](docs/adr/README.md)**.

## Project Status

Minecrarch-OS is currently in **Phase 0 — Formal Architecture Design**. No source code or build system exists yet. The repository contains the README, ADRs (`docs/adr/`), and the Claude context reference (`docs/skills.md`). All source directories described below are planned, not yet created.

## What This Project Is

A dedicated Linux gaming appliance platform built around Minecraft — not a desktop distro, not a launcher skin, not a collection of scripts. The intended boot-to-game flow:

```
UEFI → systemd-boot → linux-zen → systemd → Autologin → Gamescope Session → Minecrarch Shell → Minecraft
```

## Planned Stack

| Layer | Technology |
|---|---|
| Base System | Arch Linux |
| Kernel | linux-zen |
| Bootloader | systemd-boot |
| Filesystem | btrfs |
| Graphics | Wayland |
| Session Compositor | Gamescope |
| Launcher Backend | Prism Launcher (initially) |
| Desktop Environment | None |

## Planned Repository Structure

```
minecrarch-os/
├── shell/         # Minecrarch Shell — orchestration layer (UX, navigation, session lifecycle, overlays)
├── services/      # Runtime services (mod manager, update orchestration, logging, overlay system)
├── runtime/       # Runtime architecture and orchestration
├── packaging/     # ISO build, repository infrastructure
├── scripts/       # Integration and setup scripts
├── infra/         # Infrastructure definitions
├── iso/           # Installable ISO configuration
├── tools/         # Development tooling
├── tests/         # Test infrastructure
├── docs/
│   ├── architecture/   # System architecture docs
│   ├── adr/            # Architecture Decision Records
│   ├── roadmap.md
│   ├── session-model.md
│   ├── runtime.md
│   └── ipc.md
└── assets/        # Branding and visual assets
```

## Architecture Principles

**Minecrarch Shell** is the orchestration layer, responsible for: UX/navigation, session lifecycle, game launching, overlay management, recovery flows, controller UX, and state coordination. It explicitly must NOT handle Java internals, modloader installation, Mojang manifest parsing, or driver management — those belong to dedicated services.

**Bedrock-first, Java long-term**: Bedrock is the initial target because it aligns naturally with the controller-first, fullscreen, appliance-style UX. Java Edition support (with modloader orchestration, JVM tuning, modpack resolution) is the long-term goal once platform foundations are validated.

**Modularity**: Every component must be replaceable and isolated. Service boundaries are strict — the shell does not become a monolithic backend.

**Stability requirements**: The system must recover gracefully from game crashes, launcher failures, suspend/resume events, and compositor restarts.

**Controller-first**: Gamepad is primary input at the platform UX level; keyboard/mouse are secondary.

## Development Phases

- **Phase 0 (current)**: Architecture, session model, IPC strategy, service boundaries
- **Phase 1**: Gamescope session, minimal shell, Prism integration, controller nav, recovery
- **Phase 2**: Runtime manager, modpack manager, overlay system, logging, update orchestration
- **Phase 3**: Installable ISO, recovery environment, rollback, repository infrastructure
- **Phase 4**: Stable releases, plugin architecture, community ecosystem

## Key Design Decisions

- No desktop environment — Gamescope owns the session directly
- Linux is infrastructure; Minecraft is a privileged workload
- Architecture inspiration: SteamOS / console gaming OS model
- License undecided (GPLv3, MIT, or Apache-2.0 under consideration)
- Contributions not yet open during Phase 0
