# ADR-0002: Kernel — linux-zen

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

The kernel choice directly affects: input latency, scheduler behavior, gaming workload performance, Wayland/DRM behavior, and the platform's ability to deliver the "console-like" low-latency experience described in the project vision. For a gaming appliance, scheduler and I/O tuning matter as much as driver availability.

The linux-zen kernel is the Arch-maintained gaming-oriented kernel variant, available from official Arch repositories. It ships with patches targeting low-latency workloads, an alternative scheduler configuration, and I/O tuning — all without requiring a third-party repository or a custom build pipeline.

## Decision

We will use linux-zen as the kernel for MinecrarchOS.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| linux (mainline Arch) | No gaming-specific scheduler tuning; misses latency-sensitive optimizations that matter for input responsiveness |
| linux-lts | Older kernel; slower DRM and driver updates; fundamentally at odds with a rolling gaming platform |
| linux-hardened | Security-hardened patches conflict with gaming performance needs |
| linux-cachyos (CachyOS kernel) | Equivalent goals but introduces a dependency on a third-party repository; linux-zen achieves the same result from official Arch repos |
| Custom kernel (from scratch) | Maintainability burden is too high for a small project; not justified when linux-zen solves the problem |

## Consequences

### Positive

- Lower scheduler latency and better gaming workload characteristics out of the box.
- Available in official Arch repositories — no third-party repo needed.
- MUQ (Multiple Queue) and BFQ I/O scheduler tunings benefit Minecraft's asset-heavy I/O patterns.
- Reduces input latency for gamepad and peripheral responsiveness, consistent with the controller-first UX goal.

### Negative

- linux-zen follows Arch's rolling cadence, not a fixed LTS cycle; ABI changes can affect out-of-tree modules if any are added in the future.
- Contributors must install `linux-zen` and `linux-zen-headers` on development machines; the default `linux` package is not sufficient.

### Neutral

- Bootloader configuration (systemd-boot, per ADR-0003) must reference linux-zen vmlinuz and initramfs paths, not the default `linux` paths.
- The ISO build must be configured to install linux-zen, not the default Arch kernel.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
