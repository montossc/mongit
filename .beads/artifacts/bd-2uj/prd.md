# MVP Branch Operations on Hybrid Git Engine

**Bead:** bd-2uj
**Created:** 2026-03-15
**Status:** Draft

## Bead Metadata

```yaml
depends_on: [bd-2na] # GitResolver spike must land first — all CLI ops use resolved path
parallel: false # Cannot run until bd-2na completes
conflicts_with: [] # No file overlap with other open beads
blocks: [] # Future MVP UI beads for branch management
estimated_hours: 8
requirements_score:
  total: 91
  breakdown:
    business_value: 28/30
    functional_requirements: 23/25
    user_experience: 16/20
    technical_constraints: 14/15
    scope_and_priorities: 10/10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

### What problem are we solving?

mongit currently has only spike-level branch operations: `create_branch` and `switch_branch` in `GitCli` (src-tauri/src/git/cli.rs:32-49). MVP requires a complete branch workflow — delete, fetch, pull, push — so users can work on real repositories without dropping back to terminal for basic operations.

The existing error handling passes raw `CommandFailed` stderr to the frontend, which is insufficient for a polished GUI. Users need structured, actionable error messages (e.g., "Branch has unmerged commits" vs a raw git stderr dump).

### Why now?

After the GitResolver spike (bd-2na) validates deterministic git path resolution, this is the next natural slice — production-grade branch operations are a prerequisite for MVP's "Local changes workspace" and "Commit authoring" features. Without these, the app cannot support basic daily workflows.

### Who is affected?

- **Primary users:** Solo power developers using mongit as their daily Git client
- **Secondary users:** Frontend layer (Svelte 5) consuming Tauri IPC commands — needs clean typed error contracts

---

## Scope

### In-Scope

- Branch create (via CLI, using resolved git path from GitResolver)
- Branch switch (via CLI)
- Branch delete — safe (`-d`) and force (`-D`) modes with pre-flight checks
- Fetch from origin (single remote for MVP)
- Pull from origin (with current branch tracking)
- Push to origin (normal + `--force-with-lease`)
- Structured error enum (`BranchOpError`) with regex-based stderr parsing
- Tauri async command handlers for all operations
- Tests for happy path + common failure modes
- Error messages with actionable context for GUI consumption

### Out-of-Scope

- Interactive rebase, cherry-pick, merge conflict resolution flow
- Credential UI (auth failures surface as structured errors only)
- Signing UI
- Multi-remote support (deferred to V1.0)
- Bare `--force` push (only `--force-with-lease`)
- Advanced remote configuration
- Branch rename (deferred — not in MVP requirements)
- Remote branch delete (deferred — higher risk, needs more safety checks)

---

## Proposed Solution

### Overview

Extend the hybrid git engine with production-grade branch operations. Local-only operations (delete) use pre-flight safety checks via git2 (is_head, merge status) before CLI execution. Remote operations (fetch, pull, push) use `tokio::process::Command` for async non-blocking execution with the system git binary resolved by GitResolver. All errors flow through a structured `BranchOpError` enum that maps git stderr patterns to actionable frontend messages, following the GitHub Desktop dugite pattern.

### Architecture

```
Frontend (invoke)
    │
    ▼
