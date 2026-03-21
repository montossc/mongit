# Hunk-Level Stage/Unstage Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use skill({ name: "executing-plans" }) to implement this plan task-by-task.

**Goal:** Enable users to stage/unstage individual hunks from `/repo/changes` using `git apply --cached` via the resolved git binary.

**Architecture:** Backend creates a new `staging.rs` module (following `branch.rs` pattern) with `stage_hunk()` and `unstage_hunk()` functions that build unified diff patches from git2's `DiffHunkInfo` and pipe them to `git apply --cached`. Frontend extends `diffStore` with staged diff loading and mutation methods, and the page renders a hunk panel with per-hunk action buttons.

**Tech Stack:** Rust (git2 + CLI), Svelte 5 runes, Tauri 2.0 IPC

---

## Must-Haves

**Goal:** Users can stage/unstage individual hunks from the changes workspace.

### Observable Truths

1. User can select a changed file and see its hunks (both staged and unstaged)
2. User can stage one unstaged hunk without affecting other hunks
3. User can unstage one staged hunk without affecting other hunks
4. Patch failures produce clear, structured error messages
5. File list badges and hunk panel refresh consistently after each mutation

### Required Artifacts

| Artifact | Provides | Path |
|----------|----------|------|
| StageOpError enum | Typed staging errors for frontend | `src-tauri/src/git/error.rs` |
| run_git_with_stdin | Pipe patch data to git CLI | `src-tauri/src/git/cli.rs` |
| diff_index() | Read staged changes (HEAD→index) | `src-tauri/src/git/repository.rs` |
| staging module | stage_hunk / unstage_hunk logic | `src-tauri/src/git/staging.rs` |
| IPC commands | stage_hunk / unstage_hunk / get_diff_index | `src-tauri/src/commands.rs` |
| Extended diffStore | Staged hunks + mutation methods | `src/lib/stores/diff.svelte.ts` |
| Hunk panel UI | Hunk display + action buttons | `src/routes/repo/changes/+page.svelte` |

### Key Links

| From | To | Via | Risk |
|------|-----|-----|------|
| staging.rs | git CLI | `run_git_with_stdin` pipe | Patch format must be exact unified diff |
| staging.rs | git2 diff | `diff_workdir()` / `diff_index()` | Hunk data must match current index state |
| diffStore | backend | `invoke()` IPC | Must refresh both stores after mutation |
| +page.svelte | diffStore | reactive state | Pending state must prevent duplicate clicks |

### Task Dependencies

```
Task 1 (Backend engine): needs nothing, creates staging.rs + error types + CLI stdin + diff_index + commands
Task 2 (Frontend state): needs Task 1, extends diffStore + changesStore coordination
Task 3 (Hunk UI): needs Task 2, extends +page.svelte with hunk panel
Task 4 (Integration verification): needs Task 3, verifies contracts

Wave 1: Task 1
Wave 2: Task 2
Wave 3: Task 3
Wave 4: Task 4
```

---

## Task 1: Backend hunk patch mutation engine

**Tier:** worker

**Files:**
- Modify: `src-tauri/src/git/error.rs` (add StageOpError + parse_stage_stderr)
- Modify: `src-tauri/src/git/cli.rs` (add run_git_with_stdin)
- Modify: `src-tauri/src/git/repository.rs` (add diff_index to trait + impl)
- Create: `src-tauri/src/git/staging.rs` (stage_hunk, unstage_hunk, build_hunk_patch)
- Modify: `src-tauri/src/git/mod.rs` (pub mod staging + staging tests)
- Modify: `src-tauri/src/commands.rs` (stage_hunk, unstage_hunk, get_diff_index commands)
- Modify: `src-tauri/src/lib.rs` (register new commands)

### Step 1: Add StageOpError to error.rs

After the `BranchOpError` enum and before `pub fn parse_branch_stderr`, add:

```rust
// ── Staging Operation Errors ────────────────────────────────────────────

/// Structured error for hunk staging operations.
///
/// Serializes as a discriminated union for typed frontend consumption:
/// `{ "kind": "PatchFailed", "reason": "...", "message": "..." }`
///
/// Raw stderr is always preserved in the `message` field for debugging.
#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[serde(tag = "kind")]
pub enum StageOpError {
    #[error("patch cannot be applied: {reason}")]
    PatchFailed { reason: String, message: String },

    #[error("hunk index {index} out of range (file has {total} hunks)")]
    InvalidHunkIndex {
        index: usize,
        total: usize,
        message: String,
    },

    #[error("file '{path}' not found in diff")]
    FileNotInDiff { path: String, message: String },

    #[error("binary file '{path}' not supported for partial staging")]
    BinaryNotSupported { path: String, message: String },

    #[error("staging operation failed: {stderr}")]
    GenericStageFailed {
        cmd: String,
        stderr: String,
        exit_code: Option<i32>,
    },
}

/// Parse git CLI stderr into a typed `StageOpError`.
///
/// Matches the most common `git apply` error patterns.
/// Falls back to `GenericStageFailed` for unrecognized stderr output.
pub fn parse_stage_stderr(cmd: &str, stderr: &str, exit_code: Option<i32>) -> StageOpError {
    let lower = stderr.to_lowercase();

    if lower.contains("patch does not apply") || lower.contains("does not apply") {
        return StageOpError::PatchFailed {
            reason: "patch does not apply to current state".to_string(),
            message: stderr.to_string(),
        };
    }

    if lower.contains("corrupt patch") || lower.contains("invalid patch") {
        return StageOpError::PatchFailed {
            reason: "corrupt or invalid patch format".to_string(),
            message: stderr.to_string(),
        };
    }

    if lower.contains("binary") && lower.contains("patch") {
        let path = extract_quoted(stderr).unwrap_or_default();
        return StageOpError::BinaryNotSupported {
            path,
            message: stderr.to_string(),
        };
    }

    StageOpError::GenericStageFailed {
        cmd: cmd.to_string(),
        stderr: stderr.to_string(),
        exit_code,
    }
}
```

Add the `StageOp` variant to the `GitError` enum (after `BranchOp`):

```rust
    /// Staging operation error (structured, serializable for frontend)
    #[error("{0}")]
    StageOp(#[from] StageOpError),
```

Update `From<GitError> for String` to handle `StageOp`:

```rust
impl From<GitError> for String {
    fn from(err: GitError) -> Self {
        match &err {
            GitError::BranchOp(branch_err) => {
                serde_json::to_string(branch_err).unwrap_or_else(|_| err.to_string())
            }
            GitError::StageOp(stage_err) => {
                serde_json::to_string(stage_err).unwrap_or_else(|_| err.to_string())
            }
            _ => err.to_string(),
        }
    }
}
```

