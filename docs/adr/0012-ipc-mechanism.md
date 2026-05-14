# ADR-0012: IPC Mechanism — D-Bus (User Session Bus)

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

ADR-0009 establishes that the shell and services are separate processes. They must communicate via IPC. The mechanism determines: latency, discoverability, schema enforcement, language binding availability, async notification support, and operational debugging ease.

Required communication patterns:
- Shell → service: commands ("launch game", "install modpack", "pause download")
- Service → shell: async notifications ("game crashed", "update ready", "modpack installed")
- Shell → service: state queries ("what instances are available?", "what is download progress?")

The shell is implemented in Rust (ADR-0011). The IPC mechanism must have a mature Rust client library. Services may be implemented in different languages — the mechanism must also have good bindings for Go and C, the other likely service languages.

## Decision

We will use **D-Bus on the systemd user session bus** as the IPC mechanism between the shell and all `services/`.

- **Bus**: user session bus (`DBUS_SESSION_BUS_ADDRESS`), managed by systemd's `dbus.service` user unit.
- **Shell client**: `zbus` (Rust) — idiomatic async D-Bus library.
- **Service interfaces**: defined as D-Bus XML introspection documents, one per service.
- **Async notifications**: D-Bus signals — services emit signals; the shell subscribes.
- **State queries**: D-Bus method calls, returning typed values.
- **Debugging**: `busctl` (CLI), `d-spy` (GUI), both available on Arch Linux.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Unix sockets + JSON-RPC | Simpler initially but requires defining and enforcing the full protocol manually; no introspection; debugging requires socat + jq rather than purpose-built Linux IPC tools |
| gRPC | Strong schema via protobuf but HTTP/2 transport overhead is unnecessary for local IPC; adds protobuf to the build toolchain of every service |
| Unix sockets + custom binary protocol | Maximum performance; not justified at UI-level message frequency; significant additional engineering with no benefit |

## Consequences

### Positive

- D-Bus is the standard Linux IPC convention — well-understood by Linux systems developers, consistent with platform expectations.
- Introspection: all service interfaces are discoverable and inspectable at runtime via `busctl introspect`.
- `zbus` provides async/await-native D-Bus for Rust, compatible with Tokio — the likely async runtime for the shell.
- D-Bus signals map directly to the async notification pattern required by the shell (service events are pushed, not polled).
- The systemd user session bus is already available in the Gamescope session — no additional daemon setup required.
- Service interfaces defined in XML become the formal API contract between `shell/` and `services/` contributors.

### Negative

- D-Bus XML interface definitions are verbose; tooling (`zbus-xmlgen`) can generate Rust bindings from them but adds a code generation step.
- D-Bus has per-message overhead compared to raw sockets. At UI-level message frequency (commands, notifications) this is imperceptible, but high-frequency telemetry should not use D-Bus.
- Contributors unfamiliar with D-Bus concepts (bus names, object paths, interfaces, methods, signals, properties) need onboarding.

### Neutral

- Every service must register on the session bus with a well-defined bus name (e.g., `org.minecrarch.ModpackManager`, `org.minecrarch.UpdateOrchestrator`).
- `docs/ipc.md` must document all bus names, object paths, interface definitions, and signal schemas once services are implemented.
- High-frequency data (e.g., download progress percentage updates) should be rate-limited before emitting as D-Bus signals to avoid flooding the bus. Polling via a D-Bus property is an alternative for high-frequency state.
- IPC mock/stub tooling for testing can be implemented using `busctl` to simulate service signals, or `zbus` mock objects in unit tests.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
