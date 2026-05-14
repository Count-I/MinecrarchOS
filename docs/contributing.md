# Contributing to MinecrarchOS

Phase 0 (architecture) is complete. Contributions are now open for Phase 1 work.

Before contributing, read:
- [System Architecture](./architecture/README.md) — how the system works
- [docs/skills.md](./skills.md) — what to know for each component area
- [AGENTS.md](../AGENTS.md) — phase objectives and behavioral rules (also useful for human contributors)
- The relevant [ADR](./adr/README.md) for the component you're touching

---

## What to Work On

Phase 1 priorities, in order:

1. Gamescope session (`iso/`, `packaging/`)
2. Minecrarch Shell scaffold (`shell/`)
3. ModpackManager D-Bus service stub (`services/modpack-manager/`)
4. Controller navigation (`shell/`)
5. Recovery handling (`shell/`)

See [docs/roadmap.md](./roadmap.md) for full Phase 1 scope.

---

## Ground Rules

- **Do not contradict Accepted ADRs.** If you believe a decision should be revisited, open a new `Proposed` ADR — don't implement around it.
- **Shell is Rust, IPC is D-Bus.** These are settled (ADR-0011, ADR-0012). Do not propose alternative languages for `shell/` or alternative IPC mechanisms.
- **All UI must be gamepad-navigable** (ADR-0010). Every interactive element you add to the shell must be reachable without a keyboard or mouse.
- **No platform logic in the shell** (ADR-0009). Game launching, downloads, mod management — in `services/` only.
- **Follow the D-Bus interface contracts in `docs/ipc.md`** exactly. Propose changes to the doc before implementing deviations.

---

## Pull Request Process

1. Fork and create a branch from `main`.
2. Implement your change. Include tests where applicable (`tests/`).
3. Ensure your PKGBUILD is updated if you've added a new installable component (`packaging/`).
4. Open a PR with a description of what changed and why. Reference relevant ADRs.
5. PRs that contradict an Accepted ADR will be closed. Open an ADR first.

---

## Proposing Architecture Changes

New architectural decisions → new ADR.

1. Copy the template from [docs/adr/README.md](./adr/README.md).
2. Create `docs/adr/NNNN-short-title.md` with status `Proposed`.
3. Add it to the index table in `docs/adr/README.md`.
4. Open a PR. The ADR is discussed in review. It is merged as `Proposed`.
5. Implementation PRs follow only after the ADR is moved to `Accepted` by the maintainer.

---

## Code Style

- **Rust** (`shell/`, any Rust services): `cargo fmt`, `cargo clippy --deny warnings`. Follow Rust API guidelines for public APIs.
- **Go** (services, if used): `gofmt`, `go vet`.
- **Bash** (`scripts/`, `tools/`): `set -euo pipefail` at the top of every script. Shellcheck-clean.
- **PKGBUILD**: follow Arch Linux packaging guidelines. Run `namcap` before submitting.
- **No comments explaining what code does** — only why, when non-obvious.

---

## Code of Conduct

See [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md).
