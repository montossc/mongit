//! Hunk-level stage/unstage operations.
//!
//! Builds unified diff patches from `DiffHunkInfo` and applies them
//! via `git apply --cached` for staging and `git apply --cached --reverse`
//! for unstaging. Follows the same module pattern as `branch.rs`.

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
}
