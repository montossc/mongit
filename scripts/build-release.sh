#!/usr/bin/env bash
# scripts/build-release.sh
# One-command release build for mongit.
# Runs verification checks, builds frontend + Rust, and reports artifacts.
#
# Usage:
#   ./scripts/build-release.sh           # Full build with checks
#   ./scripts/build-release.sh --skip-checks  # Skip verification, build only

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

SKIP_CHECKS=false
if [ "${1:-}" = "--skip-checks" ]; then
    SKIP_CHECKS=true
fi

echo "=========================================="
echo "  mongit release build"
echo "=========================================="
echo ""

# ── Step 1: Version sync ─────────────────────────────────────────────────

echo "[1/5] Checking version sync..."
bash scripts/check-version-sync.sh
echo ""

# ── Step 2: Pre-build checks ─────────────────────────────────────────────

if [ "$SKIP_CHECKS" = false ]; then
    echo "[2/5] Running svelte-check..."
    pnpm check
    echo ""

    echo "[3/5] Running cargo check..."
    (cd src-tauri && cargo check --release)
    echo ""
else
    echo "[2/5] Skipping svelte-check (--skip-checks)"
    echo "[3/5] Skipping cargo check (--skip-checks)"
    echo ""
fi

# ── Step 3: Build ─────────────────────────────────────────────────────────

echo "[4/5] Building release artifacts..."
echo "  This may take several minutes (LTO + strip enabled)."
echo ""
pnpm tauri build
echo ""

# ── Step 4: Report artifacts ──────────────────────────────────────────────

BUNDLE_DIR="$REPO_ROOT/src-tauri/target/release/bundle"
echo "[5/5] Build complete. Artifacts:"
echo ""

if [ -d "$BUNDLE_DIR/macos" ]; then
    APP_PATH=$(find "$BUNDLE_DIR/macos" -name "*.app" -maxdepth 1 2>/dev/null | head -1)
    if [ -n "$APP_PATH" ]; then
        APP_SIZE=$(du -sh "$APP_PATH" | cut -f1)
        echo "  .app:  $APP_PATH ($APP_SIZE)"
    fi
fi

if [ -d "$BUNDLE_DIR/dmg" ]; then
    DMG_PATH=$(find "$BUNDLE_DIR/dmg" -name "*.dmg" -maxdepth 1 2>/dev/null | head -1)
    if [ -n "$DMG_PATH" ]; then
        DMG_SIZE=$(du -sh "$DMG_PATH" | cut -f1)
        echo "  .dmg:  $DMG_PATH ($DMG_SIZE)"
    fi
fi

echo ""
echo "=========================================="
echo "  Release build finished successfully"
echo "=========================================="
