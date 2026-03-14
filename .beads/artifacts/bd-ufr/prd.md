# Git Engine Hybrid — Spike C

**Bead:** bd-ufr  
**Created:** 2026-03-14  
**Status:** Draft

## Bead Metadata

```yaml
depends_on: []
parallel: true
conflicts_with: ["bd-15p"] # Both may touch src-tauri/src/lib.rs command registration
blocks: []
estimated_hours: 8
requirements_score:
  total: 90
  breakdown:
    business_value: 27/30
    functional_requirements: 23/25
    user_experience: 17/20
    technical_constraints: 13/15
    scope_and_priorities: 10/10
  status: passed
  rounds_used: 3
  deferred_questions: 0
```

---

## Problem Statement

### What problem are we solving?

mongit needs a validated git abstraction layer before building any user-facing git features. The current codebase has a single `get_repo_status` command with inline `git2::Repository::open()` calls and string-based errors — no reusable module, no trait abstraction, no write-path capability, and no tests.

Without a proven hybrid architecture (git2 for reads, git CLI for writes), every downstream feature (staging, commit graph, branch management) will be built on unvalidated assumptions about threading, error handling, and API coverage.

### Why now?

This is Spike C in the Foundation phase (Weeks 1–4). The commit graph renderer (Spike B) and CodeMirror diff viewer (Spike D) both depend on validated git read operations. Blocking on this spike blocks the entire MVP.

### Who is affected?

- **Primary:** Developer building mongit — needs a reliable, tested git module to build features against
- **Secondary:** End users — all git operations flow through this layer; correctness here prevents data loss

---

## Scope

### In-Scope

- **Git module structure:** `src-tauri/src/git/` with `mod.rs`, `error.rs`, `repository.rs`, `cli.rs`
- **Trait abstraction:** `GitRepository` trait defining read and write operations
- **Read-path (git2):** Status, Diff, Log/Revwalk, Branch/Ref listing
- **Write-path (git CLI):** Branch create, Branch switch/checkout
- **Error handling:** Unified `GitError` enum using `thiserror`, wrapping `git2::Error` and CLI errors
- **Repository access pattern validation:** Open-per-call with `spawn_blocking`
- **Automated tests:** Using `tempfile` crate for temporary git repos
- **Tauri command migration:** Move `get_repo_status` to use the new module

### Out-of-Scope

- Blame operations (deferred to MVP file history feature)
- Commit/amend operations (requires hook executor — separate task)
- Push/fetch/pull operations (requires credential helper wiring — separate task)
- Stash management (V1.0 scope)
- Interactive rebase (V1.0 scope)
- GPG/SSH signing (requires bundled git binary strategy — separate task)
- File watcher integration (Spike D)
- Frontend changes (this spike is Rust-only)
- Performance benchmarking (validation only, not optimization)

---

## Proposed Solution

### Overview

Create a `src-tauri/src/git/` module with a `GitRepository` trait that abstracts read operations (via git2) and write operations (via git CLI subprocess). Each Tauri command opens `git2::Repository` fresh per-call inside `tokio::task::spawn_blocking`, avoiding the `!Sync` threading issue entirely. Write operations use `std::process::Command` wrapped in `spawn_blocking`. All errors funnel through a unified `GitError` enum.

### Architecture

```
Tauri IPC Command (async)
    │
    ├── Read path: spawn_blocking → git2::Repository::open() → git2 API → owned result types
    │
    └── Write path: spawn_blocking → std::process::Command("git") → parse stdout/stderr → result
```

### Module Layout

```
src-tauri/src/git/
├── mod.rs          # Public API: GitRepository trait + re-exports
├── error.rs        # GitError enum (thiserror)
├── repository.rs   # Git2Repository: read operations via git2
└── cli.rs          # GitCli: write operations via subprocess
```

---

## Requirements

### Functional Requirements

#### GitError Unified Error Type

A single error enum that wraps both git2 errors and CLI errors, convertible to Tauri's `String` return type.

**Scenarios:**

- **WHEN** a git2 operation fails **THEN** the error is wrapped as `GitError::Git2(git2::Error)` with the original error code preserved
- **WHEN** a git CLI command exits non-zero **THEN** the error is wrapped as `GitError::Cli { command, stderr, exit_code }` with full stderr captured
- **WHEN** a repository path doesn't exist **THEN** `GitError::NotFound { path }` is returned before attempting to open
- **WHEN** a GitError is converted to String for Tauri IPC **THEN** the message includes enough context to diagnose the issue

