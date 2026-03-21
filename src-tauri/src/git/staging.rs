//! Hunk-level and line-level stage/unstage operations.
//!
//! Builds unified diff patches from `DiffHunkInfo` and applies them
//! via `git apply --cached` for staging and `git apply --cached --reverse`
//! for unstaging. Follows the same module pattern as `branch.rs`.
//!
//! Line-level operations (`build_line_patch`, `stage_lines`, `unstage_lines`)
//! construct patches from a subset of changed lines within a single hunk,
//! converting unselected lines based on patch direction.

use std::path::Path;

use super::cli::GitCli;
use super::error::{parse_stage_stderr, GitError, StageOpError};
use super::repository::{
    DiffFileEntry, DiffFileStatus, DiffHunkInfo, Git2Repository, GitRepository,
};

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
            patch.push_str("new file mode 100644\n");
            patch.push_str("--- /dev/null\n");
            patch.push_str(&format!("+++ b/{path}\n"));
        }
        DiffFileStatus::Deleted => {
            patch.push_str("deleted file mode 100644\n");
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
                // Ensure each diff line ends with \n for valid patch format.
                // If content lacks trailing newline, the subsequent '>' / '<' / '='
                // marker will emit the "no newline" indicator on its own line.
                if !line.content.ends_with('\n') {
                    patch.push('\n');
                }
            }
            '>' | '<' | '=' => {
                // "No newline at end of file" marker — must be on its own line
                patch.push_str("\\ No newline at end of file\n");
            }
            _ => {
                // Skip file headers, hunk headers, binary markers
            }
        }
    }

    patch
}

/// Direction of a line-level staging operation.
///
/// The direction determines how unselected change lines are handled:
/// - **Stage** (forward apply): unselected `-` → context, unselected `+` → omit
/// - **Unstage** (reverse apply): unselected `-` → omit, unselected `+` → context
#[derive(Clone, Copy, PartialEq, Eq)]
enum PatchDirection {
    Stage,
    Unstage,
}

