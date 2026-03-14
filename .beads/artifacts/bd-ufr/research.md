# Research: bd-ufr — Git Engine Hybrid

**Bead:** bd-ufr  
**Date:** 2026-03-14  
**Depth:** Moderate (~20 tool calls)  
**Status:** Complete — all questions answered at medium+ confidence

---

## Questions & Answers

### Q1: What is the current codebase state and what needs to change?

**Status:** ✅ Answered — HIGH confidence

**Findings:**
- `src-tauri/src/commands.rs` has inline git2 code (no module structure)
- `src-tauri/src/lib.rs` registers only 2 commands: `greet`, `get_repo_status`
- No `src-tauri/src/git/` module exists yet — all 4 files are NEW
- `thiserror = "2"` is already in `Cargo.toml` (no change needed)
- `tokio = { version = "1", features = ["full"] }` is already present
- `tempfile` is NOT in `Cargo.toml` — needs to be added as `[dev-dependencies]`
- git2 `0.20` with `vendored-libgit2` feature is already configured

**Key files:**
- `src-tauri/src/commands.rs` — current `get_repo_status` has inline `git2::Repository::open()` at line ~20
- `src-tauri/src/lib.rs` — command registration at lines 7-10

---

### Q2: git2 API for read operations — exact method signatures?

**Status:** ✅ Answered — HIGH confidence (source: `docs.rs/git2/0.20.2`)

#### Status (already working in commands.rs)
```rust
repo.statuses(Some(git2::StatusOptions::new()
    .include_untracked(true)
    .recurse_untracked_dirs(true)))
// Returns Statuses; iterate with .iter()
// entry.status() → git2::Status flags (bitflags)
// entry.path() → Option<&str>
```

Status flags:
- **Staged:** `INDEX_NEW | INDEX_MODIFIED | INDEX_DELETED | INDEX_RENAMED | INDEX_TYPECHANGE`
- **Unstaged:** `WT_MODIFIED | WT_NEW | WT_DELETED | WT_RENAMED | WT_TYPECHANGE`

#### Diff (working dir vs index = `git diff`)
```rust
// diff between index and working dir
let diff = repo.diff_index_to_workdir(None, None)?;
// None for index = uses current repo index
// None for opts = default diff options

// To get diff patches, use diff.foreach():
diff.foreach(
    &mut |_file, _progress| true,        // file callback
    None,                                  // binary callback
    Some(&mut |_file, hunk| {
        // hunk.new_start(), hunk.new_lines(), hunk.header()
        true
    }),
    Some(&mut |_file, _hunk, line| {
        // line.origin() → char: ' ', '+', '-'
        // line.content() → &[u8]
        // line.old_lineno() → Option<u32>
        // line.new_lineno() → Option<u32>
        true
    }),
)?;
```

**Note:** To produce owned types, collect data into `Vec<DiffHunkOwned>` inside the foreach closures.

#### Log / Revwalk
```rust
let mut revwalk = repo.revwalk()?;
revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
revwalk.push_head()?;
// Iterator yields Result<Oid, Error>

for oid in revwalk.take(limit) {
    let oid = oid?;
    let commit = repo.find_commit(oid)?;
    // commit.id() → Oid
    // commit.summary() → Option<&str>  (first line of message)
    // commit.message() → Option<&str>  (full message)
    // commit.author() → Signature<'_>
    //   .name() → Option<&str>
    //   .email() → Option<&str>
    // commit.time() → git2::Time  → .seconds() → i64 (Unix timestamp)
    // commit.parent_ids() → impl Iterator<Item=Oid>
    // commit.parent_count() → usize
}
```

**Critical:** Commit is `!Send + !Sync`. Extract all data into owned types immediately inside `spawn_blocking`.

#### Branches
```rust
let branches = repo.branches(Some(git2::BranchType::Local))?;
// Or None to get all (local + remote)
for item in branches {
    let (branch, branch_type) = item?;
    // branch.name() → Result<Option<&str>, Error>  (short name e.g. "main")
    // branch.get().target() → Option<Oid>  (commit OID pointed to)
    // branch.is_head() → bool  (is current HEAD)
    // branch_type → BranchType::Local | BranchType::Remote
}
```

#### Current Branch
```rust
let head = repo.head()?;
// head.is_branch() → bool
// head.shorthand() → Option<&str>  (e.g. "main")
```

---

### Q3: Open-per-call vs Arc<Mutex<Repository>> — which pattern?

**Status:** ✅ Answered — HIGH confidence

**Decision: Open-per-call inside `spawn_blocking`**

Rationale:
- `git2::Repository` is `Send` but NOT `Sync` — cannot use `Arc<Repository>`
- `Arc<Mutex<Repository>>` works but serializes all operations (anti-pattern for concurrent IPC)
- GitButler uses `OnDemand<git2::Repository>` (lazy per-thread cache), but for MVP open-per-call is simpler and sufficient
- Opening a repo is ~0.5ms — negligible overhead for IPC commands

Pattern:
```rust
#[tauri::command]
pub async fn get_repo_status(state: State<'_, AppState>) -> Result<RepoStatus, String> {
    let path = state.repo_path.lock().unwrap().clone();
    tokio::task::spawn_blocking(move || {
        let repo = git2::Repository::open(&path)
            .map_err(|e| GitError::NotFound { path: path.display().to_string() })?;
        // ... use repo ...
        Ok(status)
    })
    .await
    .map_err(|e| format!("Thread error: {}", e))?
}
```

---