### Step 2: Add run_git_with_stdin to cli.rs

Add this method to `impl GitCli`, after `run_git`:

```rust
    /// Run a git command with data piped to stdin.
    ///
    /// Used for `git apply --cached` where the patch is sent via stdin.
    /// Spawns a subprocess, writes `stdin_data`, then waits for completion.
    pub(crate) fn run_git_with_stdin(
        &self,
        args: &[&str],
        stdin_data: &[u8],
    ) -> Result<String, GitError> {
        use std::io::Write;
        use std::process::Stdio;

        let path_str = self.path.to_string_lossy();
        let mut child = Command::new(&self.git_executable)
            .arg("-C")
            .arg(path_str.as_ref())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(stdin_data)?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(GitError::CommandFailed {
                cmd: args.join(" "),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                exit_code: output.status.code(),
            })
        }
    }
```

Also add `use std::process::Stdio;` at the top if not already imported — actually it's brought in by `use std::process::Command;` at the top. We need `Stdio` explicitly. Check and add import as needed.

### Step 3: Add diff_index to repository.rs

Add `diff_index` to the `GitRepository` trait (after `diff_workdir`):

```rust
    /// Diff of the index against HEAD (staged changes).
    fn diff_index(&self) -> Result<Vec<DiffFileEntry>, GitError>;
```

Add the implementation in `impl GitRepository for Git2Repository` (after `diff_workdir`):

```rust
    fn diff_index(&self) -> Result<Vec<DiffFileEntry>, GitError> {
        use std::cell::RefCell;

        let repo = self.repo()?;

        // Get HEAD tree (None for initial/unborn commits)
        let head_tree = match repo.head() {
            Ok(head) => Some(head.peel_to_tree()?),
            Err(e) if e.code() == ErrorCode::UnbornBranch => None,
            Err(e) => return Err(e.into()),
        };

        let mut opts = DiffOptions::new();
        opts.include_typechange(true);

        let diff = repo.diff_tree_to_index(
            head_tree.as_ref(),
            None, // current index
            Some(&mut opts),
        )?;

        let entries: RefCell<Vec<DiffFileEntry>> = RefCell::new(Vec::new());

        diff.foreach(
            &mut |delta, _| {
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let status = match delta.status() {
                    Delta::Added => DiffFileStatus::Added,
                    Delta::Deleted => DiffFileStatus::Deleted,
                    Delta::Renamed => DiffFileStatus::Renamed,
                    _ => DiffFileStatus::Modified,
                };

                entries.borrow_mut().push(DiffFileEntry {
                    path,
                    status,
                    hunks: Vec::new(),
                });

                true
            },
            None,
            Some(&mut |_delta, hunk| {
                let mut entries = entries.borrow_mut();
                if let Some(file) = entries.last_mut() {
                    file.hunks.push(DiffHunkInfo {
                        old_start: hunk.old_start(),
                        old_lines: hunk.old_lines(),
                        new_start: hunk.new_start(),
                        new_lines: hunk.new_lines(),
                        header: String::from_utf8_lossy(hunk.header())
                            .trim_end_matches('\0')
                            .to_string(),
                        lines: Vec::new(),
                    });
                }
                true
            }),
            Some(&mut |_delta, _hunk, line| {
                let mut entries = entries.borrow_mut();
                if let Some(file) = entries.last_mut() {
                    if let Some(hunk) = file.hunks.last_mut() {
                        hunk.lines.push(DiffLineInfo {
                            origin: line.origin(),
                            content: String::from_utf8_lossy(line.content()).to_string(),
                            old_lineno: line.old_lineno(),
                            new_lineno: line.new_lineno(),
                        });
                    }
                }
                true
            }),
        )?;

        Ok(entries.into_inner())
    }
```

### Step 4: Create staging.rs

Create `src-tauri/src/git/staging.rs`:

```rust
//! Hunk-level stage/unstage operations.
//!
//! Builds unified diff patches from `DiffHunkInfo` and applies them
//! via `git apply --cached` for staging and `git apply --cached --reverse`
//! for unstaging. Follows the same module pattern as `branch.rs`.

use std::path::Path;

use super::cli::GitCli;
use super::error::{parse_stage_stderr, GitError, StageOpError};
use super::repository::{DiffFileEntry, DiffFileStatus, DiffHunkInfo, Git2Repository, GitRepository};

/// Map CommandFailed errors to typed StageOpError via stderr parsing.
fn map_cli_error(err: GitError) -> GitError {
    match err {
        GitError::CommandFailed {
            cmd,
            stderr,
            exit_code,
        } => GitError::StageOp(parse_stage_stderr(&cmd, &stderr, exit_code)),
        other => other,
    }
}

/// Build a unified diff patch for a single hunk.
///
/// Produces a patch in unified diff format suitable for `git apply`:
/// ```text
/// diff --git a/<path> b/<path>
/// --- a/<path>
/// +++ b/<path>
/// @@ -old_start,old_lines +new_start,new_lines @@
/// <lines>
/// ```
fn build_hunk_patch(path: &str, hunk: &DiffHunkInfo, status: &DiffFileStatus) -> String {
    let mut patch = String::new();

    // File header
    patch.push_str(&format!("diff --git a/{path} b/{path}\n"));

    match status {
        DiffFileStatus::Added => {
            patch.push_str("--- /dev/null\n");
            patch.push_str(&format!("+++ b/{path}\n"));
        }
        DiffFileStatus::Deleted => {
            patch.push_str(&format!("--- a/{path}\n"));
            patch.push_str("+++ /dev/null\n");
        }
        _ => {
            patch.push_str(&format!("--- a/{path}\n"));
            patch.push_str(&format!("+++ b/{path}\n"));
        }
    }

    // Hunk header
    patch.push_str(&format!(
        "@@ -{},{} +{},{} @@\n",
        hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines,
    ));

    // Lines
    for line in &hunk.lines {
        match line.origin {
            ' ' | '+' | '-' => {
                patch.push(line.origin);
                patch.push_str(&line.content);
                // git2 content may or may not have trailing \n
                // Don't force-add; the no-newline marker handles it
            }
            '>' | '<' | '=' => {
                // "No newline at end of file" marker
                patch.push_str("\\ No newline at end of file\n");
            }
            _ => {
                // Skip file headers, hunk headers, binary markers
            }
        }
    }

    patch
}

/// Find a file entry in a diff result by path.
fn find_file_in_diff<'a>(
    diff: &'a [DiffFileEntry],
    file_path: &str,
    context: &str,
) -> Result<&'a DiffFileEntry, GitError> {
    diff.iter()
        .find(|f| f.path == file_path)
        .ok_or_else(|| {
            GitError::StageOp(StageOpError::FileNotInDiff {
                path: file_path.to_string(),
                message: format!("file '{}' has no {} changes", file_path, context),
            })
        })
}