#### Read Operations (git2)

Status, diff, log, and branch listing implemented via `git2` crate with owned return types (no lifetime ties to Repository).

**Scenarios:**

- **WHEN** `status()` is called on a valid repo **THEN** it returns a list of `FileStatus { path, status_flags }` covering staged, unstaged, and untracked files
- **WHEN** `status()` is called with untracked file inclusion **THEN** untracked files and directories appear in results
- **WHEN** `diff_workdir()` is called **THEN** it returns structured `DiffResult` with hunks and lines for working directory changes vs HEAD
- **WHEN** `log()` is called with a limit **THEN** it returns up to N commits in topological order with oid, author, date, message, and parent oids
- **WHEN** `log()` is called on a repo with 10k+ commits **THEN** it completes without error (correctness, not performance target)
- **WHEN** `branches()` is called **THEN** it returns all local and remote branches with their target commit oid
- **WHEN** `current_branch()` is called **THEN** it returns the current HEAD branch name, or None if detached HEAD
- **WHEN** any read operation is called on a non-existent path **THEN** `GitError::NotFound` is returned

#### Write Operations (git CLI)

Branch creation and checkout via `std::process::Command` in `spawn_blocking`.

**Scenarios:**

- **WHEN** `create_branch(name, start_point)` is called **THEN** `git branch <name> <start_point>` executes and the new branch ref exists
- **WHEN** `switch_branch(name)` is called **THEN** `git switch <name>` executes and HEAD points to the target branch
- **WHEN** `switch_branch()` is called with a non-existent branch name **THEN** `GitError::Cli` is returned with the stderr from git
- **WHEN** `switch_branch()` would cause data loss (dirty working tree) **THEN** git's own safety check prevents the switch and the error is propagated

#### Repository Access Pattern

Open-per-call inside `spawn_blocking` — no shared Repository state.

**Scenarios:**

- **WHEN** a Tauri command needs git2 access **THEN** it clones the PathBuf, moves it into `spawn_blocking`, and opens `git2::Repository` fresh
- **WHEN** multiple Tauri commands run concurrently **THEN** each gets its own Repository instance with no lock contention
- **WHEN** the repo path in AppState changes (user opens different repo) **THEN** subsequent commands use the new path without stale state

#### Tauri Command Migration

Existing `get_repo_status` refactored to use the new git module.

**Scenarios:**

- **WHEN** `get_repo_status` is called **THEN** it delegates to `Git2Repository::status()` instead of inline git2 calls
- **WHEN** the refactored command is called from the frontend **THEN** the response shape (`RepoStatus`) remains unchanged (no frontend breaking changes)

### Non-Functional Requirements

- **Performance:** Repository open + status check < 50ms on a 1000-file repo (validated by tests existing, not benchmarked)
- **Thread safety:** No `!Sync` types cross async boundaries; all git2 access inside `spawn_blocking`
- **Error fidelity:** git2 error codes preserved through the error chain; CLI stderr always captured
- **Testability:** All git operations testable with temporary repos (no real repo dependency)

---

## Success Criteria

- [ ] `src-tauri/src/git/` module exists with `mod.rs`, `error.rs`, `repository.rs`, `cli.rs`
  - Verify: `ls src-tauri/src/git/`
- [ ] `GitError` enum compiles with `#[from] git2::Error`, `Cli`, `NotFound`, `Io` variants
  - Verify: `cargo check --manifest-path src-tauri/Cargo.toml`
- [ ] Read operations (status, diff, log, branches) return owned types — no lifetime ties to Repository
  - Verify: `cargo check --manifest-path src-tauri/Cargo.toml`
- [ ] Write operations (create_branch, switch_branch) execute git CLI via `std::process::Command`
  - Verify: `cargo test --manifest-path src-tauri/Cargo.toml -- git::cli`
- [ ] Open-per-call pattern validated: no `Arc<Mutex<Repository>>` in codebase
  - Verify: `grep -r "Arc<Mutex" src-tauri/src/ | grep -i repo` (expect zero results)
- [ ] Existing `get_repo_status` command uses new git module (no inline git2 calls)
  - Verify: `grep -c "git2::Repository::open" src-tauri/src/commands.rs` (expect zero)
