# MinecrarchOS — Test Suite

Full strategy: [`docs/testing-strategy.md`](../docs/testing-strategy.md)

---

## Directory Structure

```
tests/
├── ipc/                    IPC contract conformance tests (Rust, cargo test)
├── services/               Service isolation tests (systemd-nspawn)
│   └── rootfs/             Minimal Arch rootfs for nspawn (built by tools/build-test-rootfs.sh)
├── full-stack/             Full boot-to-game tests (QEMU/KVM)
└── tools/                  Test helpers
    ├── gamepad-emulator.py Synthesizes gamepad events via uinput
    └── fake-game/          Minimal test process that simulates game launch and crash
```

---

## Running Tests

### Unit tests (per component)
```bash
cargo test --workspace
```

### IPC contract tests
```bash
# Requires the service binary to be running on the session bus
cargo test --test ipc -- --nocapture
```

### Service isolation tests (requires systemd-nspawn + root)
```bash
sudo bash tests/services/test-modpack-manager.sh
```

### Full-stack tests (requires QEMU/KVM)
```bash
bash tests/full-stack/mvp-validation.sh
```

### Dependency boundary check
```bash
bash tools/check-deps.sh
```

### MVP validation (manual procedure)
See [`docs/mvp.md`](../docs/mvp.md) for the step-by-step validation procedure.

---

## CI Test Matrix

| Test suite | Runs on | Trigger |
|---|---|---|
| `cargo test` | GitHub Actions (ubuntu-latest) | Every PR and push to `main` |
| `check-deps.sh` | GitHub Actions (ubuntu-latest) | Every PR touching a `Cargo.toml` |
| `cargo deny check` | GitHub Actions (ubuntu-latest) | Every PR |
| IPC contract tests | GitHub Actions (ubuntu-latest, systemd) | Every PR touching `services/` or `docs/ipc.md` |
| Service isolation tests | Self-hosted runner (Arch Linux) | Phase 1+ |
| Full-stack QEMU tests | Self-hosted runner | Nightly + releases only |