/// Validate and retrieve a hunk from a file entry by index.
fn get_hunk_at_index<'a>(
    file_entry: &'a DiffFileEntry,
    hunk_index: usize,
) -> Result<&'a DiffHunkInfo, GitError> {
    file_entry.hunks.get(hunk_index).ok_or_else(|| {
        GitError::StageOp(StageOpError::InvalidHunkIndex {
            index: hunk_index,
            total: file_entry.hunks.len(),
            message: format!(
                "hunk index {} out of range (file has {} hunks)",
                hunk_index,
                file_entry.hunks.len()
            ),
        })
    })
}

/// Stage a single hunk from a file's working-tree changes into the index.
///
/// Reads the unstaged diff (index→workdir), extracts the specified hunk,
/// builds a unified diff patch, and applies it via `git apply --cached`.
///
/// Only the targeted hunk is staged; sibling hunks remain unstaged.
pub fn stage_hunk(
    path: &Path,
    git_executable: &Path,
    file_path: &str,
    hunk_index: usize,
) -> Result<(), GitError> {
    // 1. Read unstaged diff (index → workdir)
    let repo = Git2Repository::open(path);
    let diff = repo.diff_workdir()?;

    // 2. Find file and hunk
    let file_entry = find_file_in_diff(&diff, file_path, "unstaged")?;
    let hunk = get_hunk_at_index(file_entry, hunk_index)?;

    // 3. Build patch
    let patch = build_hunk_patch(file_path, hunk, &file_entry.status);

    // 4. Apply via git apply --cached
    let cli = GitCli::new(path, git_executable);
    cli.run_git_with_stdin(
        &["apply", "--cached", "--unidiff-zero"],
        patch.as_bytes(),
    )
    .map_err(map_cli_error)?;

    Ok(())
}

