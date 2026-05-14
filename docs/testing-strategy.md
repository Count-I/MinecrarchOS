# MinecrarchOS — Architectural Testing Strategy

## Philosophy

Testing in MinecrarchOS is not about code coverage metrics. It is about validating that the architecture works as specified. The state machines in `docs/state-machines.md` are the specification. Tests prove that implementations conform to them.

**Priority order for testing effort:**
1. IPC contract conformance (services behave as `docs/ipc.md` specifies)
2. Session state machine conformance (shell transitions correctly)
3. Process supervision correctness (runtime layer handles crashes, hangs, OOM)
4. Unit behavior (individual functions work correctly)

A unit test suite with 100% coverage that never tests the D-Bus interface is less valuable than an IPC conformance test that catches a broken signal.

---

## Test Pyramid

```text
              ┌──────────────────┐
              │   QEMU Full Stack │  ← Slowest, most valuable for integration
              │   (boot to game)  │
              └────────┬─────────┘
           ┌───────────▼──────────────┐
           │   systemd-nspawn tests   │  ← Service isolation, D-Bus IPC
           │   (per-service, fast)    │
           └───────────┬──────────────┘
        ┌──────────────▼─────────────────┐
        │   Unit tests (cargo test)      │  ← Per-crate, fast, no I/O
        │   IPC mock tests (zbus mock)   │
        └────────────────────────────────┘
```

---

## Unit Tests

**Location:** Alongside source code in each crate (`src/` directory, `#[cfg(test)]` modules or `tests/` subdirectory).

**Rules:**
- Unit tests must not touch the filesystem, network, D-Bus, or processes.
- Test pure logic: state machine transition functions, data parsing, type conversions.
- Use `zbus`'s built-in test utilities for D-Bus client mock testing — do not spin up a real D-Bus connection.

**Per-component focus:**

| Component | What to unit test |
|---|---|
| `shell` | State machine transition logic, D-Bus proxy mock behavior, focus traversal logic |
| `services/modpack-manager` | Instance config parsing, launch argument construction, exit code classification |
| `services/updater` | Snapshot naming logic, changelog parsing, update state transitions |
| `runtime` | Process exit code → signal name mapping, cgroup path construction |
| `shared` | Type serialization/deserialization, error type conversions |

---

## IPC Contract Tests

These tests verify that service implementations conform to the D-Bus interface contracts in `docs/ipc.md`.

**Location:** `tests/ipc/`

**Approach:**
- Spin up the real service binary (via `systemd-nspawn` or directly in CI).
- Connect to its D-Bus interface using a test client (Rust binary using `zbus`).
- Call each method and assert on the response type and content.
- Subscribe to signals and assert they are emitted with correct argument types.
- Trigger error conditions and assert correct D-Bus error names are returned.

**Example IPC test structure:**

```rust
// tests/ipc/test_modpack_manager.rs
#[tokio::test]
async fn list_instances_returns_empty_on_fresh_state() {
    let conn = Connection::session().await.unwrap();
    let proxy = ModpackManagerProxy::new(&conn).await.unwrap();
    let instances = proxy.list_instances().await.unwrap();
    assert!(instances.is_empty());
}

#[tokio::test]
async fn launch_nonexistent_instance_returns_error() {
    let conn = Connection::session().await.unwrap();
    let proxy = ModpackManagerProxy::new(&conn).await.unwrap();
    let result = proxy.launch_instance("does-not-exist").await;
    assert!(result.is_err());
    // Assert error is org.minecrarch.Error.InstanceNotFound
}
```

**IPC conformance CI job:** Runs on every PR that touches `services/` or `docs/ipc.md`.

---

## Service Isolation Tests (systemd-nspawn)

These tests run a single service in an isolated container and verify its behavior under operational conditions.

**Location:** `tests/services/`

**What to test:**
- Service starts and registers on D-Bus within 5 seconds.
- Service responds to all defined D-Bus methods without crashing.
- Service emits expected signals when triggered.
- Service recovers after receiving SIGTERM (systemd restart).
- Service does not crash when called with invalid arguments.

**Runner:** `systemd-nspawn` with a minimal Arch rootfs containing only the service binary, systemd, and D-Bus. No Gamescope, no shell.

```bash
# Example nspawn test invocation
systemd-nspawn \
  --directory=/tmp/test-rootfs \
  --bind-ro=/path/to/service-binary:/usr/bin/minecrarch-modpack-manager \
  -- \
  /usr/bin/minecrarch-modpack-manager &

# Wait for service to register, run IPC test client
sleep 2
cargo test --test test_modpack_manager -- --nocapture
```

