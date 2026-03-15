# Bundled Git Binary Strategy Spike

**Bead:** bd-2na
**Created:** 2026-03-14
**Status:** Draft

## Bead Metadata

```yaml
depends_on: [] # bd-ufr (Git engine hybrid) is a predecessor but already closed
parallel: true # Can run concurrently with other foundation spikes
conflicts_with: [] # No other beads modifying same files
blocks: [] # Future MVP write-path beads will depend on this decision
estimated_hours: 4
requirements_score:
  total: 91
  breakdown:
    business_value: 28/30
    functional_requirements: 22/25
    user_experience: 17/20
    technical_constraints: 14/15
    scope_and_priorities: 10/10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

### What problem are we solving?

mongit's git architecture recommends git2 for reads and a CLI git binary for writes, but the current write-path code in `GitCli::run_git()` (`src-tauri/src/git/cli.rs:57`) uses `Command::new("git")`, which resolves to whatever git the user has on their `$PATH`. This is acceptable for a spike but not stable enough for MVP because:

- Different machines have different Git versions (macOS ships ancient 2.39 via Xcode CLT)
- Feature support varies across versions (e.g., `--porcelain=v2`, `switch` command)
- Hook behavior, credential helpers, and signing support are version-dependent
- No version validation exists — silent failures on old git are possible

### Why now?

The hybrid engine spike (bd-ufr) validated git2 reads and established the `GitCli` shell-out pattern for writes. Before building MVP write operations (commit, push, merge, rebase), we need a deterministic git resolution strategy. Building on an unvalidated foundation risks rework across every write command.

### Who is affected?

- **Primary:** mongit developers — need a clear, tested pattern for invoking git in all future write commands
- **Secondary:** End users — need consistent behavior regardless of their system git version

---

## Scope

### In-Scope

- Choose and validate a git binary resolution strategy for macOS
- Implement a custom `GitResolver` module that deterministically resolves the git executable path
- Enforce minimum git version check (>= 2.35)
- Define fallback policy: error dialog with install instructions when git is missing or too old
- Prove the resolver works by executing at least one write command through it
- Document the strategy, binary size implications, and packaging tradeoffs
- Add `GIT_EXECUTABLE` environment variable override for development/CI

### Out-of-Scope

- Full push/fetch/pull implementation (future MVP beads)
- GPG/SSH signing support implementation
- Credential helper configuration
- Windows/Linux packaging or binary sourcing
- Bundling a git binary inside the .app bundle (deferred to V1.0 pre-launch)
- Replacing all existing `GitCli` write commands immediately
- UI for the error dialog (spike validates backend resolution only)

---

## Proposed Solution

### Overview

Implement a `GitResolver` module in Rust that resolves the git binary path through a deterministic priority chain: (1) `GIT_EXECUTABLE` env var override, (2) system git via PATH lookup, (3) error if not found. The resolver validates the git version is >= 2.35 and returns a structured result with the path and version. The existing `GitCli` struct is refactored to accept a resolved path instead of hardcoding `"git"`. A single write command (e.g., `git init` or `git branch`) validates end-to-end resolution.

### Resolution Priority Chain

```
1. GIT_EXECUTABLE env var    → Developer/CI override
2. System git via PATH       → which::which("git") or manual PATH search
3. Error                     → "Git >= 2.35 required. Install via: brew install git"
```

### Architecture

```
src-tauri/src/git/
├── mod.rs           # Add pub use resolver::GitResolver
├── resolver.rs      # NEW: GitResolver struct + resolve() + version_check()
├── cli.rs           # Refactor: accept resolved path, remove hardcoded "git"
├── repository.rs    # Unchanged (libgit2 reads)
└── error.rs         # Add GitNotFound, GitVersionTooOld variants
```

---

## Requirements

### Functional Requirements

#### FR-1: Git Binary Resolution

The `GitResolver` module deterministically resolves the absolute path to a git executable.

**Scenarios:**

- **WHEN** `GIT_EXECUTABLE` env var is set and points to a valid git binary **THEN** use that path without PATH lookup
- **WHEN** `GIT_EXECUTABLE` is not set and git exists in system PATH **THEN** resolve via `which::which("git")` and return the absolute path
- **WHEN** neither `GIT_EXECUTABLE` nor system git is available **THEN** return `GitError::GitNotFound` with install instructions
- **WHEN** the resolved binary is not executable or returns invalid version output **THEN** return `GitError::GitNotFound`

#### FR-2: Version Validation

The resolver validates the git version is >= 2.35.

**Scenarios:**

- **WHEN** resolved git reports version >= 2.35 **THEN** resolution succeeds with path + parsed version
- **WHEN** resolved git reports version < 2.35 **THEN** return `GitError::GitVersionTooOld { found: "2.30.0", minimum: "2.35.0" }`
- **WHEN** version output cannot be parsed **THEN** return `GitError::GitVersionTooOld` with raw output in the error message

#### FR-3: GitCli Integration

The `GitCli` struct uses the resolved path instead of hardcoded `"git"`.

**Scenarios:**

- **WHEN** `GitCli::new()` is called with a resolved path **THEN** all `run_git()` invocations use that path
- **WHEN** a write command executes successfully **THEN** the resolved path was used (not system `"git"` fallback)

#### FR-4: Environment Variable Override

Developers and CI can force a specific git binary.

**Scenarios:**

- **WHEN** `GIT_EXECUTABLE=/usr/local/bin/git` is set **THEN** that exact path is used, bypassing PATH lookup
- **WHEN** `GIT_EXECUTABLE` points to a non-existent file **THEN** return `GitError::GitNotFound` (do not fall through to PATH)

### Non-Functional Requirements

- **Performance:** Resolution should complete in < 50ms (it's a PATH lookup + one `git --version` invocation)
- **Testability:** `GitResolver` must be unit-testable without requiring a git installation (via trait or mock injection)
- **Determinism:** Same inputs always produce same resolution result — no caching side effects across calls
- **Logging:** Resolution result (path, version, source) logged at INFO level for diagnostics

---

## Success Criteria

- [ ] `GitResolver::resolve()` returns a valid git path on a standard macOS dev machine
  - Verify: `cargo test -p mongit --lib -- git::resolver`
- [ ] Version check rejects git < 2.35 with a clear error message
  - Verify: Unit test with mock version output "git version 2.30.0" returns `GitVersionTooOld`
- [ ] `GIT_EXECUTABLE` env var override works and takes priority over PATH
  - Verify: `GIT_EXECUTABLE=/usr/bin/git cargo test -p mongit --lib -- git::resolver::env_override`
- [ ] `GitCli::run_git()` uses the resolved path (not hardcoded `"git"`)
  - Verify: `cargo test -p mongit --lib -- git::cli`
- [ ] At least one write command executes successfully using the resolved path
  - Verify: Integration test creates a temp repo, resolves git, creates a branch via `GitCli`
- [ ] `cargo check` passes with zero warnings in `src-tauri/`
  - Verify: `cd src-tauri && cargo check 2>&1 | grep -c warning`
- [ ] Strategy document exists explaining the decision and future bundling path
  - Verify: `test -f docs/research/2026-03-bundled-git-strategy.md`

---

## Technical Context

### Existing Patterns

- **`src-tauri/src/git/cli.rs:55-72`** — Current `run_git()` with `Command::new("git")`. This is the exact code that needs the resolved path injected.
- **`src-tauri/src/git/repository.rs`** — `Git2Repository` uses open-per-call pattern (no shared state). Same stateless pattern should apply to `GitResolver`.
- **`src-tauri/src/git/error.rs`** — Unified `GitError` enum bridges git2 and CLI errors. New variants (`GitNotFound`, `GitVersionTooOld`) follow the same pattern.
- **`src-tauri/src/lib.rs:11`** — `tauri_plugin_shell::init()` already registered. Resolver can use `std::process::Command` directly (no Tauri dependency needed for system git).
- **`src-tauri/Cargo.toml`** — `tauri-plugin-shell = "2"` present. May need `which` crate for PATH lookup.

### Research Findings

- **dugite-native** (GitHub Desktop's git binaries): arm64 macOS ~60MB with GCM, ~15-20MB without. Available for future V1.0 bundling phase.
- **GitButler** uses system git only via `gix::path::env::exe_invocation()` — validates the system-git-first approach for power-developer tools.
- **Tauri 2.0 sidecar pattern**: Requires target-triple naming (`git-aarch64-apple-darwin`) and capability config. Reserved for future bundled binary phase.
- **macOS Xcode CLT git**: Ships >= 2.39 since macOS Ventura (2022). The >= 2.35 floor is conservative and safe.
- Source: `docs/research/2026-03-13-git-engine-architecture-recommendation.md` (hybrid architecture decision)

### Key Files

- `src-tauri/src/git/cli.rs` — Write-path CLI wrapper, target for path injection
- `src-tauri/src/git/error.rs` — Error types, needs new variants
- `src-tauri/src/git/mod.rs` — Module exports, needs resolver re-export
- `src-tauri/Cargo.toml` — May need `which` crate dependency

### Affected Files

Files this bead will modify:

```yaml
files:
  - src-tauri/src/git/resolver.rs # NEW: GitResolver module
  - src-tauri/src/git/cli.rs # Refactor to accept resolved path
  - src-tauri/src/git/error.rs # Add GitNotFound, GitVersionTooOld variants
  - src-tauri/src/git/mod.rs # Add resolver module export
  - src-tauri/Cargo.toml # Add which crate (if used)
  - docs/research/2026-03-bundled-git-strategy.md # NEW: Strategy document
