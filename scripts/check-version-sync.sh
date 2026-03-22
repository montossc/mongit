#!/usr/bin/env bash
# scripts/check-version-sync.sh
# Verify version consistency across package.json, Cargo.toml, and tauri.conf.json.
# Exits 0 if all match, 1 otherwise.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Extract versions from each source
PKG_VERSION=$(grep '"version"' "$REPO_ROOT/package.json" | head -1 | sed 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
CARGO_VERSION=$(grep '^version' "$REPO_ROOT/src-tauri/Cargo.toml" | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
TAURI_VERSION=$(grep '"version"' "$REPO_ROOT/src-tauri/tauri.conf.json" | head -1 | sed 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')

echo "Version check:"
echo "  package.json:     $PKG_VERSION"
echo "  Cargo.toml:       $CARGO_VERSION"
echo "  tauri.conf.json:  $TAURI_VERSION"

if [ "$PKG_VERSION" = "$CARGO_VERSION" ] && [ "$CARGO_VERSION" = "$TAURI_VERSION" ]; then
    echo ""
    echo "All versions match: $PKG_VERSION"
    exit 0
else
    echo ""
    echo "ERROR: Version mismatch detected!"
    exit 1
fi
