# ADR-0009: Shell as Orchestration Layer (Not Monolithic Backend)

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

The Minecrarch Shell is the user-visible layer of the platform. The critical architectural question is: what is the shell responsible for, and where is the boundary between the shell and dedicated services?

In many gaming platforms, the "launcher" grows to absorb everything: download management, mod installation, Java management, network calls, file I/O, manifest parsing. This produces a monolithic application that is difficult to test, maintain, replace, and reason about. It also creates a single point of failure: if the shell crashes, everything crashes. This pattern is explicitly rejected by MinecrarchOS.

## Decision

We have decided that the Minecrarch Shell is an orchestration and UX layer only. It coordinates services but does not implement platform logic itself.

**The shell IS responsible for:**
- User experience and navigation
- Session lifecycle management (start, suspend, resume, shutdown)
- Game launch orchestration (signaling services to launch, not launching directly)
- Overlay management and rendering
- Recovery flow coordination
- Controller UX and focus management
- State display (showing status from services)

**The shell is NOT responsible for:**
- Java runtime management
- Modloader installation
- Mojang manifest parsing or asset fetching
- Driver management
- Modpack dependency resolution
- Network download management
- File integrity verification

Those responsibilities belong to dedicated services in `services/`. The shell communicates with services over IPC (see ADR-0012).

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Monolithic shell (handles everything) | Violates the modularity principle; creates a single point of failure; prevents component replacement; makes testing very difficult |
| Thin shell with no formal service layer | Services are still needed; without a formal boundary they become informal, poorly-defined, and impossible to replace cleanly |
| Shell plus one omnibus service | The same monolith problem, moved one layer down |

## Consequences

### Positive

- The shell can be replaced or rewritten without affecting service logic.
- Services can be developed, tested, and deployed independently of the shell.
- A service crash does not crash the shell; the shell can display recovery UI while the service restarts.
- Clear contributor ownership: `shell/` contributors do not need to understand Java runtime internals; `services/` contributors do not need to understand Wayland compositing.

### Negative

- Requires a well-defined IPC mechanism between shell and services (see ADR-0012). The quality of this interface is critical — a poorly designed IPC contract will create tight coupling through the interface boundary.
- Adds latency compared to in-process calls, though for UI-level interactions this is not perceptible.

### Neutral

- `shell/` and `services/` are separate contributor domains with separate skill requirements (see `docs/skills.md`).
- The IPC mechanism (ADR-0012) becomes a critical architectural interface. Its design must be completed before Phase 1 implementation begins.
- Integration testing must explicitly cover the shell-service boundary — neither unit tests in the shell nor unit tests in services are sufficient alone.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