### Q4: thiserror 2 API — exact syntax for GitError enum?

**Status:** ✅ Answered — HIGH confidence (source: `docs.rs/thiserror/latest`)

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Repository not found: {path}")]
    NotFound { path: String },

    #[error("git2 error: {0}")]
    Git2(#[from] git2::Error),

    #[error("git CLI error: command={command}, exit={exit_code}\nstderr: {stderr}")]
    Cli {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// For Tauri IPC conversion
impl From<GitError> for String {
    fn from(e: GitError) -> String {
        e.to_string()
    }
}
```

Key: `#[from]` attribute auto-generates `From<git2::Error>` and `From<std::io::Error>` impl, and also marks as `source()`.

---

### Q5: CLI write-path — how to execute git commands?

**Status:** ✅ Answered — HIGH confidence

```rust
use std::process::Command;

pub fn create_branch(&self, name: &str, start_point: &str) -> Result<(), GitError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(&self.repo_path)
        .arg("branch")
        .arg(name)
        .arg(start_point)
        .output()
        .map_err(GitError::Io)?;

    if !output.status.success() {
        return Err(GitError::Cli {
            command: format!("git branch {} {}", name, start_point),
            exit_code: output.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(())
}
```

Use `git -C <path>` to specify the repo directory without `cd`. Use `--porcelain=v2 -z` for any status parsing (already used by the CLI module).

---

### Q6: tempfile — how to create test git repos?

**Status:** ✅ Answered — HIGH confidence (source: `docs.rs/tempfile`)

Pattern for test helper:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::process::Command;

    fn create_test_repo() -> (TempDir, git2::Repository) {
        let dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();

        // Configure identity for commits
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        drop(config);

        // Create initial commit
        let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();

        (dir, repo)
    }
}
```

**Critical:** Return `TempDir` from helper — if dropped, directory is deleted. Keep it alive for the test duration.

Cargo.toml addition:
```toml
[dev-dependencies]
tempfile = "3"
```

---

## Architecture Summary

### Module Structure (confirmed from PRD)

```
src-tauri/src/git/
├── mod.rs          → pub use + GitRepository trait
├── error.rs        → GitError enum (thiserror)
├── repository.rs   → Git2Repository { repo_path: PathBuf }
└── cli.rs          → GitCli { repo_path: PathBuf }
```

### Owned Return Types (avoids lifetime issues)

All structs must be fully owned — no `&str` or references into git2 objects:

```rust
#[derive(Debug, Serialize)]
pub struct CommitInfo {
    pub oid: String,          // hex string
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,       // Unix seconds
    pub parent_oids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String,       // "staged", "unstaged", "untracked"
}

#[derive(Debug, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub target_oid: String,
    pub is_head: bool,
    pub branch_type: String,  // "local" or "remote"
}

#[derive(Debug, Serialize)]
pub struct DiffHunkOwned {
    pub header: String,
    pub lines: Vec<DiffLineOwned>,
}

#[derive(Debug, Serialize)]
pub struct DiffLineOwned {
    pub origin: char,         // ' ', '+', '-'
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}
```

---

## Key Constraints Confirmed

| Constraint | Finding |
|---|---|
| `git2::Repository` thread safety | `Send` but `!Sync` — open-per-call in `spawn_blocking` |
| `Revwalk` thread safety | `!Send + !Sync` — must stay within `spawn_blocking` |
| `Commit` thread safety | `!Send + !Sync` — extract to owned types immediately |
| `thiserror` version | 2 already in Cargo.toml — supports same `#[derive(Error)]` syntax |
| CLI write-path | `git -C <path>` pattern; capture stdout+stderr with `.output()` |
| Test infrastructure | `TempDir::new()` + `git2::Repository::init()` + manual initial commit |

---

## Implementation Order (from PRD tasks)

Execute in dependency order:

1. **Parallel (no deps):**
   - `error.rs` + `mod.rs` scaffold (GitError enum)
   - `Cargo.toml` tempfile dev-dep + test helper in `mod.rs`

2. **After error.rs:**
   - `repository.rs` (depends on error.rs)
   - `cli.rs` (depends on error.rs, parallel with repository.rs)

3. **After repository.rs:**
   - Refactor `commands.rs` to use git module

4. **After commands.rs:**
   - Validate: grep for Arc<Mutex, verify spawn_blocking usage

---

## Sources

| Source | URL |
|---|---|
| git2 0.20.2 docs — Repository | https://docs.rs/git2/0.20.2/git2/struct.Repository |
| git2 0.20.2 docs — Revwalk | https://docs.rs/git2/0.20.2/git2/struct.Revwalk |
| git2 0.20.2 docs — Commit | https://docs.rs/git2/0.20.2/git2/struct.Commit |
| git2 0.20.2 docs — DiffDelta | https://docs.rs/git2/0.20.2/git2/struct.DiffDelta |
| git2 0.20.2 docs — DiffLine | https://docs.rs/git2/0.20.2/git2/struct.DiffLine |
| git2 0.20.2 docs — Branch | https://docs.rs/git2/0.20.2/git2/struct.Branch |
| thiserror docs | https://docs.rs/thiserror/latest/thiserror |
| tempfile docs | https://docs.rs/tempfile/latest/tempfile |
| Architecture recommendation | `docs/research/2026-03-13-git-engine-architecture-recommendation.md` |
| PRD | `.beads/artifacts/bd-ufr/prd.md` |

---

## Open Items

None — all questions answered at high confidence. Ready for implementation.