commands.rs  (#[tauri::command] async handlers)
    │
    ├── git2 pre-flight checks (is_head, merge status)
    │
    ▼
git/branch.rs  (new: branch operation logic)
    │
    ├── Local ops: GitCli::run_git() via spawn_blocking
    │
    ├── Remote ops: tokio::process::Command (async, non-blocking)
    │
    ▼
git/error.rs  (extended: BranchOpError with stderr parsing)
    │
    ▼
Frontend  (typed discriminated union errors)
```

---

## Requirements

### Functional Requirements

#### FR-1: Branch Create

Create a new local branch from an optional start point.

**Scenarios:**

- **WHEN** user creates branch "feature-x" with no start point **THEN** branch is created at HEAD and CLI returns success
- **WHEN** user creates branch with a name that already exists **THEN** return `BranchAlreadyExists` error with the branch name
- **WHEN** user creates branch with invalid characters **THEN** return `InvalidBranchName` error

#### FR-2: Branch Switch

Switch the working tree to a different branch.

**Scenarios:**

- **WHEN** user switches to existing branch "main" **THEN** HEAD points to "main" and working tree is updated
- **WHEN** user switches with dirty working tree that conflicts **THEN** return `DirtyWorkingTree` error listing affected files
- **WHEN** target branch does not exist **THEN** return `BranchNotFound` error

#### FR-3: Branch Delete (Safe + Force)

Delete a local branch with configurable safety.

**Scenarios:**

- **WHEN** user deletes a fully-merged branch with `force=false` **THEN** branch is deleted via `git branch -d`
- **WHEN** user deletes an unmerged branch with `force=false` **THEN** return `BranchNotFullyMerged` error with unmerged status
- **WHEN** user deletes an unmerged branch with `force=true` **THEN** branch is deleted via `git branch -D`
- **WHEN** user attempts to delete the current HEAD branch **THEN** return `DeleteCurrentBranch` error (pre-flight check via git2 `branch.is_head()`)
- **WHEN** branch does not exist **THEN** return `BranchNotFound` error

#### FR-4: Fetch

Fetch latest refs and objects from origin remote.

**Scenarios:**

- **WHEN** user triggers fetch **THEN** `git fetch --prune origin` executes asynchronously and returns updated ref list
- **WHEN** remote is unreachable **THEN** return `NetworkError` with host/connection details
- **WHEN** authentication fails **THEN** return `AuthFailure` error with message suggesting credential configuration
- **WHEN** remote "origin" does not exist **THEN** return `RemoteNotFound` error

#### FR-5: Pull

Pull changes from origin into the current branch.

**Scenarios:**

- **WHEN** user pulls with clean working tree and no conflicts **THEN** `git pull origin <current-branch>` succeeds
- **WHEN** pull encounters merge conflicts **THEN** return `MergeConflicts` error with conflicted file count
- **WHEN** working tree has uncommitted changes that would be overwritten **THEN** return `DirtyWorkingTree` error
- **WHEN** branches have diverged **THEN** return `BranchesDiverged` error with suggestion to configure pull strategy
- **WHEN** current branch has no upstream tracking **THEN** return `NoUpstreamBranch` error

#### FR-6: Push

Push current branch to origin.

**Scenarios:**

- **WHEN** user pushes with `force=false` **THEN** `git push origin <current-branch>` executes
- **WHEN** user pushes with `force=true` **THEN** `git push --force-with-lease origin <current-branch>` executes (never bare `--force`)
- **WHEN** push is rejected as non-fast-forward **THEN** return `PushNonFastForward` error suggesting pull first
- **WHEN** authentication fails **THEN** return `AuthFailure` error
- **WHEN** remote branch is protected **THEN** return `ProtectedBranch` error with remote message
- **WHEN** current branch has no upstream **THEN** auto-set upstream: `git push -u origin <branch>`

### Non-Functional Requirements

- **Performance:** Branch operations should complete within git's native timing. No additional overhead beyond Tauri IPC (~1-2ms). Long remote operations (fetch/push) must not block the UI thread — use `tokio::process::Command` (async).
- **Security:** Credentials are never stored or logged by mongit. Auth failures delegate to system git's `credential.helper` chain. `GIT_TERMINAL_PROMPT=0` must be set to prevent terminal prompts from hanging the process.
- **Error fidelity:** All errors must serialize to a discriminated union (`{ kind: string, message: string, ...details }`) for typed frontend consumption. Raw stderr is preserved in a `raw` field for debugging.
- **Cancellation:** Remote operations (fetch/pull/push) should support cancellation via child process kill. Not required for MVP but architecture should not preclude it.

---

## Success Criteria

- [ ] Branch create/switch/delete commands compile and pass tests
  - Verify: `cd src-tauri && cargo test branch`
- [ ] Delete pre-flight checks prevent deleting HEAD branch and warn on unmerged
  - Verify: `cd src-tauri && cargo test test_delete_current_branch_fails && cargo test test_delete_unmerged_branch_safe`
- [ ] Fetch/pull/push command surface exists with structured error mapping
  - Verify: `cd src-tauri && cargo test test_fetch && cargo test test_push`
- [ ] All errors map to typed `BranchOpError` variants (no raw strings to frontend)
  - Verify: `cd src-tauri && cargo test test_error_parsing`
- [ ] Backend passes full verification suite
  - Verify: `cd src-tauri && cargo check && cargo test --lib && cargo clippy -- -D warnings`
- [ ] Frontend remains unaffected
  - Verify: `pnpm check`
- [ ] Tauri commands registered and callable from frontend via `invoke()`
  - Verify: `cd src-tauri && cargo check` (command registration is compile-time verified)

---

## Technical Context

### Existing Patterns

- **CLI write path:** `src-tauri/src/git/cli.rs:55-72` — `GitCli::run_git()` uses `Command::new("git").arg("-C").arg(path)` pattern. After bd-2na, this will accept a resolved `PathBuf` instead of hardcoded `"git"`.
- **Async blocking:** `src-tauri/src/commands.rs:23-39` — `tokio::task::spawn_blocking` wraps git2 reads. Same pattern applies for local branch operations.
- **Error propagation:** `src-tauri/src/git/error.rs:5-31` — `GitError` enum with `CommandFailed { cmd, stderr, exit_code }`. New `BranchOpError` should parse stderr into specific variants.
- **Test fixtures:** `src-tauri/src/git/mod.rs:12-53` — `create_test_repo()` creates temp dir + git2 repo with initial commit. Branch tests extend this pattern.

### Key Files

- `src-tauri/src/git/cli.rs` — GitCli struct, `run_git()` method, existing `create_branch`/`switch_branch`
- `src-tauri/src/git/error.rs` — GitError enum with thiserror
- `src-tauri/src/git/repository.rs` — Git2Repository with `branches()`, `current_branch()`, `is_head()` for pre-flight checks
- `src-tauri/src/git/mod.rs` — Module exports and test helpers
- `src-tauri/src/commands.rs` — Tauri command handlers (4 existing)
- `src-tauri/src/lib.rs` — Command registration in `generate_handler![]`

### Affected Files

Files this bead will modify:

```yaml
files:
  - src-tauri/src/git/branch.rs # NEW: Branch operation logic (create, delete, fetch, pull, push)
  - src-tauri/src/git/error.rs # Extend with BranchOpError enum and stderr parsing
  - src-tauri/src/git/cli.rs # Refactor: extract common run_git, add async run_git_async
  - src-tauri/src/git/mod.rs # Export branch module, extend test helpers
  - src-tauri/src/commands.rs # Add 6 new Tauri command handlers
  - src-tauri/src/lib.rs # Register new commands in generate_handler![]
```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| ---- | ---------- | ------ | ---------- |
| Remote operations hang without `GIT_TERMINAL_PROMPT=0` | High | High | Always set env var on CLI subprocess. Add timeout (30s) for remote ops. |
| Stderr parsing misses edge cases | Medium | Medium | Start with top-10 error patterns from GitHub Desktop. Fall back to `GenericCommandFailed` with raw stderr. Expand patterns over time. |
| Tests require network for remote ops | Medium | Low | Mock remote ops in unit tests. Integration tests with local bare repo for fetch/push. |
| bd-2na (GitResolver) scope creeps | Low | Medium | Hard dependency — if bd-2na is delayed, this bead waits. No workarounds. |

---

## Open Questions

_None — all questions resolved in requirements refinement._

---

## Tasks

### 1. Extend error types with BranchOpError and stderr parser [backend]

`src-tauri/src/git/error.rs` contains a `BranchOpError` enum with variants for all branch operation failures, plus a `parse_branch_stderr()` function that regex-matches git CLI stderr into typed errors.

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
- `cd src-tauri && cargo test test_error_parsing`

### 2. Implement branch delete with pre-flight safety [backend]

`src-tauri/src/git/branch.rs` provides `delete_branch(path, name, force)` that checks `is_head()` via git2 before invoking `git branch -d/-D` via CLI. Returns typed `BranchOpError` on failure.

**Metadata:**

```yaml
depends_on: ["Extend error types with BranchOpError and stderr parser"]
parallel: false
conflicts_with: []
files:
  - src-tauri/src/git/branch.rs
  - src-tauri/src/git/mod.rs
```

**Verification:**

- `cd src-tauri && cargo test test_delete_branch`
- `cd src-tauri && cargo test test_delete_current_branch_fails`
- `cd src-tauri && cargo test test_delete_unmerged_branch_safe`
- `cd src-tauri && cargo test test_force_delete_unmerged_branch`

### 3. Implement fetch, pull, push wrappers [backend]

`src-tauri/src/git/branch.rs` provides `fetch_origin()`, `pull_origin()`, and `push_origin(force_with_lease)` using `tokio::process::Command` for async non-blocking execution. `GIT_TERMINAL_PROMPT=0` is set on all remote operations. Errors are parsed through `parse_branch_stderr()`.

**Metadata:**

```yaml
depends_on: ["Extend error types with BranchOpError and stderr parser"]
parallel: true
conflicts_with: ["Implement branch delete with pre-flight safety"]
files:
  - src-tauri/src/git/branch.rs
  - src-tauri/src/git/cli.rs
```

**Verification:**

- `cd src-tauri && cargo check`
- `cd src-tauri && cargo test test_fetch`
- `cd src-tauri && cargo test test_push`
- `cd src-tauri && cargo test test_pull`

### 4. Add Tauri command handlers and registration [integration]

`src-tauri/src/commands.rs` exposes `create_branch`, `switch_branch`, `delete_branch`, `fetch`, `pull`, `push` as `#[tauri::command]` async functions. All registered in `lib.rs` `generate_handler![]`. Local ops use `spawn_blocking`, remote ops use async `tokio::process`.

**Metadata:**

```yaml
depends_on: ["Implement branch delete with pre-flight safety", "Implement fetch, pull, push wrappers"]
parallel: false
conflicts_with: []
files:
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
```

**Verification:**

- `cd src-tauri && cargo check`
- `cd src-tauri && cargo test --lib`

### 5. Add integration tests for failure modes [test]

Test suite covers: delete HEAD branch (blocked), delete unmerged branch safe vs force, push non-fast-forward error, fetch with missing remote, pull with dirty working tree. Uses `create_test_repo()` fixture with local bare remote for push/fetch tests.

**Metadata:**

```yaml
depends_on: ["Add Tauri command handlers and registration"]
parallel: false
conflicts_with: []
files:
  - src-tauri/src/git/mod.rs
  - src-tauri/src/git/branch.rs
```

**Verification:**

- `cd src-tauri && cargo test branch -- --nocapture`
- `cd src-tauri && cargo clippy -- -D warnings`

### 6. Full verification pass [verification]

All backend checks pass, frontend unaffected, no clippy warnings.

**Metadata:**

```yaml
depends_on: ["Add integration tests for failure modes"]
parallel: false
conflicts_with: []
files: []
```

**Verification:**

- `cd src-tauri && cargo check && cargo test --lib && cargo clippy -- -D warnings`
- `pnpm check`

---

## Dependency Legend

| Field            | Purpose                                           | Example                               |
| ---------------- | ------------------------------------------------- | ------------------------------------- |
| `depends_on`     | Must complete before this task starts             | `["Setup database"]`                  |
| `parallel`       | Can run concurrently with other parallel tasks    | `true` / `false`                      |
| `conflicts_with` | Cannot run in parallel (same files)               | `["Update config"]`                   |
| `files`          | Files this task modifies (for conflict detection) | `["src-tauri/src/git/error.rs"]`      |

---

## Notes

- **Credential delegation:** MVP deliberately avoids custom credential UI. System git's `credential.helper` chain handles auth. The `GIT_TERMINAL_PROMPT=0` env var prevents terminal prompts from hanging the subprocess.
- **Origin-only:** Multi-remote support deferred to V1.0. All remote operations hardcode "origin" as the remote name.
- **Force-with-lease only:** Bare `--force` push is never exposed. `--force-with-lease` is the only force option, matching GitHub Desktop's safety-first pattern.
- **GitHub Desktop error patterns:** The stderr regex patterns in `parse_branch_stderr()` are derived from GitHub Desktop's `dugite` error mapping (40+ patterns). MVP starts with the top ~15 most common patterns and falls back to `GenericCommandFailed` for unmatched stderr.
- **Async model:** Local operations (create, switch, delete) use `spawn_blocking` since they complete in <100ms. Remote operations (fetch, pull, push) use `tokio::process::Command` for true async — the thread is released during I/O wait.