- [ ] All tests pass with `tempfile`-based temporary repos
  - Verify: `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] `cargo check` and `cargo clippy` pass with zero errors
  - Verify: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- [ ] Frontend `pnpm check` still passes (no regressions)
  - Verify: `pnpm check`

---

## Technical Context

### Existing Patterns

- Per-call Repository open: `src-tauri/src/commands.rs:20-21` — current pattern to formalize
- Tauri command registration: `src-tauri/src/lib.rs:7-10` — `tauri::generate_handler![]`
- Shell plugin scope: `src-tauri/tauri.conf.json:48-55` — git command already allowed
- Serde response structs: `src-tauri/src/commands.rs:10-16` — `#[derive(Debug, Serialize)]`

### Key Constraints

- `git2::Repository` is `Send` but NOT `Sync` — cannot use `Arc<Repository>`; `Arc<Mutex<Repository>>` works but serializes all operations (anti-pattern)
- All git2 access must be inside `spawn_blocking` to avoid blocking the Tokio async executor
- Write operations must use git CLI (not git2) to get hooks, signing, and credential helpers
- `--porcelain=v2 -z` for any git status parsing (NUL-delimited, format-stable)
- `thiserror` 2 already in Cargo.toml dependencies

### Reference: GitButler Pattern

GitButler validates this architecture at scale:
- `OnDemand<git2::Repository>` — lazy per-thread cache via `RefCell`
- `ThreadSafeContext` drops git2 at thread boundaries, re-opens on new thread
- `git2-hooks` crate for in-process hook execution
- CLI fallback for all write operations

### Key Files

- `src-tauri/src/commands.rs` — Current IPC handlers to refactor
- `src-tauri/src/lib.rs` — Command registration to update
- `src-tauri/Cargo.toml` — Dependencies to add (`tempfile` dev-dep)
- `docs/research/2026-03-13-git-engine-architecture-recommendation.md` — Architecture blueprint

### Affected Files

Files this bead will modify (for conflict detection):

```yaml
files:
  - src-tauri/src/git/mod.rs        # NEW: Module root, trait definition, re-exports
  - src-tauri/src/git/error.rs      # NEW: GitError enum
  - src-tauri/src/git/repository.rs # NEW: Git2Repository read implementations
  - src-tauri/src/git/cli.rs        # NEW: GitCli write implementations
  - src-tauri/src/commands.rs       # MODIFY: Refactor to use git module
  - src-tauri/src/lib.rs            # MODIFY: Add mod git declaration
  - src-tauri/Cargo.toml            # MODIFY: Add tempfile dev-dependency
```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| git2 API doesn't cover all needed status flags | Low | Medium | Verify with tempfile test repos covering all flag combinations |
| CLI git version differences across macOS versions | Medium | Low | Use `--porcelain=v2 -z` (stable format); test with system git |
| `spawn_blocking` overhead for rapid operations | Low | Low | Open-per-call measured at ~0.5ms; acceptable for IPC commands |
| Owned return types require copying large diffs | Medium | Medium | Use String-based diff for now; `convertFileSrc` optimization deferred |

---

## Open Questions

| Question | Owner | Due Date | Status |
| --- | --- | --- | --- |
| (none — all questions resolved during requirements refinement) | — | — | — |

---

## Tasks

Write tasks in a machine-convertible format for `prd-task` skill.

**Rules:**

- Each task is a `### <Title> [category]` heading
- Provide one sentence describing the end state
- Include `**Metadata:**` with dependency info
- Include `**Verification:**` with bullet steps proving it works

### Define GitError enum [foundation]

`src-tauri/src/git/error.rs` contains a `GitError` enum with `Git2`, `Cli`, `NotFound`, and `Io` variants, all deriving `thiserror::Error`, with `From<git2::Error>` and `From<std::io::Error>` impls and a `From<GitError> for String` conversion for Tauri IPC.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src-tauri/src/git/error.rs
  - src-tauri/src/git/mod.rs
  - src-tauri/src/lib.rs
  - src-tauri/Cargo.toml
