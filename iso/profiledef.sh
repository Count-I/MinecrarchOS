#!/usr/bin/env bash
# archiso profile definition for MinecrarchOS.
# Reference: https://gitlab.archlinux.org/archlinux/archiso/-/blob/master/docs/README.profile.rst

iso_name="minecrarch-os"
iso_label="MINECRARCH_$(date --utc +%Y%m)"
iso_publisher="MinecrarchOS Project <https://github.com/Count-I/MinecrarchOS>"
iso_application="MinecrarchOS Gaming Appliance"
iso_version="$(date --utc +%Y.%m.%d)"
install_dir="arch"
buildmodes=('iso')
# UEFI-only: systemd-boot is the settled bootloader (ADR-0003). No GRUB/syslinux.
bootmodes=('uefi-x86_64.systemd-boot.esp' 'uefi-x86_64.systemd-boot.eltorito')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd' '-Xcompression-level' '15')
bootstrap_tarball_compression=('zstd' '-c' '-T0' '--auto-threads=logical' '--long' '-19')

file_permissions=(
    ["/etc/shadow"]="0:0:400"
    ["/etc/sudoers.d"]="0:0:750"
    ["/etc/sudoers.d/minecrarch"]="0:0:440"
    ["/root"]="0:0:750"
    ["/root/customize_airootfs.sh"]="0:0:755"
    ["/usr/local/bin/minecrarch-install"]="0:0:755"
)
