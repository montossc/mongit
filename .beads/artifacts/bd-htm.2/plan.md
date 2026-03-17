# Plan: bd-htm.2 — Rust watcher service and debounce lifecycle

## Execution Strategy

Sequential execution (2 waves) because both tasks modify `src-tauri/src/watcher.rs`.

## Wave 1 — Lifecycle hardening

### Task W1-T1: Harden watcher lifecycle contract

**Goal**
- Keep watcher ownership in Rust while making start/replace/stop behavior more predictable.

**Planned changes**
- Add validated/canonical repo path normalization before starting watcher.
- Persist watcher session metadata (`path`) alongside watcher handle.
- No-op when `watch_repo` is called for the same canonical path.
- Keep coarse `repo-changed` and `repo-watcher-error` event contract unchanged.

**Files**
- `src-tauri/src/watcher.rs`
- `src-tauri/src/lib.rs` (only if managed state type changes)

**Verification (incremental)**
- `cargo check`
- `grep -E "repo-changed|repo-watcher-error" src-tauri/src/watcher.rs`

## Wave 2 — Rust verification coverage

### Task W2-T1: Add watcher lifecycle/debounce tests

**Goal**
- Expand backend-side tests for path normalization and lifecycle boundaries without adding frontend refresh logic.

**Planned changes**
- Add tests for invalid/non-repo path rejection and valid repo acceptance.
- Add tests for debounce constant and filtering edge cases.
- Keep focus on backend behavior in `watcher.rs` tests.

**Files**
- `src-tauri/src/watcher.rs`

**Verification (incremental)**
- `cargo test watcher --lib`

## Final Full Verification (post-wave)

- `pnpm check`
- `cargo check`
- `cargo test watcher --lib`

## Commit Plan

1. `feat(bead-bd-htm.2): harden watcher lifecycle contract`
2. `test(bead-bd-htm.2): add watcher lifecycle and debounce tests`
