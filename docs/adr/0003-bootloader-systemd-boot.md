# ADR-0003: Bootloader — systemd-boot

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

The bootloader is part of the appliance identity of MinecrarchOS. The boot sequence is `UEFI → bootloader → linux-zen → systemd`. As a single-purpose appliance targeting modern hardware, UEFI support is required and legacy BIOS support is not. The bootloader must be minimal, UEFI-native, and integrate cleanly with the rest of the system's init ecosystem.

Future phases include recovery environments (Phase 3), rollback support (Phase 3), and atomic updates — all of which interact with the bootloader via boot entry management. The bootloader must be automatable from update and recovery scripts without complex configuration syntax.

## Decision

We will use systemd-boot as the bootloader.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| GRUB2 | Heavier; more complex configuration; BIOS legacy support is not needed for this project; update automation is more complex with grub-mkconfig |
| rEFInd | Good UEFI UX but includes a graphical boot picker — not desired for an appliance that should boot straight to Minecraft with no visible boot UI |
| syslinux | Lacks modern UEFI-native features; limited community momentum |

## Consequences

### Positive

- systemd-boot is UEFI-only, which matches the target hardware profile exactly.
- Integrates naturally with systemd's kernel-install hooks for automated boot entry management during kernel updates.
- Boot configuration is trivial text files in the ESP — easy to generate and modify from update and rollback scripts.
- Minimal/no boot UI aligns with the fast boot-to-game philosophy; the appliance boots directly without user intervention.

### Negative

- No BIOS/legacy boot support — MinecrarchOS requires UEFI hardware. Older machines without UEFI are explicitly out of scope.
- ESP partition management is a hard requirement; the ISO installer must configure the ESP layout correctly for systemd-boot.

### Neutral

- Recovery and rollback features (Phase 3) must be implemented by managing boot entries in the ESP directory — no additional bootloader mechanism is available.
- All documentation and contributor guides must assume UEFI. Hardware compatibility notes must reflect this limitation.
- linux-zen boot entries must reference `/boot/vmlinuz-linux-zen` and `/boot/initramfs-linux-zen.img`, not the default linux kernel paths.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
