# AGENTS.md

This file instructs autonomous AI agents working on MinecrarchOS. It defines scope of authority, behavioral rules, phase-by-phase objectives, and how agents should interact with the established architecture and documentation.

Read this file before taking any autonomous action. It supersedes general AI coding defaults when they conflict with project-specific rules.

## Primary Directive

Your primary responsibility is **preserving architectural integrity while evolving the platform**. It is NOT rapid feature delivery.

Before implementing any feature:
1. Identify which ADR governs it.
2. Identify which component owns the responsibility.
3. Determine whether the change introduces coupling.
4. Determine whether a new ADR is required.

**If any step is unclear: stop and request clarification. Never improvise architecture.**

Full enforcement mandate: [`docs/architecture/enforcement.md`](docs/architecture/enforcement.md)
Full layer boundary rules: [`docs/architecture/layers.md`](docs/architecture/layers.md)
Canonical state machines: [`docs/state-machines.md`](docs/state-machines.md)
MVP definition and success criteria: [`docs/mvp.md`](docs/mvp.md)
Testing strategy: [`docs/testing-strategy.md`](docs/testing-strategy.md)

---

## Documentation Map

Read these documents before working on any component. They are the authoritative source of truth for this project. The order matters — read top-to-bottom before starting work.

| Document | When to read |
|---|---|
| `CLAUDE.md` | Every session — project overview and extended context pointer |
| `docs/architecture/enforcement.md` | Every session — architectural authority and failure conditions |
| `docs/skills.md` | Before working on any specific component — settled decisions and per-component constraints |
| `docs/architecture/layers.md` | Before any implementation — layer boundaries and forbidden dependencies |
| `docs/architecture/README.md` | Before any cross-component work — system layers, component responsibilities, IPC diagram |
| `docs/state-machines.md` | Before implementing any stateful flow — canonical state machine diagrams |
| `docs/session-model.md` | Before any work on `shell/`, `services/`, or session lifecycle |
| `docs/runtime.md` | Before any work on `runtime/` or game process management |
| `docs/ipc.md` | Before any work involving D-Bus — full interface contracts for all four services |
| `services/*/CONTRACT.md` | Before working on any service — owned responsibilities and explicit non-responsibilities |
| `docs/mvp.md` | Before starting Phase 1 implementation — MVP success criteria |
| `docs/testing-strategy.md` | Before writing any test — test pyramid and strategy per component |
| `docs/adr/README.md` | Before proposing any architectural change — index of all settled decisions |

---

## Authority Model

### Act autonomously

- Implement code that is consistent with settled ADRs, `docs/skills.md`, and the phase objectives below.
- Create, edit, and delete files within the current phase's scope.
- Write tests, documentation updates, and tooling within the established patterns.
- Propose new ADRs in `docs/adr/` with status `Proposed` when you encounter an undecided architectural question. Do not block on it — document the question and proceed with a clearly-stated assumption.
- Update `docs/skills.md` to reflect any new concrete technical decisions that result from implementation work.

### Always confirm with the user before acting

- Changing the status of any ADR from `Proposed` to `Accepted` — that requires explicit user decision.
- Contradicting or overriding any `Accepted` ADR, even temporarily.
- Modifying `docs/ipc.md` interface contracts in a breaking way (changing method signatures, removing signals, renaming properties).
- Changing any component's D-Bus bus name or object path.
- Deleting or restructuring the `docs/` directory.
- Any operation that affects git history (force push, rebase, amend).
- Transitioning the project to the next phase (e.g., declaring Phase 1 complete and starting Phase 2).

---

## Architectural Invariants

These must never be violated. If an implementation path requires violating one, stop and ask the user.

1. **Gamescope owns the display.** No component writes to DRM/KMS directly.
2. **Shell is an orchestration layer.** It signals services; it does not implement platform logic. Java internals, modloaders, manifests, downloads — all in `services/`.
3. **Shell is Rust.** `shell/` is implemented in Rust with GTK4/libadwaita (ADR-0011). Do not introduce another language in `shell/`.
4. **IPC is D-Bus.** Shell↔service communication is D-Bus on the user session bus via `zbus` (ADR-0012). Do not introduce sockets, HTTP, or shared memory for this layer.
5. **Game runs as a systemd transient scope.** Never exec Minecraft directly from a service or the shell. Always use `systemd-run --user --scope`.
6. **All UI is gamepad-navigable.** No interactive element may be accessible only via keyboard or mouse (ADR-0010).
7. **No desktop environment.** Never add a WM, DE, or desktop-style fallback (ADR-0006).
8. **Service interfaces are contracts.** D-Bus interface definitions in `docs/ipc.md` are the formal API boundary. Implementations may evolve; breaking the interface requires user confirmation.
9. **Prism Launcher is bounded.** All Prism-specific code stays in `services/`. The `ModpackManager` D-Bus interface is the only coupling point (ADR-0008).
10. **linux-zen, not linux.** All kernel references, initramfs presets, and boot entries must target `vmlinuz-linux-zen` / `initramfs-linux-zen.img`.

