# ADR-0008: Launcher Backend — Prism Launcher (Initial)

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

The Minecrarch Shell is the orchestration layer, not the launcher itself (see ADR-0009). It delegates instance management, modpack installation, Java runtime management, and Minecraft process lifecycle to a launcher backend. For Phase 1 and early Phase 2, the platform needs a launcher backend that is: open source, actively maintained, supports Java Edition instance management, has a usable CLI interface, and does not require a running desktop environment to function.

This decision is explicitly an "initial" choice. The platform architecture must not couple itself irrevocably to any specific launcher backend. All launcher integration must live behind a service interface in `services/`, not as direct calls in the shell.

## Decision

We will use Prism Launcher as the initial launcher backend. All Prism integration must be bounded in `services/` behind an interface contract so it can be replaced when custom runtime services mature.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| MultiMC | Prism Launcher is the actively maintained community fork; MultiMC's development has slowed significantly |
| PolyMC | Development stalled following a community split; Prism Launcher is its active successor |
| ATLauncher | Less suitable for headless/scripted operation; tighter coupling to its own GUI |
| Modrinth App | Good UI but less established CLI/API surface for integration; younger project with less track record on Linux headless operation |
| Custom launcher from scratch | Phase 1 scope does not warrant building a full launcher; Prism provides everything needed for platform validation |

## Consequences

### Positive

- Prism Launcher is open source (GPL-3.0), actively maintained, and widely used in the Arch/Linux Minecraft community.
- Its instance model maps well to the MinecrarchOS concept of isolated, manageable game instances.
- Available in the AUR and official Arch community repository.
- CLI mode is suitable for scripted and headless operation — the integration model MinecrarchOS requires.

### Negative

- MinecrarchOS becomes dependent on Prism's CLI interface and instance directory format. If Prism changes either, integration code in `services/` must be updated.
- Using Prism headlessly requires explicit integration work; it is primarily a GUI application.

### Neutral

- The shell must treat the launcher backend as a replaceable component defined by a service interface, not by direct Prism API calls. When custom runtime services mature in Phase 2+, Prism may be partially or fully replaced.
- All Prism integration code must live in a bounded component within `services/` — never in `shell/`.
- The service interface contract defined in Phase 1 for Prism integration will become the abstraction boundary that Java Edition runtime services must also satisfy.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
