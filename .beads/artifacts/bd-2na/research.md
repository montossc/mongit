# Research: bd-2na — Bundled Git Binary Strategy Spike

**Date:** 2026-03-15
**Depth:** Default (~30 tool calls)
**Confidence:** High

---

## Questions Asked → Answered

| # | Question | Status | Confidence |
|---|---------|--------|------------|
| 1 | What does the current `GitCli` look like — what needs changing? | ✅ Answered | High |
| 2 | What is the `which` crate API (version, usage)? | ✅ Answered | High |
| 3 | What error variants are already in `GitError`? | ✅ Answered | High |
| 4 | What is the standard pattern for parsing `git --version` output in Rust? | ✅ Answered | High |
| 5 | Which files does `mod.rs` currently export? | ✅ Answered | High |
| 6 | Are there existing tests for `GitCli` that need updating? | ✅ Answered | High |

---

## Key Findings

### 1. Current `GitCli` — Exact Lines That Need Changing

**File:** `src-tauri/src/git/cli.rs`

```rust
// Line 57 — hardcoded "git" is the target for path injection:
let output = Command::new("git")   // ← replace with &self.git_executable
    .arg("-C")
    ...
```

`GitCli::new()` only accepts `path: impl Into<PathBuf>`. It needs a second field:

```rust
pub struct GitCli {
    path: PathBuf,
    git_executable: PathBuf,  // NEW
}
```

Constructor signature becomes:

```rust
pub fn new(path: impl Into<PathBuf>, git_executable: PathBuf) -> Self
```

**Existing tests** in `mod.rs` that call `GitCli::new(repo_path)` will need updating:
- `test_create_branch` — uses `GitCli::new(repo_path)`
- `test_switch_branch` — uses `GitCli::new(repo_path)`
- `test_create_duplicate_branch_fails` — uses `GitCli::new(repo_path)`
- `test_log_all_branches` — uses `GitCli::new(path)`

These tests must resolve git via `GitResolver::resolve()` or pass `PathBuf::from("git")` as a simple default for test setup.

---

### 2. `which` Crate — API Confirmed

**Version:** 8.0.2 (latest)  
**Cargo.toml entry:** `which = "8"`  
**Zero transitive deps** on macOS  

```rust
use which::which;
use std::path::PathBuf;

let result: Result<PathBuf, which::Error> = which("git");
// Returns absolute PathBuf to first "git" in $PATH

// Preferred variant — ignores cwd, security-safe:
let result = which::which_global("git");
```

**Source:** https://docs.rs/which/latest/which/

**Design decision:** Use `which::which_global("git")` (not `which("git")`) to avoid cwd-relative resolution edge cases. `which_global` searches only `$PATH`, not the current directory.

---

### 3. Existing `GitError` Variants — What's There / What's Missing

**File:** `src-tauri/src/git/error.rs`

Currently defined:
- `Git2(#[from] git2::Error)` — libgit2 errors
- `CommandFailed { cmd, stderr, exit_code }` — CLI shell-out errors
- `NotFound(String)` — repo/ref not found
- `InvalidArgument(String)` — bad arg
- `Io(#[from] std::io::Error)` — filesystem I/O

**Missing (need adding):**
```rust
/// Git binary not found on system PATH
#[error("git not found: {message}")]
GitNotFound { message: String },

/// Git binary found but version is too old
#[error("git version too old: found {found}, minimum required {minimum}")]
GitVersionTooOld { found: String, minimum: String },
```

---

### 4. `git --version` Parsing Pattern

**Standard output format:** `git version 2.39.3` (macOS Xcode CLT) or `git version 2.48.1` (Homebrew git)

**Idiomatic Rust pattern** (from dandavison/delta and gitoxide):

```rust
fn parse_git_version(output: &[u8]) -> Option<(u32, u32, u32)> {
    let s = std::str::from_utf8(output).ok()?;
    let s = s.trim().strip_prefix("git version ")?;
    let mut parts = s.split('.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    let patch: u32 = parts.next()
        .and_then(|p| p.split_whitespace().next())  // handle "2.39.3.windows.1"
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    Some((major, minor, patch))
}
```

**Minimum version check:** `major > 2 || (major == 2 && minor >= 35)` — conservative; macOS Xcode CLT ships 2.39+ since Ventura (2022).

**This function is fully unit-testable** with string inputs, no git installation required.

---

### 5. `mod.rs` — Current Exports and What Needs Adding

**File:** `src-tauri/src/git/mod.rs`

Current exports:
```rust
pub mod cli;
pub mod error;
pub mod repository;

pub use cli::GitCli;
pub use error::GitError;
pub use repository::{Git2Repository, GitRepository, RefInfo, RefType};
```

**Needs adding:**
```rust
pub mod resolver;
pub use resolver::{GitResolver, ResolvedGit};
```

---

### 6. Integration Test Helper Pattern

The existing `create_test_repo()` in `mod.rs` is a `pub fn` inside a `#[cfg(test)]` block and returns `(TempDir, git2::Repository)`. The integration test for `GitResolver` can follow the same pattern:

```rust
// in mod.rs #[cfg(test)]
#[test]
fn test_resolver_end_to_end() {
    let (dir, _repo) = create_test_repo();
    let resolved = GitResolver::resolve().expect("git should be found on dev machine");
    let cli = GitCli::new(dir.path(), resolved.path);
    cli.create_branch("test-branch", None).expect("create branch should work");
    // Verify with git2
    let repo = Git2Repository::open(dir.path().to_str().unwrap());
    let branches = repo.branches().expect("branches should work");
    assert!(branches.iter().any(|b| b.name == "test-branch"));
}
```