```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| macOS user has no git installed at all | Low | Medium | Clear error message with `brew install git` instructions. Power developers nearly always have git. |
| `which::which()` resolves a wrapper script (e.g., Homebrew shim) not actual git | Low | Low | Version check validates the resolved binary produces valid `git --version` output |
| Future bundled binary phase adds complexity to resolver | Medium | Low | Resolver is designed with priority chain — bundled path slots in at position 1, system git drops to position 2 |
| `GIT_EXECUTABLE` pointed at wrong binary in CI | Low | Low | Version check still runs on env-var-specified path; fails fast with clear error |
| Git version parsing fails on non-standard git builds | Low | Low | Regex-based parsing with fallback error showing raw version string |

---

## Tasks

### Add GitNotFound and GitVersionTooOld error variants [backend]

`GitError` enum in `error.rs` includes `GitNotFound { message: String }` and `GitVersionTooOld { found: String, minimum: String }` variants with descriptive Display implementations.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src-tauri/src/git/error.rs
```

**Verification:**

- `cd src-tauri && cargo check`

### Implement GitResolver module [backend]

`GitResolver::resolve()` returns a `ResolvedGit { path: PathBuf, version: GitVersion }` by checking `GIT_EXECUTABLE` env var first, then system PATH via `which::which("git")`, then returning `GitNotFound`. Version is parsed from `git --version` output and validated >= 2.35.

