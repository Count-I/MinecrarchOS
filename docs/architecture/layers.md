# MinecrarchOS — Architectural Layers and Dependency Rules

## Layer Map

```text
┌─────────────────────────────────────────────────────┐
│                    shell/                           │  Orchestration + UX
│         Rust · GTK4 · zbus · libinput              │  NO domain logic
└──────────────────────┬──────────────────────────────┘
                       │ D-Bus only (no direct imports)
┌──────────────────────▼──────────────────────────────┐
│                  services/                          │  Domain logic
│  modpack-manager · overlay · logging · updater      │  Each owns its domain
└──────────────────────┬──────────────────────────────┘
                       │ can import shared/; systemd-run for game
┌──────────────────────▼──────────────────────────────┐
│                  runtime/                           │  Process model
│       systemd-run · cgroups · supervision           │  Game lifecycle only
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│                   shared/                           │  Common types only
│           D-Bus types · error types · consts        │  No business logic
└─────────────────────────────────────────────────────┘
```

---

## Workspace Members (Cargo)

Planned Rust workspace layout (`Cargo.toml`):

```text
shell/                    crate: minecrarch-shell
services/
  modpack-manager/        crate: minecrarch-modpack-manager
  overlay/                crate: minecrarch-overlay
  logging/                crate: minecrarch-logging
  updater/                crate: minecrarch-updater
runtime/                  crate: minecrarch-runtime
shared/                   crate: minecrarch-shared
```

---

## Allowed Dependencies

| Component | May depend on | May NOT depend on |
|---|---|---|
| `shell` | `shared` | `services/*`, `runtime` |
| `services/modpack-manager` | `shared`, `runtime` | `shell`, other `services/*` |
| `services/overlay` | `shared` | `shell`, other `services/*`, `runtime` |
| `services/logging` | `shared` | `shell`, other `services/*`, `runtime` |
| `services/updater` | `shared`, `runtime` | `shell`, other `services/*` |
| `runtime` | `shared` | `shell`, `services/*` |
| `shared` | (external crates only) | `shell`, `services/*`, `runtime` |

**Cross-component communication uses D-Bus exclusively.** No Rust crate in `shell/` imports any crate from `services/`. The only shared Rust code is `shared/`, which contains only types, error definitions, and D-Bus type mappings.

---

## Forbidden Imports

The following imports are banned in the specified components:

### In `shell/`

```text
# No direct game process management
std::process::Command (for launching Minecraft)  → use ModpackManager D-Bus call

# No filesystem access to game data
~/.local/share/PrismLauncher/                    → ModpackManager owns this path

# No HTTP clients
reqwest, ureq, hyper (as game download clients)  → ModpackManager owns downloads

# No modpack format parsing
mrpack, curseforge_api                           → ModpackManager owns these
```

### In `services/*` (cross-service)

```text
# No service importing another service's crate
minecrarch-modpack-manager in services/overlay   → communicate via D-Bus
minecrarch-overlay in services/updater           → communicate via D-Bus
```

### In `shared/`

```text
# No business logic, no I/O, no process management
# Only: type definitions, error enums, D-Bus interface types, constants
```

---

## Enforcement Mechanisms

### 1. Cargo workspace isolation

Separate crates cannot import each other unless listed explicitly in `[dependencies]`. The workspace structure makes accidental coupling visible in `Cargo.toml`.

### 2. Dependency boundary checker (`tools/check-deps.sh`)

Runs in CI. Validates that no `Cargo.toml` in `shell/` lists service crates as dependencies. Flags violations as CI failures.

### 3. `cargo-deny` (`deny.toml`)

Enforces license policy and bans specific external crates that would indicate architectural violations (e.g., a Minecraft asset downloader in `shell/`).

### 4. ADR enforcement in CI

PRs touching `shell/`, `services/`, or `runtime/` must reference an ADR or explicitly state none is required. Checked by `.github/workflows/ci.yml`.

### 5. Service CONTRACT.md

Each service maintains a `CONTRACT.md` defining owned responsibilities, non-responsibilities, and IPC contract. PRs that expand service scope without updating `CONTRACT.md` will be rejected in review.

---

## Layer Evolution Rules

- Adding a new service: requires a new ADR, a `CONTRACT.md`, an entry in `docs/ipc.md`, and a systemd unit definition.
- Splitting a service: requires an ADR documenting the new boundary.
- Merging services: requires explicit ADR justification (service merges are rarely correct — investigate the coupling that motivated the merge instead).
- Moving logic from shell to a service: encouraged, does not require an ADR but should be noted in the PR.
- Moving logic from a service to shell: requires ADR justification. This direction is almost always wrong.

---

## infra/ and tools/

```text
infra/         CI infrastructure, container definitions, reproducible build environments
tools/         Developer scripts, check-deps.sh, QEMU helpers, gamepad emulation tools
```

These directories have no dependency restrictions — they are not part of the Rust workspace. They may invoke any tool or script necessary for build, validation, or testing.
