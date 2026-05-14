# ADR-0006: No Desktop Environment

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

A desktop environment (GNOME, KDE, XFCE, etc.) provides window management, a settings UI, file managers, notification daemons, and a general-purpose application model. MinecrarchOS is a single-purpose gaming appliance. There is no use case for a desktop in the intended boot flow, which goes directly: `systemd → autologin → Gamescope session → Minecrarch Shell → Minecraft`.

Including a desktop environment — even as a hidden or secondary mode — would contradict the appliance identity, add dependency bloat, and introduce competing session management that conflicts with Gamescope's ownership of the display.

## Decision

We will not include a desktop environment in MinecrarchOS. Gamescope owns the Wayland session directly, launched from systemd via an autologin unit.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| GNOME (session for launch, then hidden) | Adds enormous dependency chain; a DE session between systemd and Gamescope adds unnecessary overhead and complexity |
| KDE Plasma with Gaming Mode | SteamOS uses KDE for its desktop mode; MinecrarchOS has no desktop mode — it is a single-purpose appliance, not a dual-mode system |
| Minimal WM (openbox, etc.) as escape hatch | Blurs the single-purpose appliance identity; creates an implicit promise of a fallback desktop that is not part of the platform's design |

## Consequences

### Positive

- Dramatically smaller base system footprint; fewer packages to maintain and audit.
- Faster boot time — no DE startup services, no desktop daemons.
- No competing input or window management; Gamescope has full authority over the display.
- Platform identity is clear and unambiguous: this is an appliance, not a desktop.

### Negative

- No built-in fallback UI if Gamescope or the Shell fails. Recovery from a broken session requires TTY access or a separately designed recovery environment.
- The recovery environment (Phase 3) must be designed explicitly as an alternative boot mode, not as a fallback to a desktop session.
- Contributors cannot use familiar DE-based debugging tools within the running session; debugging relies on SSH, TTY, and journald.

### Neutral

- All system configuration UI must be built into the Minecrarch Shell itself. There is no DE settings panel.
- Developer and debug workflows rely on SSH access, serial TTY, and `journalctl`. This must be documented clearly in the contributing guide.
- The recovery environment (Phase 3) is a separate architectural concern and should be defined in its own ADR when the time comes.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
