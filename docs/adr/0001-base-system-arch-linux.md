# ADR-0001: Base System — Arch Linux

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

MinecrarchOS is a single-purpose gaming appliance. The base Linux distribution determines: package availability, update model (rolling vs. point-release), build tooling ecosystem, and how close to upstream packages will be. A gaming appliance needs a very thin base — no bundled desktop, no bloat — and needs recent drivers and software.

The project requires: a packaging format that maps to an installable ISO pipeline, a gaming-optimized software stack (recent Mesa, Pipewire, Wayland compositors), and architectural precedent from similar platforms. The planned output includes installable ISOs and custom PKGBUILDs, both of which are first-class Arch ecosystem artifacts. MinecrarchOS draws direct architectural inspiration from SteamOS, which is Arch-based in its gaming mode layer.

## Decision

We will use Arch Linux as the base system.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Debian/Ubuntu | Older package cadence; graphics stack (Mesa, Pipewire, Wayland compositors) lags significantly behind rolling releases |
| Fedora | RPM ecosystem adds friction to the PKGBUILD-native packaging pipeline planned for the project |
| Gentoo | Per-machine compilation model conflicts with the goal of distributable, reproducible ISOs |
| NixOS | The Nix model is powerful but introduces a steep contributor onboarding barrier and significant complexity to the ISO build pipeline |
| SteamOS (Valve's) | A product, not a base for another product; not suitable as a foundation |

## Consequences

### Positive

- Rolling release gives permanent access to the latest Mesa, linux-zen, Gamescope, and Pipewire without version-pinning friction.
- AUR gives access to Minecraft-adjacent tooling and community packages.
- PKGBUILD is the natural packaging format for the project's ISO and distribution pipeline.
- SteamOS is Arch-based; Valve's public engineering work on their gaming session model applies directly to MinecrarchOS.
- `archiso` (the Arch ISO builder) is a first-class, well-documented tool for exactly this kind of project.

### Negative

- Arch's rolling nature means the project must define its own stability guarantees and snapshot/update cadence. Upstream updates are not gated.
- No LTS kernel policy by default — kernel update management must be handled explicitly by MinecrarchOS tooling.

### Neutral

- Contributors will need Arch Linux familiarity: pacman, PKGBUILD basics, AUR workflow.
- All packaging will use `makepkg`/PKGBUILD format.
- The project must independently decide its snapshot and update cadence; Arch itself does not define one.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