/// Unstage a single hunk from the index back to the working tree.
///
/// Reads the staged diff (HEAD→index), extracts the specified hunk,
/// builds a unified diff patch, and reverse-applies it via
/// `git apply --cached --reverse`.
///
/// Only the targeted hunk is unstaged; sibling hunks remain staged.
pub fn unstage_hunk(
    path: &Path,
    git_executable: &Path,
    file_path: &str,
    hunk_index: usize,
) -> Result<(), GitError> {
    // 1. Read staged diff (HEAD → index)
    let repo = Git2Repository::open(path);
    let diff = repo.diff_index()?;

    // 2. Find file and hunk
    let file_entry = find_file_in_diff(&diff, file_path, "staged")?;
    let hunk = get_hunk_at_index(file_entry, hunk_index)?;

    // 3. Build patch
    let patch = build_hunk_patch(file_path, hunk, &file_entry.status);

    // 4. Apply via git apply --cached --reverse
    let cli = GitCli::new(path, git_executable);
    cli.run_git_with_stdin(
        &["apply", "--cached", "--reverse", "--unidiff-zero"],
        patch.as_bytes(),
    )
    .map_err(map_cli_error)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::tests::create_test_repo;
    use std::path::Path;

    /// Helper: write a file with multiple hunks separated by unchanged content.
    fn write_multi_hunk_file(dir: &std::path::Path) {
        // Original content (committed in create_test_repo as initial.txt)
        // Overwrite with multi-hunk changes
        let content = "\
modified first line
line 2
line 3
line 4
line 5
line 6
line 7
line 8
line 9
modified last line
";
        std::fs::write(dir.join("initial.txt"), content).unwrap();
    }

    /// Helper: create a repo with a file that has enough lines for multi-hunk diffs.
    fn create_repo_with_content() -> (tempfile::TempDir, git2::Repository) {
        let (dir, repo) = create_test_repo();

        // Overwrite initial.txt with more lines so we get context-separated hunks
        let original = "\
line 1
line 2
line 3
line 4
line 5
line 6
line 7
line 8
line 9
line 10
";
        std::fs::write(dir.path().join("initial.txt"), original).unwrap();

        // Stage and commit
        let mut index = repo.index().unwrap();
        index
            .add_path(std::path::Path::new("initial.txt"))
            .unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Add multi-line file",
            &tree,
            &[&head],
        )
        .unwrap();

        (dir, repo)
    }

    #[test]
    fn test_build_hunk_patch_format() {
        let hunk = DiffHunkInfo {
            old_start: 1,
            old_lines: 3,
            new_start: 1,
            new_lines: 4,
            header: String::new(),
            lines: vec![
                super::super::repository::DiffLineInfo {
                    origin: ' ',
                    content: "context\n".to_string(),
                    old_lineno: Some(1),
                    new_lineno: Some(1),
                },
                super::super::repository::DiffLineInfo {
                    origin: '-',
                    content: "old line\n".to_string(),
                    old_lineno: Some(2),
                    new_lineno: None,
                },
                super::super::repository::DiffLineInfo {
                    origin: '+',
                    content: "new line\n".to_string(),
                    old_lineno: None,
                    new_lineno: Some(2),
                },
                super::super::repository::DiffLineInfo {
                    origin: '+',
                    content: "added line\n".to_string(),
                    old_lineno: None,
                    new_lineno: Some(3),
                },
                super::super::repository::DiffLineInfo {
                    origin: ' ',
                    content: "context2\n".to_string(),
                    old_lineno: Some(3),
                    new_lineno: Some(4),
                },
            ],
        };

        let patch = build_hunk_patch("test.txt", &hunk, &DiffFileStatus::Modified);
        assert!(patch.starts_with("diff --git a/test.txt b/test.txt\n"));
        assert!(patch.contains("--- a/test.txt\n"));
        assert!(patch.contains("+++ b/test.txt\n"));
        assert!(patch.contains("@@ -1,3 +1,4 @@\n"));
        assert!(patch.contains(" context\n"));
        assert!(patch.contains("-old line\n"));
        assert!(patch.contains("+new line\n"));
        assert!(patch.contains("+added line\n"));
    }

    #[test]
    fn test_build_hunk_patch_added_file() {
        let hunk = DiffHunkInfo {
            old_start: 0,
            old_lines: 0,
            new_start: 1,
            new_lines: 1,
            header: String::new(),
            lines: vec![super::super::repository::DiffLineInfo {
                origin: '+',
                content: "new content\n".to_string(),
                old_lineno: None,
                new_lineno: Some(1),
            }],
        };

        let patch = build_hunk_patch("new.txt", &hunk, &DiffFileStatus::Added);
        assert!(patch.contains("--- /dev/null\n"));
        assert!(patch.contains("+++ b/new.txt\n"));
    }

    #[test]
    fn test_stage_hunk_single_hunk_file() {
        let (dir, _repo) = create_repo_with_content();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Modify initial.txt
        std::fs::write(
            repo_path.join("initial.txt"),
            "MODIFIED line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n",
        )
        .unwrap();

        // Stage hunk 0
        stage_hunk(repo_path, git, "initial.txt", 0).unwrap();

        // Verify: staged diff should show the change
        let repo = Git2Repository::open(repo_path);
        let staged = repo.diff_index().unwrap();
        assert!(
            staged.iter().any(|f| f.path == "initial.txt"),
            "initial.txt should have staged changes"
        );
    }

    #[test]
    fn test_stage_hunk_multi_hunk_isolation() {
        let (dir, _repo) = create_repo_with_content();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Modify first and last lines (creates 2 hunks with gap > 3 context lines)
        std::fs::write(
            repo_path.join("initial.txt"),
            "MODIFIED line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nMODIFIED line 10\n",
        )
        .unwrap();

        // Verify we have 2 hunks
        let repo = Git2Repository::open(repo_path);
        let diff = repo.diff_workdir().unwrap();
        let file = diff.iter().find(|f| f.path == "initial.txt").unwrap();
        assert_eq!(file.hunks.len(), 2, "should have 2 hunks");

        // Stage only hunk 0 (first line change)
        stage_hunk(repo_path, git, "initial.txt", 0).unwrap();

        // Verify: hunk 1 should still be unstaged
        let repo2 = Git2Repository::open(repo_path);
        let unstaged = repo2.diff_workdir().unwrap();
        let remaining = unstaged.iter().find(|f| f.path == "initial.txt");
        assert!(
            remaining.is_some(),
            "file should still have unstaged changes (hunk 1)"
        );
        assert_eq!(
            remaining.unwrap().hunks.len(),
            1,
            "only one hunk should remain unstaged"
        );

        // Verify: staged diff should show only hunk 0
        let staged = repo2.diff_index().unwrap();
        let staged_file = staged.iter().find(|f| f.path == "initial.txt").unwrap();
        assert_eq!(
            staged_file.hunks.len(),
            1,
            "only one hunk should be staged"
        );
    }

    #[test]
    fn test_unstage_hunk() {
        let (dir, _repo) = create_repo_with_content();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Stage a modification via git add
        std::fs::write(
            repo_path.join("initial.txt"),
            "MODIFIED\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n",
        )
        .unwrap();

        let cli = GitCli::new(repo_path, git);
        cli.run_git(&["add", "initial.txt"]).unwrap();

        // Verify it's staged
        let repo = Git2Repository::open(repo_path);
        let staged = repo.diff_index().unwrap();
        assert!(staged.iter().any(|f| f.path == "initial.txt"));

        // Unstage hunk 0
        unstage_hunk(repo_path, git, "initial.txt", 0).unwrap();

        // Verify: no longer staged
        let repo2 = Git2Repository::open(repo_path);
        let staged2 = repo2.diff_index().unwrap();
        assert!(
            !staged2.iter().any(|f| f.path == "initial.txt"),
            "initial.txt should not have staged changes after unstage"
        );

        // Verify: still shows as unstaged (working tree change preserved)
        let unstaged = repo2.diff_workdir().unwrap();
        assert!(
            unstaged.iter().any(|f| f.path == "initial.txt"),
            "initial.txt should still have unstaged changes"
        );
    }

    #[test]
    fn test_stage_hunk_invalid_file() {
        let (dir, _repo) = create_repo_with_content();
        let repo_path = dir.path();
        let git = Path::new("git");

        let err = stage_hunk(repo_path, git, "nonexistent.txt", 0).unwrap_err();
        match err {
            GitError::StageOp(StageOpError::FileNotInDiff { path, .. }) => {
                assert_eq!(path, "nonexistent.txt");
            }
            other => panic!("expected FileNotInDiff, got: {:?}", other),
        }
    }

    #[test]
    fn test_stage_hunk_invalid_index() {
        let (dir, _repo) = create_repo_with_content();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Create a change
        std::fs::write(repo_path.join("initial.txt"), "changed\n").unwrap();

        let err = stage_hunk(repo_path, git, "initial.txt", 99).unwrap_err();
        match err {
            GitError::StageOp(StageOpError::InvalidHunkIndex { index, .. }) => {
                assert_eq!(index, 99);
            }
            other => panic!("expected InvalidHunkIndex, got: {:?}", other),
        }
    }

    #[test]
    fn test_stage_op_error_serializes_as_json() {
        let err = StageOpError::PatchFailed {
            reason: "does not apply".to_string(),
            message: "error: patch does not apply".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"PatchFailed\""));
        assert!(json.contains("\"reason\":\"does not apply\""));
    }

    #[test]
    fn test_git_error_stage_op_serializes() {
        let stage_err = StageOpError::FileNotInDiff {
            path: "gone.txt".to_string(),
            message: "not in diff".to_string(),
        };
        let git_err = GitError::StageOp(stage_err);
        let s: String = git_err.into();
        assert!(s.contains("FileNotInDiff"));
        assert!(s.contains("gone.txt"));
    }

    #[test]
    fn test_diff_index_empty_for_clean_repo() {
        let (dir, _repo) = create_test_repo();
        let repo = Git2Repository::open(dir.path());
        let staged = repo.diff_index().unwrap();
        assert!(staged.is_empty(), "clean repo should have no staged changes");
    }

    #[test]
    fn test_diff_index_shows_staged_changes() {
        let (dir, _repo) = create_test_repo();
        let repo_path = dir.path();

        // Stage a new file
        std::fs::write(repo_path.join("staged.txt"), "staged content\n").unwrap();
        let g2 = git2::Repository::open(repo_path).unwrap();
        let mut index = g2.index().unwrap();
        index
            .add_path(std::path::Path::new("staged.txt"))
            .unwrap();
        index.write().unwrap();

        let repo = Git2Repository::open(repo_path);
        let staged = repo.diff_index().unwrap();
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].path, "staged.txt");
        assert!(!staged[0].hunks.is_empty());
    }
}
```

### Step 5: Register staging module in mod.rs

Add `pub mod staging;` to `src-tauri/src/git/mod.rs`:

```rust
pub mod branch;
pub mod cli;
pub mod error;
pub mod repository;
pub mod resolver;
pub mod staging;
```

### Step 6: Add IPC commands to commands.rs

Add the imports at the top of `commands.rs`:

```rust
use crate::git::staging;
```

Add three new commands after the branch commands:

```rust
// ── Staging operation commands ─────────────────────────────────────────────────

