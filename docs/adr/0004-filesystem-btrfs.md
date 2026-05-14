# ADR-0004: Filesystem — btrfs

**Date:** 2026-05-13
**Status:** Accepted
**Deciders:** Architecture team

---

## Context

The filesystem choice affects: snapshot and rollback capability (a Phase 3 goal), subvolume-based data isolation, copy-on-write semantics for safe updates, and storage efficiency for an asset-heavy game platform.

MinecrarchOS's Phase 3 plans explicitly include rollback support and atomic-style updates. These features depend fundamentally on a filesystem with snapshot primitives — they cannot be bolted onto a traditional filesystem without additional tooling that adds complexity and failure modes. Minecraft's world data and asset cache are also large and compressible, making transparent filesystem-level compression valuable.

## Decision

We will use btrfs as the filesystem for MinecrarchOS installations.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| ext4 | No native snapshot support; rollback requires external tooling (LVM, block-level snapshotting) that adds significant complexity |
| XFS | No snapshot support; optimized for large sequential file throughput but lacks the COW semantics needed for rollback and atomic updates |
| ZFS | Excellent snapshot support, but the CDDL license is not GPL-compatible; it cannot be shipped in-kernel and requires DKMS overhead on every kernel update |
| f2fs | Flash-optimized; good for SSDs but lacks subvolume and snapshot semantics required for rollback |

## Consequences

### Positive

- Native snapshots enable Phase 3 rollback without additional infrastructure — snapshot before update, revert if the update fails.
- Subvolumes cleanly isolate root (`@`), home (`@home`), and game data, allowing targeted rollbacks.
- Transparent zstd compression reduces storage usage for Minecraft world data, asset caches, and modpack files.
- COW semantics make snapshot-before-update patterns safe and storage-efficient (only changed blocks are duplicated).

### Negative

- btrfs has a history of instability on multi-disk RAID configurations. MinecrarchOS is a single-disk appliance, so RAID is out of scope and this risk does not apply.
- btrfs requires periodic maintenance (scrub, balance) to remain healthy. A system service must handle this automatically.
- Contributors unfamiliar with btrfs subvolume layouts need onboarding before contributing to the packaging or ISO components.

### Neutral

- The ISO installer must define an explicit subvolume layout. At minimum: `@` for root and `@home` for user data. A `@snapshots` subvolume for rollback storage is likely needed and should be defined before the Phase 3 implementation begins.
- The subvolume layout is itself a sub-decision not made by this ADR; it should be captured in a separate ADR when the ISO installer is implemented.

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
