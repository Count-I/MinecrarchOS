# ADR-0005: Session Compositor — Gamescope

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

Wayland requires a compositor. For a gaming appliance, the compositor is not just a display server — it is the session boundary, the input router, the framerate controller, and the fullscreen ownership mechanism. The entire MinecrarchOS UX model depends on the compositor: the shell runs inside the compositor's session, game launching happens within the compositor's authority, and overlays are rendered through the compositor.

Gamescope is Valve's purpose-built gaming Wayland compositor, used in production by SteamOS. It is designed specifically for the use case MinecrarchOS is solving: a single game owns the screen, the compositor handles upscaling and frame pacing, and a shell UI can be embedded alongside the game.

## Decision

We will use Gamescope as the session compositor.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Hyprland | General-purpose tiling compositor; not designed for single-app gaming sessions; no native upscaling, frame pacing, or latency features |
| sway | General-purpose wlroots-based compositor; same fundamental mismatch as Hyprland for a gaming appliance session model |
| weston | Reference compositor; not production-suitable for a gaming appliance |
| KWin | KDE-native; brings in a large KDE dependency chain; not gaming-optimized |
| Building a custom compositor | Enormous scope; not justified when Gamescope already solves the exact problem |

## Consequences

### Positive

- Gamescope provides native FSR/NIS upscaling, frame limiting, HDR support, and VRR/FreeSync support out of the box.
- The gaming session model is exactly what MinecrarchOS needs: one game owns the compositor session.
- Input handling is gaming-optimized with low-latency event processing.
- Battle-tested at scale by SteamOS; Valve actively maintains and improves it.
- Available in official Arch repositories.

### Negative

- Gamescope is maintained by Valve; its development velocity is tied to Valve's priorities. MinecrarchOS must track Gamescope releases and test compatibility with each significant release.
- Gamescope's embedded compositor mode and IPC interface must be understood deeply by shell contributors; it is not extensively documented outside the source code.

### Neutral

- The Minecrarch Shell runs inside Gamescope's session as a Wayland client. Gamescope handles screen composition; the shell does not draw directly to DRM/KMS.
- Contributors to `shell/` must understand Gamescope's nested compositor mode and how to embed a Wayland application within it.
- Studying the SteamOS compositor source and Valve's GDC/XDC talks on Gamescope is required background for shell contributors.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
