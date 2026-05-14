# MinecrarchOS — Git Workflow

## Strategy

Trunk-Based Development. One permanent branch (`main`), short-lived feature branches, squash merges, releases via tags. No Git Flow, no permanent `develop` branch, no `release/*` branches.

`main` must always be bootable, deployable, and functional. Nothing merges that breaks boot, shell, compositor, session, or the core UX path.

---

## Branch Naming

```text
feature/<what>      New capability
fix/<what>          Bug fix
refactor/<area>     Structural change without behavior change
ci/<what>           Pipeline or workflow change
docs/<topic>        Documentation only
chore/<what>        Maintenance (deps, tooling, cleanup)
perf/<area>         Performance improvement
```

**Examples:**

```text
feature/gamepad-navigation
feature/bedrock-launcher
fix/session-startup-race
refactor/modpack-manager-dbus
ci/iso-build-pipeline
docs/ipc-interface-contracts
chore/update-rust-deps
```

Rules:
- Lowercase, hyphens only.
- Branches are short-lived. Merge within days, not weeks.
- No `wip/`, no `temp/`, no personal branches on the main repo.
- One concern per branch. A branch that touches shell + services + iso is doing too much.

---

## Commit Format

[Conventional Commits](https://www.conventionalcommits.org/) — mandatory.

```text
type(scope): description

[optional body]

[optional footer: BREAKING CHANGE: ..., Fixes #NNN]
```

**Types:**

| Type | When |
|---|---|
| `feat` | New user-facing capability |
| `fix` | Bug fix |
| `refactor` | Code restructuring without behavior change |
| `perf` | Performance improvement |
| `ci` | Pipeline, workflow, or automation change |
| `docs` | Documentation only |
| `test` | Test additions or changes |
| `build` | Build system, packaging, tooling |
| `chore` | Maintenance (deps, cleanup, tooling) |

**Scopes:** `shell`, `services`, `runtime`, `session`, `iso`, `packaging`, `infra`, `docs`, `ci`, `adr`

**Examples:**

```text
feat(shell): add fullscreen radial game menu
fix(session): resolve gamescope startup race on autologin
refactor(modpack-manager): split launch and install concerns
ci(build): add ISO validation to PR pipeline
docs(ipc): add ModpackManager interface contract
perf(shell): reduce D-Bus signal subscription overhead
feat(iso): add systemd-boot entry for recovery mode
BREAKING CHANGE: rename D-Bus method LaunchGame to LaunchInstance
```

**Rules:**
- Description: lowercase, no period, present tense, under 72 chars.
- Body: wrap at 100 chars. Explain *why*, not *what*.
- `BREAKING CHANGE:` in footer when a D-Bus interface, file format, or CLI API changes incompatibly.
- With squash merge: the **PR title** becomes the commit on `main`. PR title must follow this format. Individual commits on the branch are squashed away.

**Never acceptable:**
- `fix stuff`
- `WIP`
- `asdf`
- `update`
- `changes`
- Any message that does not state what changed

---

## Pull Request Process

1. Branch from `main`. Keep it short-lived and focused.
2. PR title = the squash commit message. It must follow Conventional Commits format. CI validates this automatically.
3. PR description: use the template. Reference relevant ADRs if the PR involves architecture.
4. CI must pass before merge is allowed. No exceptions.
5. Squash merge only. The branch is deleted after merge.
6. One PR per concern. If a PR needs to touch shell + services + iso for one feature, that's acceptable. If it touches two unrelated features, split it.

---

## Release Process

Releases are tags on `main`. No permanent release branches.

### Version format

**Phase 0–1 (current):** pre-release tags

```text
v0.1-alpha
v0.2-alpha
v0.5-preview
v1.0-beta
v1.0
```

**Phase 3+ (stable releases):** calendar versioning

```text
2026.08
2026.10
2027.01
```

### Creating a release

```bash
# Tag main at the release point
git tag -a v0.1-alpha -m "Phase 1 prototype: Gamescope session + Rust shell scaffold"
git push origin v0.1-alpha
```

The `release.yml` workflow triggers automatically on tag push:
1. Generates changelog from conventional commits since the previous tag.
2. Creates a GitHub Release with the changelog.
3. (Phase 1+) Builds and attaches the signed ISO.

### Changelog

Auto-generated from conventional commits. `feat` entries become "Features", `fix` entries become "Bug Fixes", `BREAKING CHANGE` entries are highlighted. Maintained in `CHANGELOG.md` via the release workflow.

---

## Distribution Channels

| Channel | Source | Audience | Stability |
|---|---|---|---|
| **nightly** | Latest `main` build | Testers, early feedback | Unstable — may not boot |
| **testing** | Tagged snapshots | QA, semi-stable validation | Semi-stable |
| **stable** | Verified, signed releases | End users | Stable |

Nightly builds are triggered automatically by `nightly.yml` at UTC midnight. They publish as a rolling GitHub pre-release named `nightly`. The previous nightly is replaced, not accumulated.

---

## Multi-Repo Evolution

The project starts as a monorepo. Components will split into separate repos as they mature. This is a planned evolution, not a current requirement.

**Current (Phases 1–2): monorepo**

```text
MinecrarchOS/   ← everything lives here
├── shell/
├── services/
├── runtime/
├── packaging/
├── iso/
├── tools/
└── tests/
```

**Future (Phase 3+): multi-repo**

```text
MinecrarchOS/         ← meta-repo, ISO builder, release manager, CI root
minecrarch-shell/     ← shell package (Rust, GTK4)
minecrarch-session/   ← session management, Gamescope integration
minecrarch-installer/ ← TUI/graphical installer
minecrarch-bedrock/   ← Bedrock Edition runtime packaging
minecrarch-config/    ← default configuration and branding
```

The meta-repo pins component versions and orchestrates the ISO build. Component repos release independently; the meta-repo integrates them.

**Split criteria for a component:**
- It has a stable, versioned D-Bus interface.
- External contributors regularly work on it independently.
- Its release cadence differs from the platform.

Never split prematurely. Splitting adds integration overhead.

---

## Branch Protection (GitHub Settings)

Configure on `main`:

```text
✅ Require a pull request before merging
   ✅ Require approvals: 1 (relaxed to 0 during early solo development)
   ✅ Dismiss stale pull request approvals when new commits are pushed
✅ Require status checks to pass before merging
   ✅ Require branches to be up to date before merging
   Required checks:
     - lint-docs
     - validate-pr-title
✅ Require linear history (enforces squash merge)
✅ Do not allow force pushes
✅ Do not allow deletions
```

Apply via GitHub CLI:

```bash
gh api repos/Count-I/MinecrarchOS/branches/main/protection \
  --method PUT \
  --input - <<'EOF'
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["lint-docs", "validate-pr-title"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": true,
    "required_approving_review_count": 0
  },
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_linear_history": true
}
EOF
```

---

## CI/CD Pipeline Overview

See `.github/workflows/` for full definitions.

| Workflow | Trigger | Jobs |
|---|---|---|
| `ci.yml` | PR to `main`, push to `main` | validate-pr-title, lint-docs, (Phase 1+: build-shell, validate-pkgbuild, build-iso) |
| `nightly.yml` | Daily UTC midnight + manual | nightly-build (Phase 1+: build ISO, publish artifact) |
| `release.yml` | Tag push `v*` or `20[0-9][0-9].*` | generate-changelog, create-release, (Phase 1+: build-iso, sign, attach) |

---

## Invariants for Agents and Contributors

- Never merge directly to `main`. Always via PR.
- Never force-push any branch after it has an open PR.
- Never create `develop`, `staging`, `release/*` branches.
- A PR that breaks the boot sequence, session start, or shell launch will be reverted immediately.
- Breaking changes to D-Bus interfaces require a `BREAKING CHANGE:` footer and an updated `docs/ipc.md` in the same PR.
- New ADRs required for any architectural decision. A PR without an ADR for a new architectural decision will not be merged.