---

## Phase Objectives

### Current Phase: Phase 1 — Prototype Runtime

**Goal:** Validate the platform architecture with a running, playable system. Not production quality — prototype quality with correct architecture.

**Deliverables (in priority order):**

1. **Gamescope session** (`iso/` + `packaging/`):
   - archiso profile with Gamescope as the session (no DE)
   - systemd user unit: `gamescope-session.service`
   - Autologin: getty override unit for `minecrarch` user
   - Bootable ISO with linux-zen, systemd-boot, btrfs root

2. **Minecrarch Shell scaffold** (`shell/`):
   - Rust project (`cargo init`)
   - GTK4 + libadwaita Wayland application
   - Minimal UI: main menu with "Launch Game" button
   - Fully gamepad-navigable (test with `python-evdev` synthetic input)
   - D-Bus connection via `zbus` — connect to services and subscribe to signals

3. **ModpackManager service stub** (`services/modpack-manager/`):
   - Registers `org.minecrarch.ModpackManager` on the user session bus
   - Implements `LaunchInstance()` via `systemd-run`
   - Emits `GameStarted`, `GameExited`, `GameCrashed` signals
   - Crash detection via systemd scope state subscription

4. **Controller navigation** (`shell/`):
   - D-pad / left thumbstick drives GTK4 focus traversal
   - Guide/Home long-press opens in-game system menu overlay
   - All UI reachable without keyboard or mouse

5. **Recovery handling** (`shell/`):
   - Shell handles `GameCrashed` signal → `RECOVERING` state
   - Recovery UI: "Restart" / "Return to Menu"
   - Shell must not crash when ModpackManager crashes (handle bus name disappearance)

**What is explicitly out of scope for Phase 1:**
- Modpack installation UI (stub `InstallModpack` as unimplemented)
- Update Orchestration service (stub or skip)
- Overlay service (Phase 1 can use shell-native notifications)
- Java Edition support
- Production-quality error handling
- ISO installer (live ISO only — no installation wizard)
- btrfs subvolume layout (Phase 3)

---

### Phase 2 — Runtime Services

Once Phase 1 is complete and validated by the user:

1. **Modpack Manager** (full implementation):
   - Prism Launcher CLI integration for instance management
   - Modrinth `.mrpack` and CurseForge pack format support
   - Download management with checksum verification
   - `InstallProgress` signals (rate-limited to 2/s)

2. **Overlay System** service:
   - `wlr-layer-shell` surface for HUD overlays
   - `ShowNotification`, `ShowCrashOverlay`, `ShowSystemMenu` methods
   - `SystemMenuAction` signal for user choices in the overlay

3. **Logging Infrastructure** service:
   - Structured journald logging from all components
   - `SetLogLevel` / `GetLogLevel` D-Bus methods
   - `GetLastCrashCursor` for crash log retrieval in recovery UI

4. **Update Orchestration** service:
   - pacman update integration
   - btrfs snapshot before every update
   - `ApplyUpdate`, `Rollback`, `ListSnapshots`
   - `UpdateApplied` / `UpdateFailed` signals

---

### Phase 3 — Distribution Layer

- Installable ISO (archiso + custom installer script)
- btrfs subvolume layout: `@`, `@home`, `@snapshots`
- systemd-boot rollback entries (pair each update with a boot entry pointing to the pre-update snapshot)
- Custom pacman repository infrastructure
- Recovery boot environment (separate boot entry, minimal shell)
- Atomic update model: snapshot → install → verify → commit or reboot-to-rollback

---

### Phase 4 — Public Platform

- Stable release process and versioning
- Plugin/extension architecture (define as ADR when the time comes)
- Java Edition support (new ADRs required: JVM selection, modloader strategy)
- Community contribution infrastructure (docs, CI, release notes)

---

## Behavioral Rules

### When writing code

- **Rust in `shell/`**: use `tokio` as the async runtime, `gtk4-rs` + `libadwaita` for UI, `zbus` for D-Bus. Do not introduce other async runtimes.
- **Service language**: not yet decided per-service. Go is recommended for `services/` given its concurrency model. If you start a service in a language, document it as a follow-up decision in a new `Proposed` ADR.
- **No comments explaining what code does** — only comments that explain non-obvious *why* (hidden constraint, workaround, subtle invariant).
- **No TODOs in code** — open an issue or write a `Proposed` ADR instead.
- **Follow the cgroup topology** in `docs/runtime.md` exactly. Do not invent new slice names.
- **Follow the D-Bus interface contracts** in `docs/ipc.md` exactly for method names, signal names, and argument types. Deviations require updating `docs/ipc.md` first.

