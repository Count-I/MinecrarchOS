# ADR-0007: Initial Game Target — Minecraft Bedrock Edition

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

MinecrarchOS must target a specific Minecraft edition first to build and validate its platform foundations. The platform-level architectural problems — session ownership, compositor lifecycle, fullscreen orchestration, input routing, suspend/resume behavior, game crash recovery, overlay rendering, controller navigation — exist independently of which edition is chosen. The initial edition choice determines: how much complexity the platform team carries in Phase 1, how well the edition aligns with the controller-first appliance UX, and how long it takes to validate the session model before adding Java Edition complexity.

Java Edition introduces additional layers that are significant engineering problems in their own right: JVM runtime management, modloader orchestration (Forge, Fabric, NeoForge), modpack dependency resolution, JVM tuning, asset synchronization, and compatibility management. Carrying this complexity in Phase 1, while also building the session model, compositor integration, and overlay system, would obscure platform-level problems under Java-specific problems.

This decision is **not** a rejection of Java Edition. Java Edition is the long-term primary pillar of the platform. The deep modded Minecraft ecosystem — modpacks, custom runtimes, CurseForge/Modrinth integration, instance isolation — is central to MinecrarchOS's long-term vision. Bedrock is chosen as the starting point because it allows the platform architecture to be validated first, on its own terms.

## Decision

We will use Minecraft Bedrock Edition as the initial target workload for Phase 1. Java Edition support is the long-term goal and will be added once platform foundations are stable and validated. The shell's game-launching interface must be designed with abstraction in mind from Phase 1, so that Java Edition can be added without architectural rework.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Java Edition first | Correct long-term, but adds JVM management, modloader orchestration, and modpack resolution to Phase 1; these Java-specific problems would obscure platform-level problems and slow down session model validation |
| Both editions simultaneously | Phase 1 scope too large; platform foundations would be harder to validate with two different process models to support |
| Neither (abstract game slot with no real game) | Too abstract; real platform decisions require a real game's process model, input requirements, and crash behavior to drive them |

## Consequences

### Positive

- Bedrock natively supports gamepads, which aligns directly with the controller-first UX goal.
- No JVM management required in Phase 1; simpler process lifecycle for session validation.
- Allows full Phase 1 focus on platform-level problems without carrying Java ecosystem complexity.
- Bedrock's deterministic process model (no modloader variability) is easier to build reliable crash recovery around.

### Negative

- Bedrock Edition for Linux is available only through community packaging (mcpelauncher or similar); this packaging dependency must be tracked and maintained.
- Some Bedrock platform behaviors may differ from Java Edition, requiring re-validation of the session model when Java support is added.
- The Bedrock-first decision may be perceived as deprioritizing the modded Java Edition community, which is the platform's long-term primary audience. Communication of this strategy must be explicit.

### Neutral

- The Minecrarch Shell's game-launching interface must treat the launcher backend as an abstraction from day one — so Java Edition can be added without reworking the shell.
- Components in `services/` and `runtime/` must be edition-agnostic in their contracts wherever possible.
- Java Edition support will require its own ADRs when the time comes: JVM selection, modloader integration strategy, instance isolation model, and modpack format support.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
