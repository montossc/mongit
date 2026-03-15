# Bundled Git Strategy for mongit (Phase-Gated Resolution)

**Date:** 2026-03-15  
**Status:** Decision recorded for Foundation/MVP; bundling deferred to V1.0 pre-launch

---

## Executive Summary

mongit should adopt a **phase-gated Git binary strategy**:

- **Foundation/MVP:** use **system Git** with strict version gate `>= 2.35`.
- **V1.0 pre-launch:** introduce **bundled dugite-native Git** and move it to highest resolver priority.

This balances short-term delivery speed and app-size constraints with a clean path to deterministic runtime control before V1.0 launch.

---

## Problem Statement

mongit write operations currently shell out using:

```rust
Command::new("git")
```

This appears in `src-tauri/src/git/cli.rs` (`run_git`). Resolution is delegated to whatever `git` is first on `PATH`.

### Why this is a problem

1. **Non-determinism:** behavior varies by machine-installed Git version and configuration.
2. **Support complexity:** bugs become environment-specific and difficult to reproduce.
3. **Unsafe assumptions:** command/features may not exist or differ on older Git versions.
4. **No migration seam:** bundling later is harder without a dedicated resolver abstraction.

mongit needs a deterministic, explicit Git resolution contract for all write/mutation operations.

---

## Strategy: Phase-Gated Resolution

### Design principle

All write-path command execution must use a resolved absolute Git binary path from a single `GitResolver` module. No direct `Command::new("git")` calls.

### Phase 1 (Foundation/MVP): System Git Only

Priority chain:

1. `GIT_EXECUTABLE` env var override
2. System PATH lookup via `which::which_global("git")`
3. Error with install instructions

Version gate:

- Require `git --version >= 2.35`
- Fail fast when below floor, returning detected version and remediation guidance

### Phase 2 (V1.0 Pre-Launch): Bundled Git Added

Add dugite-native binary inside app bundle, then update priority chain:

1. Bundled Git path (inside `.app` resources/sidecar)
2. `GIT_EXECUTABLE` env var override
3. System PATH via `which::which_global("git")`
4. Error with install instructions

This keeps the API stable and makes migration low-risk.

---

## Proposed Resolver Contract

```rust
pub struct ResolvedGit {
    pub path: std::path::PathBuf,
    pub version: semver::Version,
    pub source: GitSource,
}

pub enum GitSource {
    Bundled,
    EnvOverride,
    SystemPath,
}

pub struct GitResolver;

impl GitResolver {
    pub fn resolve() -> Result<ResolvedGit, GitResolveError>;
}
```

Execution path should become:

```rust
Command::new(resolved_git.path)
```

Operational notes:

- Validate version during resolve.
- Cache resolved result for process lifetime.
- Log source/path/version for diagnostics.

---

## Binary Size Analysis

| Option | Additional App Size | Notes |
|---|---:|---|
| dugite-native arm64 macOS (with GCM) | ~60 MB | Large payload; likely exceeds current size budget |
| dugite-native arm64 macOS (without GCM) | ~15–20 MB | Smaller but still significant |
| System Git (Xcode CLT) | 0 MB | No bundle increase; Ventura-era CLT ships modern Git (>= 2.39) |
| System Git (Homebrew) | 0 MB | User-installed latest stable, no bundle increase |

**Implication:** deferring bundling preserves current `.app` target (`< 25 MB`) during Foundation/MVP.

---

## Packaging Implications (Tauri)

### 1) Sidecar naming pattern

Tauri sidecar assets follow target-triple naming, e.g.:

- `git-aarch64-apple-darwin`

### 2) Capability/config wiring

`tauri.conf.json` requires sidecar/process capability configuration for executing bundled binary.

### 3) Code signing

Bundled Git executable must be signed with the same certificate as the `.app` for Gatekeeper/notarization compliance.

### 4) Size budget pressure

Given current `< 25 MB` app target, bundling is deferred until V1.0 pre-launch.

---

## Version Gate Rationale (`>= 2.35`)

| Capability | Git Version | Why it matters |
|---|---:|---|
| `git switch` introduced | 2.23 | Modern branch switching UX |
| Sparse checkout improvements | 2.25 | Better behavior on larger repos |
| Worktree improvements | 2.35 | Better baseline for advanced local workflows |
| Xcode CLT on Ventura-era macOS | >= 2.39 | Confirms `2.35` is conservative and practical |

Why not lower:

- Older versions increase behavior variance and test matrix burden.
- Deterministic UX improves with a modern floor.

---

## Prior Art

| Product | Strategy | Notes |
|---|---|---|
| GitHub Desktop | Bundled dugite-native | Ships own Git runtime for determinism |
| GitButler | System Git | Uses `gix::path::env::exe_invocation()` path strategy |
| Fork | System Git | Depends on installed Git |
| Tower | System Git | Depends on installed Git |

Conclusion from market patterns: both approaches are valid; sequencing depends on product phase constraints.

---

## Future Bundling Path

The resolver is intentionally designed so bundled Git can be inserted at priority position 1 without changing command-call sites.

### Migration steps

1. Add bundled artifact to packaging.
2. Add bundled probe to resolver top slot.
3. Keep env override/system fallback unchanged.
4. Keep version validation and diagnostic shape unchanged.

Result: seamless transition from system-first to bundled-first resolution.

---

## Decision

**Adopt now:**

- **Foundation/MVP:** system Git via `GitResolver`, version-gated at `>= 2.35`.

**Adopt later (V1.0 pre-launch):**

- Bundle dugite-native Git in app.
- Promote bundled path to resolver priority position 1.

This is the chosen strategy for mongit’s write-path binary resolution.

---

## Immediate Implementation Notes

1. Introduce `src-tauri/src/git/resolver.rs`.
2. Replace direct `Command::new("git")` with resolver-provided binary path.
3. Add user-facing remediation errors for:
   - Git missing
   - Git version below floor
4. Add tests for resolver precedence and version gating.

