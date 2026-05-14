# Architecture Decision Records — MinecrarchOS

Architecture Decision Records for MinecrarchOS — a dedicated Linux gaming appliance platform built around Minecraft.

## What Are ADRs?

An Architecture Decision Record captures a significant architectural decision: what was decided, why it was decided, what alternatives were considered, and what the consequences are. ADRs are immutable once accepted — they are a historical record, not a living document. When a decision changes, a new ADR supersedes the old one rather than rewriting it.

MinecrarchOS uses ADRs because the project involves many non-obvious architectural choices driven by the gaming appliance context. ADRs give contributors the context they need to understand the system's shape and avoid re-litigating settled decisions.

## Status Definitions

| Status | Meaning |
|---|---|
| **Accepted** | Decision is made and in effect. Implementation should follow this decision. |
| **Proposed** | Decision is under discussion. Not yet final. Do not implement until Accepted. |
| **Deprecated** | Decision was Accepted but is no longer relevant (e.g., the component was removed). |
| **Superseded** | Decision was Accepted but replaced by a newer ADR (linked in the Superseded ADR). |

## ADR Index

| Number | Title | Status | Date |
|---|---|---|---|
| [ADR-0001](./0001-base-system-arch-linux.md) | Base System: Arch Linux | Accepted | 2026-05-13 |
| [ADR-0002](./0002-kernel-linux-zen.md) | Kernel: linux-zen | Accepted | 2026-05-13 |
| [ADR-0003](./0003-bootloader-systemd-boot.md) | Bootloader: systemd-boot | Accepted | 2026-05-13 |
| [ADR-0004](./0004-filesystem-btrfs.md) | Filesystem: btrfs | Accepted | 2026-05-13 |
| [ADR-0005](./0005-session-compositor-gamescope.md) | Session Compositor: Gamescope | Accepted | 2026-05-13 |
| [ADR-0006](./0006-no-desktop-environment.md) | No Desktop Environment | Accepted | 2026-05-13 |
| [ADR-0007](./0007-bedrock-edition-initial-target.md) | Initial Game Target: Minecraft Bedrock Edition | Accepted | 2026-05-13 |
| [ADR-0008](./0008-launcher-backend-prism.md) | Launcher Backend: Prism Launcher (Initial) | Accepted | 2026-05-13 |
| [ADR-0009](./0009-shell-as-orchestration-layer.md) | Shell as Orchestration Layer (Not Monolithic Backend) | Accepted | 2026-05-13 |
| [ADR-0010](./0010-controller-first-ux.md) | Controller-First UX as Primary Interaction Model | Accepted | 2026-05-13 |
| [ADR-0011](./0011-shell-implementation-language.md) | Shell Implementation Language: Rust | Accepted | 2026-05-13 |
| [ADR-0012](./0012-ipc-mechanism.md) | IPC Mechanism: D-Bus (User Session Bus) | Accepted | 2026-05-13 |

## Proposing a New ADR

1. Copy the template below into a new file: `docs/adr/NNNN-short-title.md` where `NNNN` is the next available number (zero-padded to 4 digits).
2. Set Status to `Proposed`.
3. Open a PR. The ADR is discussed in the PR review.
4. When consensus is reached, Status is updated to `Accepted` and the PR is merged.
5. Add a row to the index table above.

### ADR Template

```markdown
# ADR-NNNN: [Title]

**Date:** YYYY-MM-DD
**Status:** Proposed
**Deciders:** Architecture team

---

## Context

[Why this decision is needed. What forces are at play. What makes this decision non-trivial.]

## Decision

[What is decided. Start with "We will use..." or "We have decided to..."]

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| ... | ... |

## Consequences

### Positive
- ...

### Negative
- ...

### Neutral
- ...

---

*This ADR is part of the [MinecrarchOS ADR index](./README.md).*
```
