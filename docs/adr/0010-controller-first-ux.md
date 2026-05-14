# ADR-0010: Controller-First UX as Primary Interaction Model

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

MinecrarchOS targets couch gaming and console-like scenarios. The project vision describes "controller-first interaction", "couch gaming", "fullscreen navigation", and "appliance-style interaction" as core properties of the platform. These are not UI preferences — they are architectural constraints. The choice to prioritize gamepad input determines: which UI frameworks are viable, how focus traversal must be designed, what testing infrastructure is required, and what accessibility assumptions can be made.

This decision also interacts with ADR-0007 (Bedrock Edition initial target), since Bedrock natively supports gamepads, validating the controller-first model from the start.

## Decision

We have decided that controller (gamepad) input is the primary interaction model for the Minecrarch Shell. All navigation, menus, overlays, and user-facing flows must be fully operable with a standard gamepad. Keyboard and mouse are supported but are secondary — no feature may be accessible only via keyboard or mouse.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Keyboard/mouse primary | Contradicts the couch gaming appliance vision; forces desktop-style UI patterns that are wrong for the platform |
| Touch-first | Not a target use case for MinecrarchOS |
| Equal priority for both from the start | Leads to UX design compromises in both directions; controller-first with keyboard/mouse fallback produces a more coherent interface |

## Consequences

### Positive

- A clear UX constraint produces a more focused, coherent interface. Every design decision has an anchor.
- Aligns with Bedrock Edition's native gamepad support (ADR-0007), allowing the controller UX to be validated from Phase 1.
- Enables genuine couch gaming without requiring a keyboard or mouse connected to the machine.
- Drives a UI architecture that is fundamentally navigable, which also benefits accessibility.

### Negative

- All UI components in `shell/` must be built with gamepad focus traversal in mind from day one. Retrofitting focus traversal onto a UI designed for mouse navigation is very difficult.
- Testing requires physical gamepad hardware or accurate software emulation of gamepad input (uinput/evdev). Contributors without a gamepad must set up emulation.
- The overlay system must also be fully controller-navigable, which adds constraints to overlay design.

### Neutral

- The UI framework selected for `shell/` (per ADR-0011) must support controller-native focus management natively or via integration. This is a hard requirement, not a nice-to-have.
- Contributors to `shell/` must understand gamepad input handling: SDL2 gamepad API, libinput, udev rules for input devices, and evdev protocol.
- Documentation must include gamepad testing instructions, including how to use software emulation for contributors without hardware gamepads.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