/// Stage a single hunk from the working tree into the index.
#[tauri::command]
pub async fn stage_hunk(
    path: String,
    file_path: String,
    hunk_index: usize,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        staging::stage_hunk(&path, &git, &file_path, hunk_index).map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Unstage a single hunk from the index back to the working tree.
#[tauri::command]
pub async fn unstage_hunk(
    path: String,
    file_path: String,
    hunk_index: usize,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        staging::unstage_hunk(&path, &git, &file_path, hunk_index).map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Get staged changes (HEAD → index diff) for hunk display.
#[tauri::command]
pub async fn get_diff_index(path: String) -> Result<Vec<DiffFileEntry>, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Git2Repository::open(&path);
        repo.diff_index().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
```

### Step 7: Register commands in lib.rs

Add to the `invoke_handler` in `src-tauri/src/lib.rs`:

```rust
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_repo_status,
            commands::get_commit_log,
            commands::get_refs,
            commands::get_diff_workdir,
            commands::get_changed_files,
            commands::get_file_content_for_diff,
            commands::create_branch,
            commands::switch_branch,
            commands::delete_branch,
            commands::fetch,
            commands::pull,
            commands::push,
            commands::open_repo,
            commands::get_recent_repos,
            commands::remove_recent_repo,
            commands::stage_hunk,
            commands::unstage_hunk,
            commands::get_diff_index,
            watcher::watch_repo,
            watcher::stop_watching,
        ])
```

### Step 8: Verify backend

Run in `src-tauri/`:

```bash
cargo check
cargo test
```

Expected: All tests pass, including new staging tests.

### Handoff Contract

**Produces:**
- `staging::stage_hunk(path, git, file_path, hunk_index)` — stages one hunk
- `staging::unstage_hunk(path, git, file_path, hunk_index)` — unstages one hunk
- IPC: `stage_hunk`, `unstage_hunk`, `get_diff_index` commands
- `StageOpError` structured errors serialized as JSON

**Consumed By:**
- Task 2: Frontend state layer
- Task 3: UI components

---

## Task 2: Workspace diff and hunk action state

**Tier:** worker

**Files:**
- Modify: `src/lib/stores/diff.svelte.ts` (add staged diff + mutations)
- Modify: `src/lib/stores/changes.svelte.ts` (no structural changes, just refresh coordination)

### Step 1: Extend diffStore with staged diff loading and mutations

Replace `src/lib/stores/diff.svelte.ts` with this extended version.

Key changes from current version:
- Add `stagedFiles` state for HEAD→index diff
- Add `stagedRequestId` guard
- Modify `fetchDiff()` to load both unstaged and staged diffs in parallel
- Add `staging` and `stagingError` state for mutation in-flight tracking
- Add `stageHunk()` and `unstageHunk()` mutation methods
- Add computed getters for selected file's unstaged/staged hunks
- Update selection logic to consider both unstaged and staged file lists

```typescript
import { invoke } from "@tauri-apps/api/core";

// ── Types matching Rust serialization ────────────────────────────────────

export interface DiffLineInfo {
	origin: string; // ' ' | '+' | '-' | '\\'
	content: string;
	old_lineno: number | null;
	new_lineno: number | null;
}

export interface DiffHunkInfo {
	old_start: number;
	old_lines: number;
	new_start: number;
	new_lines: number;
	header: string;
	lines: DiffLineInfo[];
}

export type DiffFileStatus = "Added" | "Modified" | "Deleted" | "Renamed";

export interface DiffFileEntry {
	path: string;
	status: DiffFileStatus;
	hunks: DiffHunkInfo[];
}

export interface FileContentPair {
	original: string;
	modified: string;
}

// ── Store ────────────────────────────────────────────────────────────────

function createDiffStore() {
	let files = $state<DiffFileEntry[]>([]);
	let stagedFiles = $state<DiffFileEntry[]>([]);
	let selectedPath = $state<string | null>(null);
	let content = $state<FileContentPair | null>(null);
	let loading = $state(false);
	let loadingContent = $state(false);
	let error = $state<string | null>(null);
	let repoPath = $state("");
	let diffRequestId = 0; // Guard against stale repo-level diff responses
	let contentRequestId = 0; // Guard against out-of-order async responses

	// Staging mutation state
	let staging = $state(false);
	let stagingError = $state<string | null>(null);

	/** Fetch both unstaged and staged diffs for a repository. */
	async function fetchDiff(path: string): Promise<boolean> {
		diffRequestId += 1;
		const thisRequest = diffRequestId;
		loading = true;
		error = null;
		repoPath = path;

		try {
			const [nextFiles, nextStaged] = await Promise.all([
				invoke<DiffFileEntry[]>("get_diff_workdir", { path }),
				invoke<DiffFileEntry[]>("get_diff_index", { path }),
			]);
			if (thisRequest !== diffRequestId || repoPath !== path) {
				return false;
			}

			files = nextFiles;
			stagedFiles = nextStaged;

			// Validate selection against both unstaged and staged files
			const allPaths = new Set([
				...nextFiles.map((f) => f.path),
				...nextStaged.map((f) => f.path),
			]);

			if (allPaths.size > 0) {
				const stillValid = selectedPath && allPaths.has(selectedPath);
				if (!stillValid) {
					const firstPath =
						nextFiles[0]?.path ?? nextStaged[0]?.path ?? null;
					if (firstPath) {
						await selectFile(firstPath);
					} else {
						selectedPath = null;
						content = null;
					}
				} else {
					await fetchContent(selectedPath!);
				}
			} else {
				selectedPath = null;
				content = null;
			}

			return true;
		} catch (e) {
			if (thisRequest === diffRequestId && repoPath === path) {
				error = String(e);
				files = [];
				stagedFiles = [];
				selectedPath = null;
				content = null;
			}
			return false;
		} finally {
			if (thisRequest === diffRequestId) {
				loading = false;
			}
		}
	}

	/** Select a file and fetch its full content for diff rendering. */
	async function selectFile(path: string): Promise<void> {
		selectedPath = path;
		await fetchContent(path);
	}

	/** Internal: fetch file content pair with race-condition guard. */
	async function fetchContent(filePath: string): Promise<void> {
		if (!repoPath) return;
		contentRequestId += 1;
		const thisRequest = contentRequestId;
		loadingContent = true;
		try {
			const result = await invoke<FileContentPair>(
				"get_file_content_for_diff",
				{
					path: repoPath,
					filePath,
				},
			);
			// Only apply if this is still the latest request
			if (thisRequest === contentRequestId) {
				content = result;
			}
		} catch (e) {
			if (thisRequest === contentRequestId) {
				error = String(e);
				content = null;
			}
		} finally {
			if (thisRequest === contentRequestId) {
				loadingContent = false;
			}
		}
	}

	/** Stage a single hunk. Returns true on success. */
	async function stageHunk(
		filePath: string,
		hunkIndex: number,
	): Promise<boolean> {
		if (staging || !repoPath) return false;
		staging = true;
		stagingError = null;
		try {
			await invoke("stage_hunk", {
				path: repoPath,
				filePath,
				hunkIndex,
			});
			return true;
		} catch (e) {
			stagingError = String(e);
			return false;
		} finally {
			staging = false;
		}
	}

	/** Unstage a single hunk. Returns true on success. */
	async function unstageHunk(
		filePath: string,
		hunkIndex: number,
	): Promise<boolean> {
		if (staging || !repoPath) return false;
		staging = true;
		stagingError = null;
		try {
			await invoke("unstage_hunk", {
				path: repoPath,
				filePath,
				hunkIndex,
			});
			return true;
		} catch (e) {
			stagingError = String(e);
			return false;
		} finally {
			staging = false;
		}
	}

	/** Reset store to initial state. */
	function reset(): void {
		files = [];
		stagedFiles = [];
		selectedPath = null;
		content = null;
		loading = false;
		loadingContent = false;
		error = null;
		staging = false;
		stagingError = null;
		repoPath = "";
	}

	/** Re-fetch diff for the current repo (no-op if no repo loaded or already loading). */
	async function refresh(): Promise<void> {
		if (!repoPath || loading) return;
		await fetchDiff(repoPath);
	}

	return {
		get files() {
			return files;
		},
		get stagedFiles() {
			return stagedFiles;
		},
		get selectedPath() {
			return selectedPath;
		},
		/** Unstaged hunks for the selected file. */
		get selectedFileUnstagedHunks(): DiffHunkInfo[] {
			if (!selectedPath) return [];
			return files.find((f) => f.path === selectedPath)?.hunks ?? [];
		},
		/** Staged hunks for the selected file. */
		get selectedFileStagedHunks(): DiffHunkInfo[] {
			if (!selectedPath) return [];
			return (
				stagedFiles.find((f) => f.path === selectedPath)?.hunks ?? []
			);
		},
		get content() {
			return content;
		},
		get loading() {
			return loading;
		},
		get loadingContent() {
			return loadingContent;
		},
		get error() {
			return error;
		},
		get staging() {
			return staging;
		},
		get stagingError() {
			return stagingError;
		},
		get repoPath() {
			return repoPath;
		},
		fetchDiff,
		selectFile,
		stageHunk,
		unstageHunk,
		refresh,
		reset,
	};
}

