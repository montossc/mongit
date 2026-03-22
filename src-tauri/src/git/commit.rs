//! Commit operations: commit and amend.
//!
//! All operations route CLI errors through `parse_commit_stderr()` for
//! typed `CommitError` frontend consumption.

use std::path::Path;

use serde::Serialize;

use super::cli::GitCli;
use super::error::{parse_commit_stderr, CommitError, GitError};

/// Information about a successfully created commit.
#[derive(Debug, Clone, Serialize)]
pub struct CommitResult {
    /// The full SHA of the new commit.
    pub sha: String,
    /// The first line of the commit message.
    pub summary: String,
}

/// Author identity from git config.
#[derive(Debug, Clone, Serialize)]
pub struct AuthorInfo {
    pub name: String,
    pub email: String,
}

/// Map CommandFailed errors to typed CommitError via stderr parsing.
fn map_cli_error(err: GitError) -> GitError {
    match err {
        GitError::CommandFailed {
            cmd,
            stderr,
            exit_code,
        } => GitError::CommitOp(parse_commit_stderr(&cmd, &stderr, exit_code)),
        other => other,
    }
}

/// Check whether the index has staged changes.
///
/// Uses `git diff --cached --quiet` which exits with code 1 if there are
/// staged changes, and 0 if the index matches HEAD.
fn has_staged_changes(cli: &GitCli) -> Result<bool, GitError> {
    match cli.run_git(&["diff", "--cached", "--quiet"]) {
        Ok(_) => Ok(false), // exit 0 = no changes
        Err(GitError::CommandFailed {
            exit_code: Some(1), ..
        }) => Ok(true), // exit 1 = has changes
        Err(e) => Err(e),   // other errors propagate
    }
}

/// Get the message of the HEAD commit (for amend pre-fill).
///
/// Returns the full commit message of the current HEAD.
pub fn get_head_message(path: &Path, git_executable: &Path) -> Result<String, GitError> {
    let cli = GitCli::new(path, git_executable);
    let output = cli.run_git(&["log", "-1", "--format=%B"])?;
    Ok(output.trim().to_string())
}

/// Create a commit from staged changes.
///
/// If `amend` is true, amends the most recent commit instead of creating a new one.
/// The commit message must be non-empty.
///
/// Returns the SHA and summary of the created commit on success.
/// Pre-commit and commit-msg hooks run normally (handled by `git commit`).
pub fn commit_changes(
    path: &Path,
    git_executable: &Path,
    message: &str,
    amend: bool,
) -> Result<CommitResult, GitError> {
    let cli = GitCli::new(path, git_executable);

    // Validate message is non-empty
    if message.trim().is_empty() {
        return Err(GitError::CommitOp(CommitError::EmptyMessage {
            message: "Commit message cannot be empty".to_string(),
        }));
    }

    // Check for staged changes (skip for amend — git allows amend with no new changes)
    if !amend {
        let has_staged = has_staged_changes(&cli)?;
        if !has_staged {
            return Err(GitError::CommitOp(CommitError::NothingStaged {
                message: "No changes staged for commit".to_string(),
            }));
        }
    }

    // Build commit command
    let mut args = vec!["commit", "-m", message];
    if amend {
        args.push("--amend");
    }

    let output = cli.run_git(&args).map_err(map_cli_error)?;

    // Parse the commit SHA from git output
    // git commit output format: "[branch hash] message"
    // e.g., "[main abc1234] Fix the bug"
    let sha = parse_commit_sha(&output);
    let summary = message.lines().next().unwrap_or(message).to_string();

    Ok(CommitResult { sha, summary })
}

/// Get the configured author identity from git config.
///
/// Reads `user.name` and `user.email` from the repository's git config.
pub fn get_author_config(path: &Path, git_executable: &Path) -> Result<AuthorInfo, GitError> {
    let cli = GitCli::new(path, git_executable);

    let name = cli
        .run_git(&["config", "user.name"])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let email = cli
        .run_git(&["config", "user.email"])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    Ok(AuthorInfo { name, email })
}

/// Parse the commit SHA from `git commit` stdout.
///
/// Git outputs lines like:
/// `[main abc1234] Fix the bug`
/// `[main (root-commit) abc1234] Initial commit`
///
/// We extract the hex SHA from between the last space and `]`.
fn parse_commit_sha(output: &str) -> String {
    // Look for pattern: [branch SHA] or [branch (root-commit) SHA]
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            if let Some(bracket_end) = line.find(']') {
                let inside = &line[1..bracket_end];
                // SHA is the last whitespace-separated token
                if let Some(sha) = inside.split_whitespace().last() {
                    return sha.to_string();
                }
            }
        }
    }
    // Fallback: return empty string (shouldn't happen in normal git output)
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::tests::create_test_repo;
    use std::path::Path as StdPath;

    #[test]
    fn test_parse_commit_sha_normal() {
        let output = "[main abc1234] Fix the bug\n";
        assert_eq!(parse_commit_sha(output), "abc1234");
    }

    #[test]
    fn test_parse_commit_sha_root_commit() {
        let output = "[main (root-commit) def5678] Initial commit\n";
        assert_eq!(parse_commit_sha(output), "def5678");
    }

    #[test]
    fn test_parse_commit_sha_empty() {
        assert_eq!(parse_commit_sha(""), "");
    }

    #[test]
    fn test_commit_empty_message_rejected() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path();

        let result = commit_changes(path, StdPath::new("git"), "", false);
        assert!(result.is_err());
        let err_str: String = result.unwrap_err().into();
        assert!(err_str.contains("EmptyMessage"));
    }

    #[test]
    fn test_commit_nothing_staged_rejected() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path();

        // No changes made, nothing staged
        let result = commit_changes(path, StdPath::new("git"), "test commit", false);
        assert!(result.is_err());
        let err_str: String = result.unwrap_err().into();
        assert!(err_str.contains("NothingStaged"));
    }

    #[test]
    fn test_commit_with_staged_changes() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path();

        // Create and stage a new file using git CLI (not git2, to avoid index issues)
        std::fs::write(path.join("new_file.txt"), "hello world\n").unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["add", "new_file.txt"])
            .output()
            .expect("git add should succeed");

        let result = commit_changes(path, StdPath::new("git"), "Add new file", false);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(!info.sha.is_empty());
        assert_eq!(info.summary, "Add new file");
    }

    #[test]
    fn test_amend_commit() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path();

        // Create and stage a file using git CLI, then commit
        std::fs::write(path.join("amend_file.txt"), "v1\n").unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["add", "amend_file.txt"])
            .output()
            .expect("git add should succeed");

        commit_changes(path, StdPath::new("git"), "Original message", false).unwrap();

        // Now amend (with no new staged changes — amend allows this)
        let result = commit_changes(path, StdPath::new("git"), "Amended message", true);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.summary, "Amended message");
    }

    #[test]
    fn test_get_head_message() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path();

        let msg = get_head_message(path, StdPath::new("git")).unwrap();
        assert_eq!(msg, "Initial commit");
    }

    #[test]
    fn test_get_author_config() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path();

        let author = get_author_config(path, StdPath::new("git")).unwrap();
        assert_eq!(author.name, "Test User");
        assert_eq!(author.email, "test@example.com");
    }
}