```

**Verification:**

- `cargo check --manifest-path src-tauri/Cargo.toml` passes
- `src-tauri/src/git/error.rs` exists with all 4 variants

### Implement git2 read operations [core]

`src-tauri/src/git/repository.rs` contains a `Git2Repository` struct with methods for `status()`, `diff_workdir()`, `log()`, `branches()`, and `current_branch()`, all returning owned types with no lifetime dependencies on `git2::Repository`.

**Metadata:**

```yaml
depends_on: ["Define GitError enum"]
parallel: false
conflicts_with: []
files:
  - src-tauri/src/git/repository.rs
  - src-tauri/src/git/mod.rs
```

**Verification:**

- `cargo check --manifest-path src-tauri/Cargo.toml` passes
- `cargo test --manifest-path src-tauri/Cargo.toml -- git::repository` passes (at least 5 tests)

### Implement git CLI write operations [core]

`src-tauri/src/git/cli.rs` contains a `GitCli` struct with methods for `create_branch()` and `switch_branch()`, executing git commands via `std::process::Command` with full stderr capture and structured error reporting.

**Metadata:**

```yaml
depends_on: ["Define GitError enum"]
parallel: true
conflicts_with: []
files:
  - src-tauri/src/git/cli.rs
  - src-tauri/src/git/mod.rs
```

**Verification:**

- `cargo check --manifest-path src-tauri/Cargo.toml` passes
- `cargo test --manifest-path src-tauri/Cargo.toml -- git::cli` passes (at least 3 tests)

### Add tempfile dev-dependency and test infrastructure [testing]

`src-tauri/Cargo.toml` has `tempfile` in `[dev-dependencies]`, and a test helper function exists that creates a temporary git repository with initial commit for use across all git module tests.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src-tauri/Cargo.toml
  - src-tauri/src/git/mod.rs
```

**Verification:**

- `cargo test --manifest-path src-tauri/Cargo.toml -- git::tests::test_helper` passes
- `grep "tempfile" src-tauri/Cargo.toml` shows entry in `[dev-dependencies]`

### Refactor get_repo_status to use git module [integration]

`src-tauri/src/commands.rs` delegates to `Git2Repository::status()` and `Git2Repository::current_branch()` instead of inline git2 calls, with `spawn_blocking` wrapping the blocking git2 work.

**Metadata:**

```yaml
depends_on: ["Implement git2 read operations"]
parallel: false
conflicts_with: []
files:
  - src-tauri/src/commands.rs
```

**Verification:**

- `cargo check --manifest-path src-tauri/Cargo.toml` passes
- `grep -c "git2::Repository::open" src-tauri/src/commands.rs` returns 0
- `cargo test --manifest-path src-tauri/Cargo.toml` — all tests pass
- `pnpm check` passes (no frontend regressions)

### Validate open-per-call pattern [validation]

No `Arc<Mutex<Repository>>` exists in the codebase; all git2 access is inside `spawn_blocking` closures; concurrent command execution works without lock contention.

**Metadata:**

```yaml
depends_on: ["Refactor get_repo_status to use git module"]
parallel: false
conflicts_with: []
files: []
```

**Verification:**

- `grep -r "Arc<Mutex" src-tauri/src/ | grep -ci repo` returns 0
- `grep -r "spawn_blocking" src-tauri/src/commands.rs` returns at least 1 match
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` passes

---

## Dependency Legend

| Field | Purpose | Example |
| --- | --- | --- |
| `depends_on` | Must complete before this task starts | `["Setup database", "Create schema"]` |
| `parallel` | Can run concurrently with other parallel tasks | `true` / `false` |
| `conflicts_with` | Cannot run in parallel (same files) | `["Update config"]` |
| `files` | Files this task modifies (for conflict detection) | `["src/db/schema.ts", "src/db/client.ts"]` |

---

## Notes

- **GitButler reference:** Their `OnDemand<git2::Repository>` pattern confirms open-per-call is production-viable. For mongit's MVP, simple open-per-call in `spawn_blocking` is sufficient; caching can be added later if profiling shows need.
- **Future extension points:** This module's trait abstraction (`Git2Repository` / `GitCli`) is designed so that commit, push, fetch, stash, and blame operations can be added incrementally without restructuring.
- **Bundled git binary decision deferred:** This spike validates the CLI write-path using system git. The decision to bundle a git binary (like GitHub Desktop's dugite) is a separate task for the MVP phase.
- **Hook execution deferred:** The `git2-hooks` crate (used by GitButler) will be evaluated when implementing commit operations. Branch switch doesn't require hooks.