export const diffStore = createDiffStore();
```

### Step 2: Verify frontend

```bash
pnpm check
```

Expected: No type errors.

### Handoff Contract

**Produces:**
- `diffStore.selectedFileUnstagedHunks` — reactive unstaged hunks for selected file
- `diffStore.selectedFileStagedHunks` — reactive staged hunks for selected file
- `diffStore.stageHunk(filePath, hunkIndex)` — stage one hunk
- `diffStore.unstageHunk(filePath, hunkIndex)` — unstage one hunk
- `diffStore.staging` — whether a mutation is in flight
- `diffStore.stagingError` — last mutation error

**Consumed By:**
- Task 3: Hunk panel UI

---

## Task 3: Changes workspace hunk action UI

**Tier:** worker

**Files:**
- Modify: `src/routes/repo/changes/+page.svelte` (add hunk panel)

### Step 1: Extend +page.svelte with hunk panel

The page currently shows a file list. Add a right-side hunk panel that appears when a file is selected, showing unstaged and staged hunks with action buttons.

Key additions:
- Import `diffStore` and use it for hunk data
- Load diff data alongside changes data on mount
- Add hunk panel to the right of the file list
- Each hunk shows: header, diff lines (color-coded), action button
- Pending state disables buttons and shows spinner
- Error state shows staging error with dismiss
- Empty state explains when no hunks are available

The page should have a split layout: file list (left) | hunk panel (right).

Replace the full `+page.svelte` content with:

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { changesStore, type FileChangeKind } from '$lib/stores/changes.svelte';
	import { diffStore, type DiffHunkInfo } from '$lib/stores/diff.svelte';
	import { listen } from '@tauri-apps/api/event';

	let unlisten: (() => void) | null = null;
	let mounted = true;

	onMount(() => {
		// Load files and diff when the route mounts
		if (repoStore.activeRepoPath) {
			changesStore.loadFiles(repoStore.activeRepoPath);
			diffStore.fetchDiff(repoStore.activeRepoPath);
		}

		// Listen for file system changes and refresh
		const setupListener = async () => {
			const cb = await listen('repo-changed', () => {
				changesStore.refresh();
				diffStore.refresh();
			});
			if (mounted) {
				unlisten = cb;
			} else {
				cb(); // Already unmounted — clean up immediately
			}
		};
		setupListener();

		return () => {
			mounted = false;
			if (unlisten) unlisten();
		};
	});

	/** Handle file selection — update both stores. */
	function handleFileSelect(path: string) {
		changesStore.selectFile(path);
		diffStore.selectFile(path);
	}

	/** Handle staging a hunk, then refresh both stores. */
	async function handleStageHunk(filePath: string, hunkIndex: number) {
		const success = await diffStore.stageHunk(filePath, hunkIndex);
		if (success) {
			await Promise.all([diffStore.refresh(), changesStore.refresh()]);
		}
	}

	/** Handle unstaging a hunk, then refresh both stores. */
	async function handleUnstageHunk(filePath: string, hunkIndex: number) {
		const success = await diffStore.unstageHunk(filePath, hunkIndex);
		if (success) {
			await Promise.all([diffStore.refresh(), changesStore.refresh()]);
		}
	}

	/** Map a FileChangeKind to a short display label. */
	function kindLabel(kind: FileChangeKind): string {
		switch (kind) {
			case 'Added': return 'A';
			case 'Modified': return 'M';
			case 'Deleted': return 'D';
			case 'Renamed': return 'R';
			case 'Typechange': return 'T';
		}
	}

	/** Map a FileChangeKind to a CSS modifier class. */
	function kindClass(kind: FileChangeKind): string {
		switch (kind) {
			case 'Added': return 'added';
			case 'Modified': return 'modified';
			case 'Deleted': return 'deleted';
			case 'Renamed': return 'renamed';
			case 'Typechange': return 'typechange';
		}
	}

	/** Get the filename from a path. */
	function fileName(path: string): string {
		const parts = path.split('/');
		return parts[parts.length - 1];
	}

	/** Get the directory from a path, or empty string. */
	function fileDir(path: string): string {
		const lastSlash = path.lastIndexOf('/');
		return lastSlash > 0 ? path.substring(0, lastSlash + 1) : '';
	}

	/** Format diff line origin for display. */
	function lineClass(origin: string): string {
		switch (origin) {
			case '+': return 'line-add';
			case '-': return 'line-del';
			default: return 'line-ctx';
		}
	}

	/** Format hunk header for display. */
	function formatHunkHeader(hunk: DiffHunkInfo): string {
		return `@@ -${hunk.old_start},${hunk.old_lines} +${hunk.new_start},${hunk.new_lines} @@`;
	}

	// Reactive: whether we have any hunks to show
	const hasUnstagedHunks = $derived(diffStore.selectedFileUnstagedHunks.length > 0);
	const hasStagedHunks = $derived(diffStore.selectedFileStagedHunks.length > 0);
	const hasAnyHunks = $derived(hasUnstagedHunks || hasStagedHunks);
</script>

<div class="changes-workspace">
	{#if changesStore.loading && changesStore.files.length === 0}
		<!-- Loading state (only when no cached files) -->
		<div class="state-message">
			<div class="spinner"></div>
			<p>Loading changed files…</p>
		</div>

	{:else if changesStore.error}
		<!-- Error state -->
		<div class="state-message error">
			<p class="state-label">Error loading changes</p>
			<p class="state-detail">{changesStore.error}</p>
			<button class="retry-btn" onclick={() => changesStore.refresh()}>Retry</button>
		</div>

	{:else if changesStore.files.length === 0}
		<!-- Empty state: clean repo -->
		<div class="state-message">
			<p class="state-label">No changes</p>
			<p class="state-detail">Working tree is clean</p>
		</div>

	{:else}
		<div class="split-layout">
			<!-- File list (left panel) -->
			<div class="file-list-panel">
				<div class="file-list" role="listbox" aria-label="Changed files">
					{#each changesStore.files as file (file.path)}
						<button
							class="file-row"
							class:selected={changesStore.selectedPath === file.path}
							role="option"
							aria-selected={changesStore.selectedPath === file.path}
							onclick={() => handleFileSelect(file.path)}
						>
							<span class="file-path">
								{#if fileDir(file.path)}
									<span class="file-dir">{fileDir(file.path)}</span>
								{/if}
								<span class="file-name">{fileName(file.path)}</span>
							</span>

							<span class="file-badges">
								{#if file.staged}
									<span class="status-badge staged {kindClass(file.staged)}" title="Staged: {file.staged}">
										{kindLabel(file.staged)}
									</span>
								{/if}
								{#if file.unstaged}
									<span class="status-badge unstaged {kindClass(file.unstaged)}" title="Unstaged: {file.unstaged}">
										{kindLabel(file.unstaged)}
									</span>
								{/if}
							</span>
						</button>
					{/each}
				</div>
			</div>

			<!-- Hunk panel (right panel) -->
			<div class="hunk-panel">
				{#if !changesStore.selectedPath}
					<div class="state-message">
						<p class="state-detail">Select a file to view hunks</p>
					</div>

				{:else if diffStore.loading}
					<div class="state-message">
						<div class="spinner"></div>
						<p>Loading diff…</p>
					</div>

				{:else if !hasAnyHunks}
					<div class="state-message">
						<p class="state-label">No hunks</p>
						<p class="state-detail">This file has no renderable text hunks</p>
					</div>

				{:else}
					<div class="hunk-scroll">
						{#if diffStore.stagingError}
							<div class="staging-error">
								<p>{diffStore.stagingError}</p>
							</div>
						{/if}

						<!-- Unstaged hunks -->
						{#if hasUnstagedHunks}
							<div class="hunk-section">
								<h3 class="hunk-section-title">Unstaged Changes</h3>
								{#each diffStore.selectedFileUnstagedHunks as hunk, hunkIndex}
									<div class="hunk-block">
										<div class="hunk-header">
											<span class="hunk-header-text">{formatHunkHeader(hunk)}</span>
											<button
												class="hunk-action-btn stage-btn"
												disabled={diffStore.staging}
												onclick={() => handleStageHunk(changesStore.selectedPath!, hunkIndex)}
												title="Stage this hunk"
											>
												{#if diffStore.staging}
													<span class="btn-spinner"></span>
												{:else}
													Stage
												{/if}
											</button>
										</div>
										<div class="hunk-lines">
											{#each hunk.lines as line}
												{#if line.origin === ' ' || line.origin === '+' || line.origin === '-'}
													<div class="diff-line {lineClass(line.origin)}">
														<span class="line-origin">{line.origin}</span>
														<span class="line-content">{line.content}</span>
													</div>
												{/if}
											{/each}
										</div>
									</div>
								{/each}
							</div>
						{/if}

						<!-- Staged hunks -->
						{#if hasStagedHunks}
							<div class="hunk-section">
								<h3 class="hunk-section-title">Staged Changes</h3>
								{#each diffStore.selectedFileStagedHunks as hunk, hunkIndex}
									<div class="hunk-block">
										<div class="hunk-header">
											<span class="hunk-header-text">{formatHunkHeader(hunk)}</span>
											<button
												class="hunk-action-btn unstage-btn"
												disabled={diffStore.staging}
												onclick={() => handleUnstageHunk(changesStore.selectedPath!, hunkIndex)}
												title="Unstage this hunk"
											>
												{#if diffStore.staging}
													<span class="btn-spinner"></span>
												{:else}
													Unstage
												{/if}
											</button>
										</div>
										<div class="hunk-lines">
											{#each hunk.lines as line}
												{#if line.origin === ' ' || line.origin === '+' || line.origin === '-'}
													<div class="diff-line {lineClass(line.origin)}">
														<span class="line-origin">{line.origin}</span>
														<span class="line-content">{line.content}</span>
													</div>
												{/if}
											{/each}
										</div>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.changes-workspace {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* ── Split layout ──────────────────────────────────────────────── */

	.split-layout {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.file-list-panel {
		width: 280px;
		min-width: 200px;
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.hunk-panel {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* ── State messages ─────────────────────────────────────────────── */

	.state-message {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-3);
		color: var(--color-text-muted);
		padding: var(--space-8);
	}

	.state-message.error {
		color: var(--color-danger);
	}

	.state-label {
		font-size: var(--text-body-size);
		font-weight: 500;
		margin: 0;
	}

	.state-detail {
		font-size: var(--text-body-sm-size);
		margin: 0;
		max-width: 360px;
		text-align: center;
		word-break: break-word;
	}

	.state-message.error .state-detail {
		color: var(--color-text-secondary);
	}

	.retry-btn {
		margin-top: var(--space-3);
		padding: var(--space-2) var(--space-5);
		font-size: var(--text-body-sm-size);
		background: none;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-secondary);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.retry-btn:hover {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.spinner {
		width: 20px;
		height: 20px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	/* ── File list ──────────────────────────────────────────────────── */

	.file-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-2) 0;
	}

	.file-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		height: var(--size-row-default);
		padding: 0 var(--space-5);
		background: none;
		border: none;
		color: var(--color-text-primary);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		text-align: left;
		transition: background var(--transition-fast);
	}

	.file-row:hover {
		background: var(--color-bg-hover);
	}

	.file-row.selected {
		background: var(--color-bg-active);
	}

	.file-row:focus-visible {
		outline: var(--focus-ring-width) solid var(--focus-ring-color);
		outline-offset: calc(-1 * var(--focus-ring-width));
	}

	/* ── File path ──────────────────────────────────────────────────── */

	.file-path {
		display: flex;
		align-items: baseline;
		min-width: 0;
		overflow: hidden;
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
	}

	.file-dir {
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex-shrink: 1;
	}

	.file-name {
		color: var(--color-text-primary);
		white-space: nowrap;
		flex-shrink: 0;
	}

	/* ── Status badges ──────────────────────────────────────────────── */

	.file-badges {
		display: flex;
		gap: var(--space-1);
		flex-shrink: 0;
		margin-left: var(--space-3);
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		border-radius: var(--radius-xs, 3px);
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		line-height: 1;
	}

	/* Staged badges: filled background */
	.status-badge.staged.added     { background: var(--color-success); color: white; }
	.status-badge.staged.modified  { background: var(--color-info); color: white; }
	.status-badge.staged.deleted   { background: var(--color-danger); color: white; }
	.status-badge.staged.renamed   { background: var(--color-warning); color: white; }
	.status-badge.staged.typechange { background: var(--color-text-muted); color: white; }

	/* Unstaged badges: outline style */
	.status-badge.unstaged.added     { border: 1px solid var(--color-success); color: var(--color-success); }
	.status-badge.unstaged.modified  { border: 1px solid var(--color-info); color: var(--color-info); }
	.status-badge.unstaged.deleted   { border: 1px solid var(--color-danger); color: var(--color-danger); }
	.status-badge.unstaged.renamed   { border: 1px solid var(--color-warning); color: var(--color-warning); }
	.status-badge.unstaged.typechange { border: 1px solid var(--color-text-muted); color: var(--color-text-muted); }

	/* ── Hunk panel ────────────────────────────────────────────────── */

	.hunk-scroll {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-3);
	}

	.staging-error {
		padding: var(--space-3) var(--space-4);
		margin-bottom: var(--space-3);
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		border: 1px solid var(--color-danger);
		border-radius: var(--radius-sm);
		color: var(--color-danger);
		font-size: var(--text-body-sm-size);
		word-break: break-word;
	}

	.staging-error p {
		margin: 0;
	}

	.hunk-section {
		margin-bottom: var(--space-5);
	}

	.hunk-section-title {
		font-size: var(--text-body-sm-size);
		font-weight: 600;
		color: var(--color-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin: 0 0 var(--space-3) 0;
		padding: 0 var(--space-2);
	}

	.hunk-block {
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		margin-bottom: var(--space-3);
		overflow: hidden;
	}

	.hunk-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-2) var(--space-3);
		background: var(--color-bg-hover);
		border-bottom: 1px solid var(--color-border);
		gap: var(--space-3);
	}

	.hunk-header-text {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.hunk-action-btn {
		flex-shrink: 0;
		padding: var(--space-1) var(--space-3);
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: background var(--transition-fast), color var(--transition-fast);
		min-width: 64px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.hunk-action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.hunk-action-btn:focus-visible {
		outline: var(--focus-ring-width) solid var(--focus-ring-color);
		outline-offset: 1px;
	}

	.stage-btn {
		background: color-mix(in srgb, var(--color-success) 10%, transparent);
		color: var(--color-success);
		border-color: var(--color-success);
	}

	.stage-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-success) 20%, transparent);
	}

	.unstage-btn {
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
		color: var(--color-warning);
		border-color: var(--color-warning);
	}

	.unstage-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-warning) 20%, transparent);
	}

	.btn-spinner {
		width: 12px;
		height: 12px;
		border: 2px solid currentColor;
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
		display: inline-block;
	}

	/* ── Diff lines ────────────────────────────────────────────────── */

	.hunk-lines {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		line-height: 1.5;
	}

	.diff-line {
		display: flex;
		padding: 0 var(--space-3);
		white-space: pre;
	}

	.diff-line.line-add {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
	}

	.diff-line.line-del {
		background: color-mix(in srgb, var(--color-danger) 12%, transparent);
	}

	.diff-line.line-ctx {
		background: transparent;
	}

	.line-origin {
		width: 16px;
		flex-shrink: 0;
		color: var(--color-text-muted);
		user-select: none;
	}

	.line-content {
		flex: 1;
		overflow-x: auto;
	}
</style>
```