---

## Architecture Summary

```
src-tauri/src/git/
├── mod.rs           # Add: pub mod resolver; pub use resolver::{GitResolver, ResolvedGit};
├── resolver.rs      # NEW: GitResolver::resolve() → Result<ResolvedGit, GitError>
├── cli.rs           # Refactor: add git_executable: PathBuf field, update new() + run_git()
├── repository.rs    # Unchanged
└── error.rs         # Add: GitNotFound, GitVersionTooOld variants
```

### `resolver.rs` Public API

```rust
/// The resolved git binary with its validated version.
pub struct ResolvedGit {
    pub path: PathBuf,
    pub version: GitVersion,
}

/// Parsed git version triple.
pub struct GitVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl GitVersion {
    pub fn meets_minimum(&self, major: u32, minor: u32) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }
}

pub struct GitResolver;

impl GitResolver {
    /// Resolve the git binary using: GIT_EXECUTABLE env var → system PATH → error.
    /// Validates version >= 2.35.
    pub fn resolve() -> Result<ResolvedGit, GitError>;
}
```

### Resolution Priority (Implementation)

```rust
pub fn resolve() -> Result<ResolvedGit, GitError> {
    // 1. GIT_EXECUTABLE env var override
    let git_path = if let Ok(exe) = std::env::var("GIT_EXECUTABLE") {
        PathBuf::from(exe)
    } else {
        // 2. System PATH lookup
        which::which_global("git")
            .map_err(|_| GitError::GitNotFound {
                message: "git not found in PATH. Install via: brew install git".into(),
            })?
    };

    // 3. Version validation
    let output = Command::new(&git_path)
        .arg("--version")
        .output()
        .map_err(|_| GitError::GitNotFound {
            message: format!("git at {} is not executable", git_path.display()),
        })?;

    let (major, minor, _patch) = parse_git_version(&output.stdout)
        .ok_or_else(|| GitError::GitVersionTooOld {
            found: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            minimum: "2.35.0".into(),
        })?;

    if major < 2 || (major == 2 && minor < 35) {
        return Err(GitError::GitVersionTooOld {
            found: format!("{major}.{minor}"),
            minimum: "2.35.0".into(),
        });
    }

    log::info!(
        "GitResolver: resolved git at {} (version {major}.{minor})",
        git_path.display()
    );

    Ok(ResolvedGit {
        path: git_path,
        version: GitVersion { major, minor, patch: _patch },
    })
}
```

---

## `Cargo.toml` Change

Add to `[dependencies]`:
```toml
which = "8"
```

No feature flags needed for macOS.

---

## Test Strategy

| Test | Type | Requires git? |
|------|------|---------------|
| `test_parse_git_version_valid` | Unit | No |
| `test_parse_git_version_old_returns_err` | Unit | No |
| `test_parse_git_version_invalid_output` | Unit | No |
| `test_env_var_override_uses_exact_path` | Unit (`GIT_EXECUTABLE=/usr/bin/git`) | Yes |
| `test_resolver_end_to_end` | Integration | Yes |
| `test_git_cli_uses_resolved_path` | Integration | Yes |

The `parse_git_version` function is the core pure-function to test exhaustively without installation.

---

## Risks & Notes

| Risk | Impact | Note |
|------|--------|------|
| `which_global` not finding git on fresh macOS | Low | macOS Xcode CLT ships git 2.39+. Error message covers the case. |
| `GIT_EXECUTABLE` env var edge cases | Low | Non-existent path → `Command::new().output()` returns `Io` error, caught cleanly |
| Existing test breakage from `GitCli::new()` signature change | Medium | 4 tests use `GitCli::new(path)`. All need updating. Can use `GitResolver::resolve().unwrap().path` or a helper `test_git_cli(path)` factory |
| `log::info!` not yet set up in mongit | Low | Use `eprintln!` for spike; add `log` crate properly in production |

---

## Open Items

None blocking implementation. The PRD is complete and well-specified.

---

## Recommendation

Proceed directly to implementation. All questions are answered at high confidence.

**Implementation order (mirrors PRD tasks):**
1. `error.rs` — add `GitNotFound` + `GitVersionTooOld` variants  
2. `resolver.rs` — implement `GitResolver` module with `parse_git_version` + `resolve()`  
3. `cli.rs` — inject `git_executable: PathBuf`, update `run_git()`, fix tests  
4. `mod.rs` — add resolver re-exports  
5. `Cargo.toml` — add `which = "8"`  
6. Integration test — resolver + write command end-to-end  
7. `docs/research/2026-03-bundled-git-strategy.md` — strategy document  

**Sources:**
- `src-tauri/src/git/cli.rs` — current `GitCli` implementation
- `src-tauri/src/git/error.rs` — current error types
- `src-tauri/src/git/mod.rs` — current module exports and tests
- `src-tauri/Cargo.toml` — current dependencies
- https://docs.rs/which/latest/which/ — `which` crate v8.0.2 API
- `docs/research/2026-03-13-git-engine-architecture-recommendation.md` — hybrid git architecture decision
- dandavison/delta `src/utils/git.rs` — `parse_git_version` pattern (GitHub)
- GitoxideLabs/gitoxide `tests/tools/src/lib.rs` — `parse_git_version` pattern (GitHub)
