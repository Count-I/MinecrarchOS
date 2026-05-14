#!/usr/bin/env bash
# Architectural dependency boundary checker for MinecrarchOS
# Validates that no component imports another component it is forbidden from importing.
# Runs in CI on every PR that touches a Cargo.toml.
# Exit code 0 = no violations. Exit code 1 = violations found.

set -euo pipefail

VIOLATIONS=0

fail() {
    echo "VIOLATION: $1" >&2
    VIOLATIONS=$((VIOLATIONS + 1))
}

check_cargo_toml() {
    local file="$1"
    local component="$2"
    shift 2
    local forbidden=("$@")

    if [ ! -f "$file" ]; then
        return 0
    fi

    for dep in "${forbidden[@]}"; do
        # Match only dependency declarations (lines like `crate-name = ...` or `crate-name = { ... }`)
        # Excludes [package] name = "..." lines by requiring the crate name at line start
        if grep -qE "^[[:space:]]*${dep}[[:space:]]*(=|\\.)" "$file"; then
            fail "${component} (${file}) must not depend on ${dep}"
        fi
    done
}

# ── shell/ must not import any service or runtime crate ──────────────────────
check_cargo_toml "shell/Cargo.toml" "shell" \
    "minecrarch-modpack-manager" \
    "minecrarch-overlay" \
    "minecrarch-logging" \
    "minecrarch-updater" \
    "minecrarch-runtime"

# ── shared/ must not import any component crate ──────────────────────────────
check_cargo_toml "shared/Cargo.toml" "shared" \
    "minecrarch-shell" \
    "minecrarch-modpack-manager" \
    "minecrarch-overlay" \
    "minecrarch-logging" \
    "minecrarch-updater" \
    "minecrarch-runtime"

# ── runtime/ must not import shell or service crates ─────────────────────────
check_cargo_toml "runtime/Cargo.toml" "runtime" \
    "minecrarch-shell" \
    "minecrarch-modpack-manager" \
    "minecrarch-overlay" \
    "minecrarch-logging" \
    "minecrarch-updater"

# ── services must not import each other ──────────────────────────────────────
SERVICES=(
    "minecrarch-shell"
    "minecrarch-modpack-manager"
    "minecrarch-overlay"
    "minecrarch-logging"
    "minecrarch-updater"
    "minecrarch-runtime"
)

SERVICE_DIRS=(
    "services/modpack-manager"
    "services/overlay"
    "services/logging"
    "services/updater"
)

for dir in "${SERVICE_DIRS[@]}"; do
    component=$(basename "$dir")
    file="${dir}/Cargo.toml"
    # Each service cannot import shell or any other service
    # (it may import shared and runtime)
    check_cargo_toml "$file" "services/${component}" \
        "minecrarch-shell" \
        "minecrarch-modpack-manager" \
        "minecrarch-overlay" \
        "minecrarch-logging" \
        "minecrarch-updater"
    # Exception: modpack-manager and updater may import runtime
    # overlay and logging may NOT import runtime
    if [[ "$component" == "overlay" || "$component" == "logging" ]]; then
        check_cargo_toml "$file" "services/${component}" "minecrarch-runtime"
    fi
done

# ── Result ───────────────────────────────────────────────────────────────────
if [ "$VIOLATIONS" -gt 0 ]; then
    echo ""
    echo "Found $VIOLATIONS architectural dependency violation(s)." >&2
    echo "See docs/architecture/layers.md for allowed dependency rules." >&2
    exit 1
else
    echo "Dependency boundary check passed. No violations found."
fi