### When writing documentation

- New architectural decisions → new ADR (`docs/adr/NNNN-title.md` with status `Proposed`, added to the index).
- Implementation discoveries that change the understanding of a component → update `docs/skills.md` in the relevant component section.
- Session model changes → update `docs/session-model.md` state machine and transition table.
- New D-Bus methods or signals → update `docs/ipc.md` interface definition first, then implement.
- Architecture diagram changes → update `docs/architecture/README.md`.

### When encountering ambiguity

1. Check `docs/skills.md` (settled decisions table at the top).
2. Check the relevant ADR.
3. Check the relevant phase doc in this file.
4. If still ambiguous: write a `Proposed` ADR stating the options, make a clearly-stated assumption, proceed, and surface it to the user at the end of the session.

Do not block on ambiguity. Document assumptions and move forward.

### When a task would violate an invariant

Stop. Do not find a workaround that technically avoids the violation but achieves the same effect. Surface the conflict to the user explicitly:

> "Implementing X as described would require [invariant violation]. The options are: [A], [B]. I recommend [A] because [reason]. Waiting for your decision before proceeding."

---

## ADR Protocol

When you need to propose a new ADR:

1. Create `docs/adr/NNNN-short-title.md` with status `Proposed`. Number from the next available after 0012.
2. Fill in Context (the actual problem being faced), options (at least two), and your recommendation.
3. Add a row to `docs/adr/README.md` with status `Proposed`.
4. Note in your session summary that a new `Proposed` ADR was created and requires user decision before implementation proceeds.

When an ADR moves from `Proposed` to `Accepted` (only after user confirmation):

1. Update the ADR file: change `Status: Proposed` to `Status: Accepted`.
2. Update the index table in `docs/adr/README.md`.
3. Update `docs/skills.md` settled decisions table if the decision affects a tech choice.
4. Update any "TBD" references in other docs.

---

## Git Workflow

Full reference: `docs/git-workflow.md`. Key rules for agents:

**Branches:**
- Never commit directly to `main` for implementation work. Use a branch: `feature/`, `fix/`, `refactor/`, `ci/`, `docs/`, `chore/`.
- Branches are short-lived. Open a PR and merge within days.
- Branch names: lowercase, hyphens only. Example: `feature/modpack-manager-dbus-stub`.

**Commits:**
- Mandatory format: `type(scope): description` (Conventional Commits).
- Valid types: `feat`, `fix`, `refactor`, `perf`, `ci`, `docs`, `test`, `build`, `chore`.
- Valid scopes: `shell`, `services`, `runtime`, `session`, `iso`, `packaging`, `infra`, `docs`, `ci`, `adr`.
- Description: lowercase, no period, present tense, under 72 chars.
- Breaking D-Bus interface changes: include `BREAKING CHANGE:` in the commit footer.
- Never: `fix stuff`, `WIP`, `update`, `changes`, or any non-descriptive message.
- **Never include `Co-Authored-By:` lines** in commit messages.

**PRs and merging:**
- PR title = the squash commit message. It must follow Conventional Commits format.
- CI must pass before merge. Required checks: `validate-pr-title`, `lint-docs`.
- Squash merge only. Delete branch after merge.

**Releases:**
- Tags trigger the release workflow automatically.
- Format: `v0.1-alpha`, `v0.2-alpha`, `v1.0-beta`, `v1.0`, then `2026.08`, `2027.01`.
- Never create `release/*` branches.

## Repository Conventions

- **File naming**: kebab-case for all files. Directories match the planned structure in `CLAUDE.md`.
- **PKGBUILD**: all installable MinecrarchOS components must have a PKGBUILD in `packaging/`. Do not skip packaging for a component because "it can be installed manually."
- **Tests**: integration tests in `tests/` using QEMU/KVM for full-stack tests, `systemd-nspawn` for service-level tests. Unit tests alongside source code using the native test framework of the component's language.

---

## Current State Summary (as of Phase 0 completion)

- All 12 ADRs are `Accepted`. No open architectural decisions remain.
- Stack is fully defined: Arch + linux-zen + systemd-boot + btrfs + Gamescope + Rust shell (GTK4) + D-Bus IPC + Prism Launcher (initial).
- Full architecture documented in `docs/architecture/README.md`.
- Session state machine documented in `docs/session-model.md` (7 states, 13 transitions).
- Runtime process model documented in `docs/runtime.md` (cgroup topology, supervision, crash taxonomy).
- D-Bus interface contracts documented in `docs/ipc.md` (4 services, full method/signal/property specs).
- No source code exists yet. Phase 1 is the first implementation phase.
- One follow-up needed before shell UI work begins: GTK4/libadwaita vs. Iced evaluation (noted in ADR-0011). Create a `Proposed` ADR-0013 for this if you start `shell/` work.