**Metadata:**

```yaml
depends_on: ["Add GitNotFound and GitVersionTooOld error variants"]
parallel: false
conflicts_with: []
files:
  - src-tauri/src/git/resolver.rs
  - src-tauri/src/git/mod.rs
  - src-tauri/Cargo.toml
```

**Verification:**

- `cd src-tauri && cargo test --lib -- git::resolver`
- `cd src-tauri && cargo check`

### Refactor GitCli to accept resolved git path [backend]

`GitCli::new()` accepts a `PathBuf` for the git executable. `run_git()` uses `Command::new(&self.git_executable)` instead of `Command::new("git")`. Existing tests updated.

**Metadata:**

```yaml
depends_on: ["Implement GitResolver module"]
parallel: false
conflicts_with: []
files:
  - src-tauri/src/git/cli.rs
```

**Verification:**

- `cd src-tauri && cargo test --lib -- git::cli`
- `cd src-tauri && cargo check`

### Integration test: resolve + write command [integration]

An integration test creates a temp git repo, calls `GitResolver::resolve()`, constructs `GitCli` with the resolved path, and executes `create_branch()`. Asserts the branch exists via `Git2Repository`.

**Metadata:**

```yaml
depends_on: ["Refactor GitCli to accept resolved git path"]
parallel: false
conflicts_with: []
files:
  - src-tauri/src/git/mod.rs
```

**Verification:**

- `cd src-tauri && cargo test --lib -- git::integration`

### Document bundled git strategy [documentation]

A research document at `docs/research/2026-03-bundled-git-strategy.md` explains the phase-gated approach: system git with version gate for Foundation/MVP, bundled dugite-native binary for V1.0 pre-launch. Includes binary size analysis, packaging implications, code-signing notes, and fallback policy.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - docs/research/2026-03-bundled-git-strategy.md
```

**Verification:**

- `test -f docs/research/2026-03-bundled-git-strategy.md`
- Document includes: strategy rationale, binary size table, packaging implications, future bundling path

---

## Notes

- **Phase-gated approach:** System git for Foundation+MVP (zero bundle complexity, < 25MB target). Bundled dugite-native binary deferred to V1.0 pre-launch when binary size budget allows.
- **GitButler precedent:** They ship with system git only for the same target audience (power developers). Validates this approach.
- **Future extensibility:** The `GitResolver` priority chain is designed so a bundled binary path slots in at position 1 when that phase arrives, with system git dropping to position 2 as fallback.
- **`which` crate:** Preferred over manual PATH traversal. Cross-platform, well-tested, 0 transitive deps. Add to `[dependencies]` in Cargo.toml.