---

## Full-Stack Tests (QEMU/KVM)

These tests boot the full MinecrarchOS stack and validate end-to-end flows.

**Location:** `tests/full-stack/`

**What to test:**
- Complete boot sequence: linux-zen → systemd → autologin → Gamescope → shell.
- MVP validation (see `docs/mvp.md` for the complete procedure).
- Session state machine transitions (boot → menu → launch → in-game → crash → recovery → menu).
- Suspend/resume cycle with a running fake game.
- Service restart while in-game (shell degrades gracefully).

**Runner:** QEMU/KVM with the MinecrarchOS live ISO. Display via VNC or Wayland passthrough.

**Gamepad emulation:** `uinput` kernel module + `python-evdev` to synthesize gamepad events programmatically during tests.

```bash
# Example: synthesize d-pad down press to navigate recovery UI
python3 tools/gamepad-emulator.py --event dpad_down
python3 tools/gamepad-emulator.py --event button_a
```

**CI:** Full-stack QEMU tests run on release builds and nightly, not on every PR (too slow for PR CI). Estimated runtime: 5–15 minutes per test suite.

---

## Gamescope Session Testing

Testing that the shell integrates correctly with Gamescope is non-trivial. Approach:

**Phase 1 (headless Gamescope):**
- Run Gamescope with a virtual display (`--backend headless` if available, otherwise `Xvfb` + XWayland).
- Verify the shell starts and registers its Wayland surface.
- Verify that when a fake game process starts, Gamescope routes input to the game surface.

**Phase 2+ (full QEMU test):**
- Boot full stack in QEMU with GPU passthrough or software rendering.
- Capture VNC output to verify visual state.

---

## Architectural Boundary Tests

CI job that validates no forbidden dependencies exist.

**Location:** `tools/check-deps.sh`

**What it checks:**
- `shell/Cargo.toml` does not list any `minecrarch-*` service crate as a dependency.
- `shared/Cargo.toml` does not list `shell`, `services/*`, or `runtime` crates.
- No service `Cargo.toml` lists another service crate as a dependency.

This test runs on every PR that touches any `Cargo.toml`.

---

## IPC Mock Strategy for Shell Tests

The shell communicates with services exclusively via D-Bus. Unit testing the shell's response to service events requires a D-Bus mock.

**Approach:** Use `zbus` mock server in test binaries to simulate service behavior.

```rust
// In shell unit tests: mock ModpackManager that emits GameCrashed
struct MockModpackManager;

#[dbus_interface(name = "org.minecrarch.ModpackManager")]
impl MockModpackManager {
    async fn launch_instance(&self, id: &str) {
        // Immediately emit crash signal for testing recovery flow
        self.game_crashed(id, -1, "SIGSEGV").await;
    }
}
```

This allows testing the shell's LAUNCHING → RECOVERING transition without running a real game or real service.

---

## Test Infrastructure Requirements

| Tool | Purpose | When needed |
|---|---|---|
| `cargo test` | Unit + IPC mock tests | Phase 1 |
| `systemd-nspawn` | Service isolation tests | Phase 1 |
| `QEMU/KVM` | Full-stack tests, MVP validation | Phase 1 |
| `python-evdev` + `uinput` | Gamepad emulation | Phase 1 |
| `busctl` | Manual D-Bus verification | Phase 1 |
| `Xvfb` or Gamescope headless | Headless compositor testing | Phase 2 |
| VNC capture | Visual test verification in CI | Phase 2 |

---

## tests/ Directory Structure

```text
tests/
├── README.md               (this strategy in brief + how to run each suite)
├── ipc/                    (IPC contract conformance tests — Rust)
│   ├── modpack_manager.rs
│   ├── overlay.rs
│   ├── updater.rs
│   └── logging.rs
├── services/               (systemd-nspawn service isolation tests)
│   ├── test-modpack-manager.sh
│   ├── test-overlay.sh
│   └── rootfs/             (minimal Arch rootfs for nspawn)
├── full-stack/             (QEMU full-stack test scripts)
│   ├── mvp-validation.sh
│   ├── session-lifecycle.sh
│   └── suspend-resume.sh
└── tools/                  (test helpers)
    ├── gamepad-emulator.py
    └── fake-game/          (test game binary — see docs/mvp.md)
```
