# Release Artifact Pipeline

**Bead:** bd-1sy.1
**Type:** task
**Status:** In Progress

---

## Problem Statement

`pnpm tauri build` is the only build path, but it lacks macOS entitlements, has misconfigured bundle targets ("all" instead of macOS-only for V1), no version sync verification, and no automation script. This means:
- Builds may fail at runtime due to missing entitlements (file system access, FSEvents)
- Unnecessary Windows/Linux artifacts are attempted on macOS
- Version drift between `package.json`, `Cargo.toml`, and `tauri.conf.json` goes undetected
- No single command produces a verified, distributable `.app` + `.dmg`

## Scope

### In-Scope
1. **macOS entitlements** — `src-tauri/entitlements.plist` with file system + network permissions
2. **Bundle target refinement** — change `targets: "all"` to macOS-specific (`dmg`, `app`)
3. **Version sync script** — `scripts/check-version-sync.sh` to verify all three files match
4. **Release build script** — `scripts/build-release.sh` that runs checks → frontend build → tauri build → reports artifacts
5. **Test build** — verify `pnpm tauri build` produces clean `.app` and `.dmg`

### Out-of-Scope
- Code signing with Apple Developer certificate (requires account, separate concern)
- Notarization (requires Apple Developer account)
- CI/CD GitHub Actions pipeline (future bead)
- Windows/Linux builds (V1 is macOS-only)
- Homebrew cask formula
- Auto-update mechanism

## Proposed Solution

### 1. entitlements.plist
Standard macOS entitlements for a desktop Git client:
- `com.apple.security.files.user-selected.read-write` — file system access via dialog
- `com.apple.security.network.client` — outbound network for git operations

### 2. Bundle targets
Change `tauri.conf.json` bundle targets from `"all"` to `["dmg", "app"]` — macOS only for V1.

### 3. Version sync check
Shell script that extracts version from all three config files and fails if they differ.

### 4. Release build script
Single entry point: `scripts/build-release.sh`
- Runs version sync check
- Runs `pnpm check` (svelte-check)
- Runs `cargo check` in src-tauri/
- Runs `pnpm tauri build`
- Reports artifact paths and sizes

## Success Criteria
- [ ] `pnpm tauri build` produces a clean `.app` bundle in `src-tauri/target/release/bundle/macos/`
- [ ] `pnpm tauri build` produces a `.dmg` in `src-tauri/target/release/bundle/dmg/`
- [ ] entitlements.plist exists with correct permissions
- [ ] `scripts/check-version-sync.sh` passes when versions match, fails when they differ
- [ ] `scripts/build-release.sh` runs end-to-end and reports artifacts
- [ ] Version is consistent across package.json, Cargo.toml, tauri.conf.json (all 0.1.0)

## Affected Files
- `src-tauri/tauri.conf.json` — bundle targets + entitlements reference
- `src-tauri/entitlements.plist` — NEW
- `scripts/build-release.sh` — NEW
- `scripts/check-version-sync.sh` — NEW

## Risks
- `pnpm tauri build` may require Xcode Command Line Tools installed
- DMG creation may require `create-dmg` or similar tool (Tauri handles this natively)
- Build time may be long (LTO + strip enabled in release profile)

---

## Metadata

**Parent:** bd-1sy
