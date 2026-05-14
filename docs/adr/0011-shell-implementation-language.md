# ADR-0011: Shell Implementation Language — Rust

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

The Minecrarch Shell is the core user-facing component of the platform. The language choice determines: performance characteristics, available UI frameworks, Wayland client library support, IPC client library availability, contributor onboarding difficulty, and long-term maintainability.

The shell must:
- Drive a fullscreen Wayland application embedded in Gamescope (ADR-0005)
- Handle gamepad input with low latency (ADR-0010)
- Communicate with `services/` over D-Bus (ADR-0012)
- Manage session lifecycle events from systemd
- Render overlays as separate Wayland surfaces

This is a systems-adjacent UI application running in a constrained, headless-adjacent environment. The language choice for the shell does not mandate the same language for `services/` — each service may choose the language best suited to its workload.

## Decision

We will use **Rust** for the Minecrarch Shell implementation.

- **UI framework**: GTK4 + libadwaita (primary choice). Iced is a viable alternative and should be evaluated before Phase 1 implementation begins — the final framework decision between GTK4/libadwaita and Iced should be made as a follow-up ADR.
- **Input handling**: libinput (via Rust bindings) or SDL2 for gamepad events.
- **IPC client**: `zbus` for D-Bus communication (per ADR-0012).
- **Build system**: Cargo, with a PKGBUILD wrapping the Cargo build for packaging.

Services in `services/` are not required to use Rust. Go is a strong candidate for services given its concurrency model and suitability for long-running background processes — this is a separate decision to be made per service.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Go | Good concurrency model for service orchestration, but limited mature Wayland UI framework options; better suited to the `services/` layer than to the shell UI |
| C with GTK4 | Maximum ecosystem access; native GTK4 and libinput; but memory safety burden is significant for a long-lived project, and Rust provides the same ecosystem access with safety guarantees |
| Python | Rapid prototyping; poor performance for a fullscreen compositor-embedded app; runtime dependency adds overhead and packaging complexity |

## Consequences

### Positive

- Memory safety without garbage collection — important for a long-running session process where a crash returns the user to a bad state.
- Excellent Wayland client support: `wayland-client` crate, `wlr-layer-shell` bindings, `smithay-client-toolkit`.
- `zbus` provides a mature, idiomatic D-Bus client for Rust, consistent with the ADR-0012 decision.
- GTK4/libadwaita is well-supported on Arch Linux and provides native controller/focus navigation primitives (ADR-0010 alignment).
- Cargo + PKGBUILD is a well-understood packaging pattern in the Arch ecosystem.

### Negative

- Rust has a steep learning curve; contributor pool is smaller than Go or Python.
- Compile times are longer than Go — development iteration speed may be slower.
- GTK4 Rust bindings (`gtk4-rs`) are mature but add a layer of abstraction over the C GTK4 API that contributors must understand.

### Neutral

- Services may use different languages. The D-Bus interface (ADR-0012) is the coupling point — Rust in `shell/` and Go in `services/` are fully compatible.
- The follow-up decision between GTK4/libadwaita and Iced should evaluate: controller focus traversal support, Gamescope embedding behavior, and accessibility tree support.
- All build pipelines, PKGBUILD files, and CI configurations for `shell/` assume Cargo as the build system.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
