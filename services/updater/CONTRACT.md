# Service Contract: Updater

**D-Bus bus name:** `org.minecrarch.Updater`
**Object path:** `/org/minecrarch/Updater`
**Interface:** `org.minecrarch.Updater`
**IPC reference:** [`docs/ipc.md — Updater`](../../docs/ipc.md)

---

## Owned Responsibilities

- Platform update checking: querying the MinecrarchOS pacman repository for available updates.
- Update application: executing `pacman` update operations with btrfs snapshot safety.
- Snapshot management: creating btrfs snapshots before updates, listing available snapshots, managing the `@snapshots` subvolume.
- Rollback: activating a previous snapshot as the active root (requires systemd-boot entry management).
- `UpdateAvailable`, `UpdateProgress`, `UpdateApplied`, `UpdateFailed`, `RollbackComplete` signal emission.
- D-Bus property maintenance: `CurrentVersion`, `UpdateAvailable`, `PendingUpdateVersion`, `LastSnapshotTimestamp`.

## Explicit Non-Responsibilities

This service must NOT:

- Update Minecraft itself (the game runtime is managed by `services/modpack-manager` via Prism).
- Update individual modpacks or instances (belongs to modpack-manager).
- Reboot the system directly. It emits `UpdateApplied(reboot_required=true)` and the shell presents the reboot decision to the user.
- Modify the shell, overlay, or other service binaries outside the normal pacman update path.
- Apply updates while a game is running. If `ActiveInstance` is non-empty on ModpackManager, `ApplyUpdate` must return `org.minecrarch.Error.GameRunning`.

## Lifecycle

**Startup:**
1. Register `org.minecrarch.Updater` on the user session bus.
2. Read current platform version from `/etc/minecrarch-release`.
3. Schedule periodic update checks (every 6 hours by default, configurable).
4. Ready to accept method calls.

**Shutdown:**
1. Cancel any in-progress update check (downloads may be abandoned; they will resume on next start if pacman's partial download cache is preserved).
2. Do NOT cancel an in-progress snapshot or pacman transaction — a partially applied update is dangerous. If shutdown is requested during an active `ApplyUpdate`, emit `UpdateFailed(reason="interrupted")`, attempt to roll back the partial transaction, then exit.
3. Exit cleanly.

**Restart policy:** `Restart=on-failure` with `RestartSec=10s`. The updater is non-critical for daily operation. The shell disables the update UI when the service is unavailable.

## Update Safety Protocol

This protocol is mandatory. Deviating from it is an architectural failure:

```text
1. Check that no game is currently running (ModpackManager.ActiveInstance == "")
2. Create btrfs snapshot of @ subvolume:
     btrfs subvolume snapshot / /mnt/snapshots/@pre-update-YYYYMMDD-HHMMSS
3. Add systemd-boot entry pointing to the snapshot (for emergency rollback)
4. Execute pacman update
5. Verify update succeeded (package database consistent, no broken deps)
6. If verification fails: activate snapshot boot entry, emit UpdateFailed(snapshot_preserved=true)
7. If verification succeeds: emit UpdateApplied(reboot_required=true)
```

The snapshot must exist before any pacman command runs. If snapshot creation fails, `ApplyUpdate` must return `org.minecrarch.Error.SnapshotFailed` without running pacman.

## Failure Behavior

- `CheckForUpdates` fails (network unreachable): log the failure, return `false` (no update available), emit nothing.
- `ApplyUpdate` fails mid-transaction: attempt to restore the pre-update snapshot, emit `UpdateFailed(snapshot_preserved=true or false)`.
- `Rollback` fails: emit `RollbackFailed` with reason; log extensively to journald; do not attempt further automatic recovery (this is a serious failure requiring human intervention).
- Service crash during update: on restart, detect the partial update state by checking if the pre-update snapshot exists and if the pacman database is consistent. If inconsistent, emit `UpdateFailed` and present rollback option to the shell.

## State Ownership

This service owns:
- Knowledge of the current platform version.
- Knowledge of available updates (cached from last check).
- The list and metadata of all btrfs snapshots in `@snapshots`.
- The systemd-boot entry for the last update's rollback target.

This service does NOT own:
- Modpack or instance version information (modpack-manager owns this).
- The decision to reboot (the shell presents this to the user after receiving `UpdateApplied`).
- The btrfs snapshot subvolume itself (`@snapshots` exists; this service manages entries within it).