/// Build a unified diff patch from selected lines within a single hunk.
///
/// Selected change lines (`+` or `-`) are always included as-is.
/// Unselected lines are handled based on `direction`:
///
/// **Stage** (forward-apply to index):
/// - Unselected `-` → context (line remains in both workdir and index)
/// - Unselected `+` → omit (line stays only in workdir)
///
/// **Unstage** (reverse-apply from index):
/// - Unselected `-` → omit (deletion stays staged; line not in index)
/// - Unselected `+` → context (addition stays staged; line IS in index)
///
/// Context lines (origin `' '`) are always included.
///
/// Returns an error if no change lines are selected or if indices are
/// out of range.
fn build_line_patch(
    path: &str,
    hunk: &DiffHunkInfo,
    selected_indices: &[usize],
    status: &DiffFileStatus,
    direction: PatchDirection,
) -> Result<String, StageOpError> {
    // Validate indices
    let selection: std::collections::HashSet<usize> = selected_indices.iter().copied().collect();
    for &idx in &selection {
        if idx >= hunk.lines.len() {
            return Err(StageOpError::InvalidLineSelection {
                reason: format!(
                    "line index {} out of range (hunk has {} lines)",
                    idx,
                    hunk.lines.len()
                ),
                message: format!("line index {} out of range", idx),
            });
        }
    }

    // Check that at least one change line is selected
    let has_change = selection.iter().any(|&idx| {
        let origin = hunk.lines[idx].origin;
        origin == '+' || origin == '-'
    });
    if !has_change {
        return Err(StageOpError::InvalidLineSelection {
            reason: "no change lines selected (only context lines)".to_string(),
            message: "selection must include at least one added or removed line".to_string(),
        });
    }

    let mut patch = String::new();

    // File header (same as build_hunk_patch)
    patch.push_str(&format!("diff --git a/{path} b/{path}\n"));
    match status {
        DiffFileStatus::Added => {
            patch.push_str("new file mode 100644\n");
            patch.push_str("--- /dev/null\n");
            patch.push_str(&format!("+++ b/{path}\n"));
        }
        DiffFileStatus::Deleted => {
            patch.push_str("deleted file mode 100644\n");
            patch.push_str(&format!("--- a/{path}\n"));
            patch.push_str("+++ /dev/null\n");
        }
        _ => {
            patch.push_str(&format!("--- a/{path}\n"));
            patch.push_str(&format!("+++ b/{path}\n"));
        }
    }

    // Build filtered lines and calculate counts
    let mut patch_lines: Vec<(char, &str)> = Vec::new();
    let mut old_lines: u32 = 0;
    let mut new_lines: u32 = 0;

    for (idx, line) in hunk.lines.iter().enumerate() {
        match line.origin {
            ' ' => {
                // Context: always include
                patch_lines.push((' ', &line.content));
                old_lines += 1;
                new_lines += 1;
            }
            '-' => {
                if selection.contains(&idx) {
                    // Selected removal: include as '-'
                    patch_lines.push(('-', &line.content));
                    old_lines += 1;
                } else if direction == PatchDirection::Stage {
                    // Unselected removal (staging): convert to context
                    // (line remains in both workdir and index)
                    patch_lines.push((' ', &line.content));
                    old_lines += 1;
                    new_lines += 1;
                }
                // Unselected removal (unstaging): omit entirely
                // (deletion stays staged; line not in index)
            }
            '+' => {
                if selection.contains(&idx) {
                    // Selected addition: include as '+'
                    patch_lines.push(('+', &line.content));
                    new_lines += 1;
                } else if direction == PatchDirection::Unstage {
                    // Unselected addition (unstaging): convert to context
                    // (addition stays staged; line IS in index)
                    patch_lines.push((' ', &line.content));
                    old_lines += 1;
                    new_lines += 1;
                }
                // Unselected addition (staging): omit entirely
                // (line stays only in workdir)
            }
            '>' | '<' | '=' => {
                // No-newline marker: include if preceding line was included
                if idx > 0 {
                    let prev = &hunk.lines[idx - 1];
                    let prev_included = match prev.origin {
                        ' ' => true,
                        '-' => {
                            // In stage mode: always included (as '-' or context)
                            // In unstage mode: only if selected
                            direction == PatchDirection::Stage
                                || selection.contains(&(idx - 1))
                        }
                        '+' => {
                            // In stage mode: only if selected
                            // In unstage mode: always included (as '+' or context)
                            direction == PatchDirection::Unstage
                                || selection.contains(&(idx - 1))
                        }
                        _ => false,
                    };
                    if prev_included {
                        patch_lines.push(('\\', ""));
                    }
                }
            }
            _ => {
                // Skip file headers, hunk headers, binary markers
            }
        }
    }

    // Hunk header with recalculated counts
    patch.push_str(&format!(
        "@@ -{},{} +{},{} @@\n",
        hunk.old_start, old_lines, hunk.new_start, new_lines,
    ));

    // Emit lines
    for (origin, content) in &patch_lines {
        if *origin == '\\' {
            patch.push_str("\\ No newline at end of file\n");
        } else {
            patch.push(*origin);
            patch.push_str(content);
            if !content.ends_with('\n') {
                patch.push('\n');
            }
        }
    }

    Ok(patch)
}
/// Find a file entry in a diff result by path.
fn find_file_in_diff<'a>(
    diff: &'a [DiffFileEntry],
    file_path: &str,
    context: &str,
) -> Result<&'a DiffFileEntry, GitError> {
    diff.iter().find(|f| f.path == file_path).ok_or_else(|| {
        GitError::StageOp(StageOpError::FileNotInDiff {
            path: file_path.to_string(),
            message: format!("file '{}' has no {} changes", file_path, context),
        })
    })
}