### Step 2: Verify

```bash
pnpm check
pnpm build
```

Expected: No type errors, build succeeds.

### Handoff Contract

**Produces:**
- Full hunk stage/unstage UI on `/repo/changes`
- Split layout: file list (left) + hunk panel (right)
- Per-hunk action buttons with pending/disabled state
- Staging error display

**Consumed By:**
- Task 4: Integration verification

---

## Task 4: Downstream handoff verification

**Tier:** worker

**Files:**
- Read-only verification of contracts in:
  - `src/lib/stores/diff.svelte.ts`
  - `src/routes/repo/changes/+page.svelte`
  - `src-tauri/src/git/repository.rs`

### Step 1: Full verification pass

```bash
cd src-tauri && cargo check && cargo test && cd ..
pnpm check
pnpm build
```

All must pass.

### Step 2: Contract verification checklist

Verify these contracts are stable for `bd-20d.3`:

1. **DiffHunkInfo** includes `lines: Vec<DiffLineInfo>` with individual line data — bd-20d.3 can select individual lines from this
2. **diffStore.selectedPath** identifies the currently selected file — bd-20d.3 can attach line-level targeting to this
3. **`/repo/changes` route** remains the single workspace — bd-20d.3 extends it, doesn't replace it
4. **`stage_hunk` / `unstage_hunk` IPC commands** take `hunk_index` — bd-20d.3 can add `line_ranges` parameter alongside
5. **`build_hunk_patch` function** in staging.rs — bd-20d.3 can extend it to filter lines within a hunk

No code changes needed. This is a verification-only task.
