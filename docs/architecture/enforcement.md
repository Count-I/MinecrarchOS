# MinecrarchOS — Architectural Enforcement Mandate

> The architecture is the product. The code exists to preserve and implement the architecture.

This document is the canonical reference for architectural authority in MinecrarchOS. It governs every implementation decision, PR review, and agent action.

---

## What MinecrarchOS Must Never Become

- A launcher on top of a desktop
- A collection of startup scripts
- An Electron frontend
- A tightly coupled monolith
- A "Minecraft distro"
- A Wayland rice with branding

Any evolution toward these forms is an architectural failure — regardless of how useful the shortcut appears in the moment.

---

## Non-Negotiable Invariants

### 1. Minecrarch-OS is a Platform, Not a Desktop

The system is a dedicated gaming session, an appliance-style runtime, a fullscreen platform. All architecture must reinforce this. When a design decision makes the system feel more like a desktop, it is wrong.

### 2. Gamescope Owns the Session

Gamescope is the session compositor, the fullscreen owner, the runtime display environment. It is not a helper tool, not a wrapper, not optional infrastructure. Never introduce architecture that weakens or bypasses Gamescope's session ownership.

### 3. The Shell is an Orchestrator

The shell orchestrates, navigates, coordinates. It does not implement domain logic. The moment shell code starts parsing manifests, resolving dependencies, managing JVMs, or processing game assets — that code belongs in a service. Extract immediately.

### 4. Services Own Domain Logic

Heavy logic lives in dedicated services: Java runtime, modpack resolution, launcher integration, updates, logging, overlays, asset management. The shell may orchestrate these services. It must not absorb them.

### 5. IPC is a First-Class Architectural Layer

Inter-process communication is not plumbing — it is architecture. All shell↔service communication goes through D-Bus contracts defined in `docs/ipc.md`. Ad-hoc HTTP APIs, hidden shared state, filesystem polling IPC, and direct process access are architectural violations.

### 6. Controller-First UX is Mandatory

Every UX decision assumes gamepad-first navigation, fullscreen permanence, couch-distance readability, deterministic focus behavior. Mouse-first assumptions in platform UX are architectural violations. Keyboard/mouse support for gameplay (especially Java Edition) is not affected by this rule.

### 7. Bedrock is a Strategic Starting Point

Bedrock is the initial target because it simplifies platform validation. The architecture must remain runtime-agnostic. Do not couple the runtime layer, services layer, or IPC contracts to Bedrock-specific behaviors. Java Edition and modded ecosystems are primary long-term goals.

### 8. systemd is Platform Infrastructure

Use systemd for what it was designed for: units, scopes, slices, cgroups, service supervision, lifecycle orchestration. Do not replace systemd responsibilities with shell scripts, background loops, or fragile process babysitting.

### 9. State Machines are Required

Complex runtime flows must be modeled explicitly. Session lifecycle, launcher lifecycle, suspend/resume, crash recovery, overlay ownership — all must have documented state machines. See `docs/state-machines.md`. Implicit state behavior is an architectural violation.

### 10. No Temporary Hacks Without Tracking

Never implement "just for now" logic, hidden shortcuts, or bypass layers without: an explicit TODO with owner, a tracking issue, an expiration condition, and an architectural justification. Undocumented hacks are architectural debt with no repayment plan.

---

## Decision-Making Priorities

When forced to choose, prefer in this order:

1. Architectural integrity
2. Modularity
3. Explicitness
4. Observability
5. Recoverability
6. Linux-native patterns
7. Long-term maintainability

Over:
- Speed
- Convenience
- Shortcuts
- Temporary hacks
- Rapid feature growth

---

## Architectural Failure Conditions

The following are explicit failure states. If any of these occur, work stops and the architecture is repaired before continuing:

| Failure | Signal |
|---|---|
| Shell becoming a monolith | Shell code handles domain logic |
| Direct runtime coupling | Shell or service directly execs game process |
| Hidden state ownership | State lives outside declared service |
| Compositor ownership ambiguity | Multiple components claim display authority |
| Desktop-environment assumptions | Fallback WM or DE introduced |
| Ad-hoc IPC sprawl | HTTP, sockets, or polling introduced alongside D-Bus |
| Controller UX regressions | UI element unreachable without mouse/keyboard |
| Service responsibility overlap | Two services own the same domain |
| Architecture undocumented in code evolution | Code drifts from ADRs without new ADRs |

---

## Before Implementing Any Feature

1. Identify which ADR governs it.
2. Identify which component owns the responsibility.
3. Determine whether the change introduces coupling.
4. Determine whether a new architectural boundary is created.
5. Determine whether a new ADR is required.

If any step is unclear: **stop and request clarification**. Never improvise architecture.

---

## Success Condition

Success is not "Minecraft launches."

Success is:

> The platform remains architecturally coherent while evolving into a long-term Linux gaming ecosystem.

Protect the architecture first. Everything else is replaceable.