/// Validate and retrieve a hunk from a file entry by index.
fn get_hunk_at_index(
    file_entry: &DiffFileEntry,
    hunk_index: usize,
) -> Result<&DiffHunkInfo, GitError> {
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
    cli.run_git_with_stdin(&["apply", "--cached", "--unidiff-zero"], patch.as_bytes())
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

/// Stage selected lines from a single hunk into the index.
///
/// Reads the unstaged diff (index→workdir), extracts the specified hunk,
/// builds a line-level patch from the selected line indices, and applies
/// it via `git apply --cached --unidiff-zero`.
pub fn stage_lines(
    path: &Path,
    git_executable: &Path,
    file_path: &str,
    hunk_index: usize,
    line_indices: &[usize],
) -> Result<(), GitError> {
    // 1. Read unstaged diff (index → workdir)
    let repo = Git2Repository::open(path);
    let diff = repo.diff_workdir()?;

    // 2. Find file and hunk
    let file_entry = find_file_in_diff(&diff, file_path, "unstaged")?;
    let hunk = get_hunk_at_index(file_entry, hunk_index)?;

    // 3. Build line-level patch
    let patch =
        build_line_patch(file_path, hunk, line_indices, &file_entry.status, PatchDirection::Stage)
            .map_err(GitError::StageOp)?;

    // 4. Apply via git apply --cached
    let cli = GitCli::new(path, git_executable);
    cli.run_git_with_stdin(&["apply", "--cached", "--unidiff-zero"], patch.as_bytes())
        .map_err(map_cli_error)?;

    Ok(())
}

/// Unstage selected lines from the index back to the working tree.
///
/// Reads the staged diff (HEAD→index), extracts the specified hunk,
/// builds a line-level patch from the selected line indices, and
/// reverse-applies it via `git apply --cached --reverse --unidiff-zero`.
pub fn unstage_lines(
    path: &Path,
    git_executable: &Path,
    file_path: &str,
    hunk_index: usize,
    line_indices: &[usize],
) -> Result<(), GitError> {
    // 1. Read staged diff (HEAD → index)
    let repo = Git2Repository::open(path);
    let diff = repo.diff_index()?;

    // 2. Find file and hunk
    let file_entry = find_file_in_diff(&diff, file_path, "staged")?;
    let hunk = get_hunk_at_index(file_entry, hunk_index)?;

    // 3. Build line-level patch
    let patch =
        build_line_patch(file_path, hunk, line_indices, &file_entry.status, PatchDirection::Unstage)
            .map_err(GitError::StageOp)?;

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
        index.add_path(std::path::Path::new("initial.txt")).unwrap();
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
        drop(tree);
        drop(head);

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
        assert_eq!(staged_file.hunks.len(), 1, "only one hunk should be staged");
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
        assert!(
            staged.is_empty(),
            "clean repo should have no staged changes"
        );
    }

    #[test]
    fn test_diff_index_shows_staged_changes() {
        let (dir, _repo) = create_test_repo();
        let repo_path = dir.path();

        // Stage a new file
        std::fs::write(repo_path.join("staged.txt"), "staged content\n").unwrap();
        let g2 = git2::Repository::open(repo_path).unwrap();
        let mut index = g2.index().unwrap();
        index.add_path(std::path::Path::new("staged.txt")).unwrap();
        index.write().unwrap();

        let repo = Git2Repository::open(repo_path);
        let staged = repo.diff_index().unwrap();
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].path, "staged.txt");
        assert!(!staged[0].hunks.is_empty());
    }

    // ── Line-level patch construction tests ─────────────────────────────

    /// Helper: build a standard hunk for line-level testing.
    fn test_hunk() -> DiffHunkInfo {
        use super::super::repository::DiffLineInfo;
        DiffHunkInfo {
            old_start: 1,
            old_lines: 4,
            new_start: 1,
            new_lines: 5,
            header: String::new(),
            lines: vec![
                DiffLineInfo {
                    origin: ' ',
                    content: "context A\n".to_string(),
                    old_lineno: Some(1),
                    new_lineno: Some(1),
                },
                DiffLineInfo {
                    origin: '-',
                    content: "old line 1\n".to_string(),
                    old_lineno: Some(2),
                    new_lineno: None,
                },
                DiffLineInfo {
                    origin: '-',
                    content: "old line 2\n".to_string(),
                    old_lineno: Some(3),
                    new_lineno: None,
                },
                DiffLineInfo {
                    origin: '+',
                    content: "new line 1\n".to_string(),
                    old_lineno: None,
                    new_lineno: Some(2),
                },
                DiffLineInfo {
                    origin: '+',
                    content: "new line 2\n".to_string(),
                    old_lineno: None,
                    new_lineno: Some(3),
                },
                DiffLineInfo {
                    origin: '+',
                    content: "new line 3\n".to_string(),
                    old_lineno: None,
                    new_lineno: Some(4),
                },
                DiffLineInfo {
                    origin: ' ',
                    content: "context B\n".to_string(),
                    old_lineno: Some(4),
                    new_lineno: Some(5),
                },
            ],
        }
    }

    #[test]
    fn test_build_line_patch_single_add() {
        let hunk = test_hunk();
        // Select only index 3 (first '+' line)
        let patch =
            build_line_patch("test.txt", &hunk, &[3], &DiffFileStatus::Modified, PatchDirection::Stage).unwrap();

        // Should include the selected '+' line
        assert!(patch.contains("+new line 1\n"));
        // Should NOT include unselected '+' lines
        assert!(!patch.contains("+new line 2"));
        assert!(!patch.contains("+new line 3"));
        // Unselected '-' lines should become context
        assert!(patch.contains(" old line 1\n")); // was '-', now context
        assert!(patch.contains(" old line 2\n")); // was '-', now context
        // Context lines always included
        assert!(patch.contains(" context A\n"));
        assert!(patch.contains(" context B\n"));
        // Hunk header: old_lines = 4 (2 ctx + 2 unselected-del-as-ctx), new_lines = 5 (4 ctx + 1 add)
        assert!(patch.contains("@@ -1,4 +1,5 @@"));
    }

    #[test]
    fn test_build_line_patch_single_del() {
        let hunk = test_hunk();
        // Select only index 1 (first '-' line)
        let patch =
            build_line_patch("test.txt", &hunk, &[1], &DiffFileStatus::Modified, PatchDirection::Stage).unwrap();

        // Should include the selected '-' line
        assert!(patch.contains("-old line 1\n"));
        // Unselected '-' line becomes context
        assert!(patch.contains(" old line 2\n"));
        // No '+' lines selected, so none should appear
        assert!(!patch.contains("+new line"));
        // old_lines = 4 (2 ctx + 1 del + 1 unselected-del-as-ctx), new_lines = 3 (3 ctx)
        assert!(patch.contains("@@ -1,4 +1,3 @@"));
    }

    #[test]
    fn test_build_line_patch_mixed_selection() {
        let hunk = test_hunk();
        // Select index 1 ('-' old line 1) and index 3 ('+' new line 1)
        let patch =
            build_line_patch("test.txt", &hunk, &[1, 3], &DiffFileStatus::Modified, PatchDirection::Stage).unwrap();

        assert!(patch.contains("-old line 1\n"));
        assert!(patch.contains("+new line 1\n"));
        // Unselected '-' old line 2 → context
        assert!(patch.contains(" old line 2\n"));
        // Unselected '+' lines → omitted
        assert!(!patch.contains("+new line 2"));
        assert!(!patch.contains("+new line 3"));
        // old_lines = 4, new_lines = 4 (3 ctx + 1 add)
        assert!(patch.contains("@@ -1,4 +1,4 @@"));
    }

    #[test]
    fn test_build_line_patch_all_lines_equals_whole_hunk() {
        let hunk = test_hunk();
        // Select all change lines: indices 1,2,3,4,5
        let patch =
            build_line_patch("test.txt", &hunk, &[1, 2, 3, 4, 5], &DiffFileStatus::Modified, PatchDirection::Stage)
                .unwrap();

        // Should match the original hunk's counts
        assert!(patch.contains("@@ -1,4 +1,5 @@"));
        assert!(patch.contains("-old line 1\n"));
        assert!(patch.contains("-old line 2\n"));
        assert!(patch.contains("+new line 1\n"));
        assert!(patch.contains("+new line 2\n"));
        assert!(patch.contains("+new line 3\n"));
    }

    #[test]
    fn test_build_line_patch_no_change_lines_error() {
        let hunk = test_hunk();
        // Select only context line indices (0 and 6)
        let err = build_line_patch("test.txt", &hunk, &[0, 6], &DiffFileStatus::Modified, PatchDirection::Stage)
            .unwrap_err();
        match err {
            StageOpError::InvalidLineSelection { reason, .. } => {
                assert!(reason.contains("no change lines"));
            }
            other => panic!("expected InvalidLineSelection, got: {:?}", other),
        }
    }

    #[test]
    fn test_build_line_patch_empty_selection_error() {
        let hunk = test_hunk();
        let err = build_line_patch("test.txt", &hunk, &[], &DiffFileStatus::Modified, PatchDirection::Stage).unwrap_err();
        match err {
            StageOpError::InvalidLineSelection { .. } => {}
            other => panic!("expected InvalidLineSelection, got: {:?}", other),
        }
    }

    #[test]
    fn test_build_line_patch_out_of_range_error() {
        let hunk = test_hunk();
        let err =
            build_line_patch("test.txt", &hunk, &[99], &DiffFileStatus::Modified, PatchDirection::Stage).unwrap_err();
        match err {
            StageOpError::InvalidLineSelection { reason, .. } => {
                assert!(reason.contains("out of range"));
            }
            other => panic!("expected InvalidLineSelection, got: {:?}", other),
        }
    }

    #[test]
    fn test_build_line_patch_add_only_hunk() {
        use super::super::repository::DiffLineInfo;
        let hunk = DiffHunkInfo {
            old_start: 0,
            old_lines: 0,
            new_start: 1,
            new_lines: 3,
            header: String::new(),
            lines: vec![
                DiffLineInfo {
                    origin: '+',
                    content: "line A\n".to_string(),
                    old_lineno: None,
                    new_lineno: Some(1),
                },
                DiffLineInfo {
                    origin: '+',
                    content: "line B\n".to_string(),
                    old_lineno: None,
                    new_lineno: Some(2),
                },
                DiffLineInfo {
                    origin: '+',
                    content: "line C\n".to_string(),
                    old_lineno: None,
                    new_lineno: Some(3),
                },
            ],
        };

        // Select only first line
        let patch =
            build_line_patch("new.txt", &hunk, &[0], &DiffFileStatus::Added, PatchDirection::Stage).unwrap();
        assert!(patch.contains("+line A\n"));
        assert!(!patch.contains("+line B"));
        assert!(!patch.contains("+line C"));
        assert!(patch.contains("@@ -0,0 +1,1 @@"));
        assert!(patch.contains("--- /dev/null"));
    }

    #[test]
    fn test_build_line_patch_del_only_hunk() {
        use super::super::repository::DiffLineInfo;
        let hunk = DiffHunkInfo {
            old_start: 1,
            old_lines: 2,
            new_start: 1,
            new_lines: 0,
            header: String::new(),
            lines: vec![
                DiffLineInfo {
                    origin: '-',
                    content: "dead A\n".to_string(),
                    old_lineno: Some(1),
                    new_lineno: None,
                },
                DiffLineInfo {
                    origin: '-',
                    content: "dead B\n".to_string(),
                    old_lineno: Some(2),
                    new_lineno: None,
                },
            ],
        };

        // Select only first deletion
        let patch =
            build_line_patch("gone.txt", &hunk, &[0], &DiffFileStatus::Modified, PatchDirection::Stage).unwrap();
        assert!(patch.contains("-dead A\n"));
        // Unselected '-' → context
        assert!(patch.contains(" dead B\n"));
        // old_lines = 2, new_lines = 1 (the context line)
        assert!(patch.contains("@@ -1,2 +1,1 @@"));
    }

    // ── Line-level integration tests (stage_lines / unstage_lines) ──────

    #[test]
    fn test_stage_lines_partial_add() {
        let (dir, _repo) = create_repo_with_content();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Add two new lines at the top (creates a hunk with additions)
        std::fs::write(
            repo_path.join("initial.txt"),
            "NEW LINE A\nNEW LINE B\nline 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n",
        )
        .unwrap();

        // Read diff to find hunk structure
        let repo = Git2Repository::open(repo_path);
        let diff = repo.diff_workdir().unwrap();
        let file = diff.iter().find(|f| f.path == "initial.txt").unwrap();
        assert!(!file.hunks.is_empty(), "should have at least one hunk");

        // Find the '+' line indices in hunk 0
        let add_indices: Vec<usize> = file.hunks[0]
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.origin == '+' && l.content.contains("NEW LINE A"))
            .map(|(i, _)| i)
            .collect();
        assert!(!add_indices.is_empty(), "should find NEW LINE A in hunk");

        // Stage only NEW LINE A
        stage_lines(repo_path, git, "initial.txt", 0, &add_indices).unwrap();

        // Verify: staged should show NEW LINE A
        let repo2 = Git2Repository::open(repo_path);
        let staged = repo2.diff_index().unwrap();
        let staged_file = staged.iter().find(|f| f.path == "initial.txt").unwrap();
        let staged_content: String = staged_file.hunks[0]
            .lines
            .iter()
            .filter(|l| l.origin == '+')
            .map(|l| l.content.clone())
            .collect();
        assert!(
            staged_content.contains("NEW LINE A"),
            "staged should contain NEW LINE A"
        );

        // Unstaged should still show NEW LINE B
        let unstaged = repo2.diff_workdir().unwrap();
        let remaining = unstaged.iter().find(|f| f.path == "initial.txt");
        assert!(
            remaining.is_some(),
            "file should still have unstaged changes"
        );
        let unstaged_content: String = remaining
            .unwrap()
            .hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.origin == '+')
            .map(|l| l.content.clone())
            .collect();
        assert!(
            unstaged_content.contains("NEW LINE B"),
            "unstaged should still contain NEW LINE B"
        );
    }

    #[test]
    fn test_stage_lines_invalid_indices() {
        let (dir, _repo) = create_repo_with_content();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Create a change
        std::fs::write(repo_path.join("initial.txt"), "changed\n").unwrap();

        let err = stage_lines(repo_path, git, "initial.txt", 0, &[99]).unwrap_err();
        match err {
            GitError::StageOp(StageOpError::InvalidLineSelection { .. }) => {}
            other => panic!("expected InvalidLineSelection, got: {:?}", other),
        }
    }

    #[test]
    fn test_stage_lines_only_context_error() {
        let (dir, _repo) = create_repo_with_content();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Modify first line (creates hunk with context + change)
        std::fs::write(
            repo_path.join("initial.txt"),
            "MODIFIED\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n",
        )
        .unwrap();

        // Find context line indices
        let repo = Git2Repository::open(repo_path);
        let diff = repo.diff_workdir().unwrap();
        let file = diff.iter().find(|f| f.path == "initial.txt").unwrap();
        let ctx_indices: Vec<usize> = file.hunks[0]
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.origin == ' ')
            .map(|(i, _)| i)
            .collect();

        if !ctx_indices.is_empty() {
            let err =
                stage_lines(repo_path, git, "initial.txt", 0, &ctx_indices).unwrap_err();
            match err {
                GitError::StageOp(StageOpError::InvalidLineSelection { .. }) => {}
                other => panic!("expected InvalidLineSelection, got: {:?}", other),
            }
        }
    }

    #[test]
    fn test_unstage_lines_partial() {
        let (dir, _repo) = create_repo_with_content();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Stage a modification via git add
        std::fs::write(
            repo_path.join("initial.txt"),
            "MODIFIED A\nMODIFIED B\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n",
        )
        .unwrap();

        let cli = GitCli::new(repo_path, git);
        cli.run_git(&["add", "initial.txt"]).unwrap();

        // Read staged diff
        let repo = Git2Repository::open(repo_path);
        let staged = repo.diff_index().unwrap();
        let file = staged.iter().find(|f| f.path == "initial.txt").unwrap();
        assert!(!file.hunks.is_empty());

        // Find first '+' line in staged hunk
        let first_add_idx = file.hunks[0]
            .lines
            .iter()
            .position(|l| l.origin == '+')
            .expect("should have a + line");

        // Unstage only the first '+' line
        unstage_lines(repo_path, git, "initial.txt", 0, &[first_add_idx]).unwrap();

        // Verify: still has some staged changes
        let repo2 = Git2Repository::open(repo_path);
        let staged2 = repo2.diff_index().unwrap();
        assert!(
            staged2.iter().any(|f| f.path == "initial.txt"),
            "should still have some staged changes"
        );
    }

    #[test]
    fn test_line_patch_error_serializes_as_json() {
        let err = StageOpError::InvalidLineSelection {
            reason: "test reason".to_string(),
            message: "test message".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"InvalidLineSelection\""));
        assert!(json.contains("\"reason\":\"test reason\""));
    }
}
