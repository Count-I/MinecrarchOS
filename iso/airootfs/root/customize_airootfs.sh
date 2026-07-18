#!/usr/bin/env bash
# Runs inside the archiso chroot during ISO build (as root).
# Sets up the minecrarch appliance user and enables platform services.
set -euo pipefail

# ── locale ────────────────────────────────────────────────────────────────────
locale-gen

# ── minecrarch user ───────────────────────────────────────────────────────────
# The appliance user. No password — autologin handles authentication-free boot.
useradd --create-home --groups audio,video,input,wheel --shell /bin/bash minecrarch

# Enable linger so systemd starts user services at boot without an interactive
# login session (required for the autologin → Gamescope session flow).
mkdir -p /var/lib/systemd/linger
touch /var/lib/systemd/linger/minecrarch

# ── sudoers ───────────────────────────────────────────────────────────────────
# Grant minecrarch NOPASSWD access to the specific commands the updater service
# needs for privileged operations (pacman, btrfs snapshot management).
cat > /etc/sudoers.d/minecrarch <<'EOF'
minecrarch ALL=(ALL) NOPASSWD: /usr/bin/pacman, /usr/bin/btrfs
EOF
chmod 440 /etc/sudoers.d/minecrarch

# ── NetworkManager ────────────────────────────────────────────────────────────
systemctl enable NetworkManager.service

# ── MinecrarchOS packages ─────────────────────────────────────────────────────
# minecrarch-core and minecrarch-session are installed from the custom
# [minecrarch] pacman repository configured in pacman.conf (P3-D5).
# The packages are already listed in packages.x86_64; no action needed here.

echo "customize_airootfs.sh complete"
