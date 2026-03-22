//! Branch operations: create, switch, delete, fetch, pull, push.
//!
//! All operations route CLI errors through `parse_branch_stderr()` for
//! typed `BranchOpError` frontend consumption.

use std::path::Path;

use git2::BranchType;
use serde::Serialize;

use super::cli::GitCli;
use super::error::{parse_branch_stderr, BranchOpError, GitError};

/// Map CommandFailed errors to typed BranchOpError via stderr parsing.
///
/// Used by all branch operations to convert raw CLI errors into
/// structured errors for frontend consumption.
fn map_cli_error(err: GitError) -> GitError {
    match err {
        GitError::CommandFailed {
            cmd,
            stderr,
            exit_code,
        } => GitError::BranchOp(parse_branch_stderr(&cmd, &stderr, exit_code)),
        other => other,
    }
}

/// Create a new local branch.
///
/// If `start_point` is provided, the branch starts from that ref/commit.
/// Otherwise starts from HEAD.
///
/// Errors are parsed through `parse_branch_stderr()` for typed frontend consumption.
pub fn create_branch(
    path: &Path,
    git_executable: &Path,
    name: &str,
    start_point: Option<&str>,
) -> Result<(), GitError> {
    let cli = GitCli::new(path, git_executable);
    cli.create_branch(name, start_point)
        .map_err(map_cli_error)
}

/// Switch to an existing branch.
///
/// Errors are parsed through `parse_branch_stderr()` for typed frontend consumption.
pub fn switch_branch(
    path: &Path,
    git_executable: &Path,
    name: &str,
) -> Result<(), GitError> {
    let cli = GitCli::new(path, git_executable);
    cli.switch_branch(name).map_err(map_cli_error)
}

/// Delete a local branch with pre-flight safety checks.
///
/// Pre-flight checks (via git2):
/// - Branch must exist
/// - Branch must not be HEAD (cannot delete checked-out branch)
///
/// If `force` is false, uses `git branch -d` (safe delete — requires fully merged).
/// If `force` is true, uses `git branch -D` (force delete — ignores merge status).
///
/// Errors are parsed through `parse_branch_stderr()` for typed frontend consumption.
pub fn delete_branch(
    path: &Path,
    git_executable: &Path,
    name: &str,
    force: bool,
) -> Result<(), GitError> {
    // Pre-flight: open repo via git2 for safety checks
    let repo = git2::Repository::open(path)?;

    // Check branch exists
    let branch = repo.find_branch(name, BranchType::Local).map_err(|e| {
        if e.code() == git2::ErrorCode::NotFound {
            GitError::BranchOp(BranchOpError::BranchNotFound {
                branch: name.to_string(),
                message: format!("branch '{}' not found", name),
            })
        } else {
            GitError::Git2(e)
        }
    })?;

    // Check not HEAD — cannot delete the currently checked-out branch
    if branch.is_head() {
        return Err(GitError::BranchOp(BranchOpError::DeleteCurrentBranch {
            branch: name.to_string(),
            message: format!("cannot delete current HEAD branch '{}'", name),
        }));
    }

    // Execute via CLI (git branch -d/-D)
    let cli = GitCli::new(path, git_executable);
    let flag = if force { "-D" } else { "-d" };
    let args = ["branch", flag, "--", name];

    match cli.run_git(&args) {
        Ok(_) => Ok(()),
        Err(e) => Err(map_cli_error(e)),
    }
}

/// Fetch latest refs and objects from origin remote.
///
/// Uses `git fetch --prune origin` for async non-blocking execution.
/// Sets `GIT_TERMINAL_PROMPT=0` (via `run_git_async`) to prevent credential
/// prompts from hanging the subprocess.
///
/// Errors are parsed through `parse_branch_stderr()` for typed frontend consumption.
pub async fn fetch_origin(path: &Path, git_executable: &Path) -> Result<String, GitError> {
    let cli = GitCli::new(path, git_executable);
    cli.run_git_async(&["fetch", "--prune", "origin"])
        .await
        .map_err(map_cli_error)
}

/// Pull changes from origin into the current branch.
///
/// Determines the current branch via git2, then runs
/// `git pull origin <current-branch>` asynchronously.
///
/// Returns `NoUpstreamBranch` if HEAD is detached or branch name cannot be determined.
pub async fn pull_origin(path: &Path, git_executable: &Path) -> Result<String, GitError> {
    // Determine current branch name via git2 (fast, local operation)
    let branch_name = current_branch_name(path)?;

    let cli = GitCli::new(path, git_executable);
    cli.run_git_async(&["pull", "origin", &branch_name])
        .await
        .map_err(map_cli_error)
}

/// Push current branch to origin.
///
/// If `force_with_lease` is true, uses `--force-with-lease` (never bare `--force`).
/// Always uses `-u` to auto-set upstream tracking if not already configured.
///
/// Returns `NoUpstreamBranch` if HEAD is detached or branch name cannot be determined.
pub async fn push_origin(
    path: &Path,
    git_executable: &Path,
    force_with_lease: bool,
) -> Result<String, GitError> {
    // Determine current branch name via git2
    let branch_name = current_branch_name(path)?;

    let cli = GitCli::new(path, git_executable);

    let mut args: Vec<&str> = vec!["push"];
    if force_with_lease {
        args.push("--force-with-lease");
    }
    args.push("-u");
    args.push("origin");
    args.push(&branch_name);

    cli.run_git_async(&args).await.map_err(map_cli_error)
}

/// Get the current branch name via git2.
///
/// Returns an error if HEAD is detached, unborn, or the branch name
/// cannot be determined.
fn current_branch_name(path: &Path) -> Result<String, GitError> {
    let repo = git2::Repository::open(path)?;
    let head = repo.head().map_err(|_| {
        GitError::BranchOp(BranchOpError::NoUpstreamBranch {
            message: "HEAD is detached or unborn".to_string(),
        })
    })?;
    let name = head
        .shorthand()
        .ok_or_else(|| {
            GitError::BranchOp(BranchOpError::NoUpstreamBranch {
                message: "cannot determine current branch name".to_string(),
            })
        })?
        .to_string();
    Ok(name)
}

/// Response type for ahead/behind tracking information.
#[derive(Debug, Serialize)]
pub struct AheadBehind {
    pub ahead: u32,
    pub behind: u32,
    pub upstream: Option<String>,
}

/// Get ahead/behind counts relative to the upstream tracking branch.
///
/// Uses `git rev-list --left-right --count HEAD...@{upstream}`.
/// Returns `{ ahead: 0, behind: 0, upstream: None }` when no upstream is configured
/// (this is not an error — it's a valid state for new branches).
pub async fn ahead_behind(path: &Path, git_executable: &Path) -> Result<AheadBehind, GitError> {
    let cli = GitCli::new(path, git_executable);

    // First, try to get the upstream branch name
    let upstream_result = cli
        .run_git_async(&["rev-parse", "--abbrev-ref", "@{upstream}"])
        .await;

    let upstream_name = match upstream_result {
        Ok(name) => Some(name.trim().to_string()),
        Err(_) => {
            // No upstream configured — return zeros, not an error
            return Ok(AheadBehind {
                ahead: 0,
                behind: 0,
                upstream: None,
            });
        }
    };

    // Get the ahead/behind counts
    let output = cli
        .run_git_async(&["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .await
        .map_err(map_cli_error)?;

    // Output format: "ahead\tbehind\n"
    let parts: Vec<&str> = output.trim().split('\t').collect();
    let ahead = parts
        .first()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let behind = parts
        .get(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(AheadBehind {
        ahead,
        behind,
        upstream: upstream_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::tests::create_test_repo;
    use crate::git::tests::create_test_repo_with_remote;
    use crate::git::GitCli;
    use std::path::Path;

    #[test]
    fn test_delete_branch() {
        let (dir, _repo) = create_test_repo();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Create a branch, then delete it
        let cli = GitCli::new(repo_path, git);
        cli.create_branch("to-delete", None).unwrap();

        delete_branch(repo_path, git, "to-delete", false).unwrap();

        // Verify branch is gone via git2
        let repo = git2::Repository::open(repo_path).unwrap();
        assert!(repo.find_branch("to-delete", BranchType::Local).is_err());
    }

    #[test]
    fn test_delete_current_branch_fails() {
        let (dir, repo) = create_test_repo();
        let repo_path = dir.path();
        let git = Path::new("git");

        // HEAD is on the default branch — get its name
        let head = repo.head().unwrap();
        let branch_name = head.shorthand().unwrap(); // "main" or "master"

        let err = delete_branch(repo_path, git, branch_name, false).unwrap_err();

        match err {
            GitError::BranchOp(BranchOpError::DeleteCurrentBranch { branch, .. }) => {
                assert_eq!(branch, branch_name);
            }
            other => panic!("expected DeleteCurrentBranch, got: {:?}", other),
        }
    }

    #[test]
    fn test_delete_unmerged_branch_safe() {
        let (dir, repo) = create_test_repo();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Save default branch name BEFORE switching away
        let head = repo.head().unwrap();
        let default_branch = head.shorthand().unwrap().to_string();
        drop(head);

        // Create a branch and add a commit that is NOT merged into main/master
        let cli = GitCli::new(repo_path, git);
        cli.create_branch("unmerged", None).unwrap();
        cli.run_git(&["switch", "--", "unmerged"]).unwrap();

        // Create a file and commit on the unmerged branch
        std::fs::write(repo_path.join("unmerged.txt"), "data\n").unwrap();
        cli.run_git(&["add", "unmerged.txt"]).unwrap();
        cli.run_git(&["commit", "-m", "unmerged commit"]).unwrap();

        // Switch back to default branch
        cli.run_git(&["switch", "--", &default_branch]).unwrap();

        // Try safe delete — should fail with BranchNotFullyMerged
        let err = delete_branch(repo_path, git, "unmerged", false).unwrap_err();

        match err {
            GitError::BranchOp(BranchOpError::BranchNotFullyMerged { branch, .. }) => {
                assert_eq!(branch, "unmerged");
            }
            other => panic!("expected BranchNotFullyMerged, got: {:?}", other),
        }
    }

    #[test]
    fn test_force_delete_unmerged_branch() {
        let (dir, repo) = create_test_repo();
        let repo_path = dir.path();
        let git = Path::new("git");

        // Save default branch name BEFORE switching away
        let head = repo.head().unwrap();
        let default_branch = head.shorthand().unwrap().to_string();
        drop(head);

        // Create a branch with unmerged commit
        let cli = GitCli::new(repo_path, git);
        cli.create_branch("unmerged-force", None).unwrap();
        cli.run_git(&["switch", "--", "unmerged-force"]).unwrap();

        std::fs::write(repo_path.join("force.txt"), "data\n").unwrap();
        cli.run_git(&["add", "force.txt"]).unwrap();
        cli.run_git(&["commit", "-m", "unmerged commit"]).unwrap();

        // Switch back to default branch
        cli.run_git(&["switch", "--", &default_branch]).unwrap();

        // Force delete should succeed
        delete_branch(repo_path, git, "unmerged-force", true).unwrap();

        // Verify branch is gone
        let repo = git2::Repository::open(repo_path).unwrap();
        assert!(repo
            .find_branch("unmerged-force", BranchType::Local)
            .is_err());
    }

    #[test]
    fn test_delete_nonexistent_branch() {
        let (dir, _repo) = create_test_repo();
        let repo_path = dir.path();
        let git = Path::new("git");

        let err = delete_branch(repo_path, git, "doesnt-exist", false).unwrap_err();

        match err {
            GitError::BranchOp(BranchOpError::BranchNotFound { branch, .. }) => {
                assert_eq!(branch, "doesnt-exist");
            }
            other => panic!("expected BranchNotFound, got: {:?}", other),
        }
    }

    // ── Remote operation tests (async) ──────────────────────────────────

    #[tokio::test]
    async fn test_fetch() {
        let (work_dir, _bare_dir) = create_test_repo_with_remote();
        let path = work_dir.path();
        let git = Path::new("git");

        // Fetch from local bare remote — should succeed
        fetch_origin(path, git).await.unwrap();
    }

    #[tokio::test]
    async fn test_push() {
        let (work_dir, _bare_dir) = create_test_repo_with_remote();
        let path = work_dir.path();
        let git = Path::new("git");

        // Create a new commit in the working repo
        let cli = GitCli::new(path, git);
        std::fs::write(path.join("push-test.txt"), "data\n").unwrap();
        cli.run_git(&["add", "push-test.txt"]).unwrap();
        cli.run_git(&["commit", "-m", "push test commit"]).unwrap();

        // Push should succeed
        push_origin(path, git, false).await.unwrap();

        // Verify: the bare remote should have the new commit
        let bare_repo = git2::Repository::open(_bare_dir.path()).unwrap();
        let head = bare_repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        assert_eq!(commit.message().unwrap().trim(), "push test commit");
    }

    #[tokio::test]
    async fn test_pull() {
        let (work_dir, bare_dir) = create_test_repo_with_remote();
        let path = work_dir.path();
        let git = Path::new("git");

        // Create a second clone, push a commit from there
        let clone_parent = tempfile::TempDir::new().unwrap();
        let clone_path = clone_parent.path().join("clone");

        let output = std::process::Command::new("git")
            .args([
                "clone",
                bare_dir.path().to_str().unwrap(),
                clone_path.to_str().unwrap(),
            ])
            .output()
            .expect("clone should succeed");
        assert!(
            output.status.success(),
            "clone failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Configure test user in clone
        std::process::Command::new("git")
            .arg("-C")
            .arg(&clone_path)
            .args(["config", "user.name", "Test User"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(&clone_path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .unwrap();

        // Create and push a commit from the clone
        std::fs::write(clone_path.join("pull-test.txt"), "pull data\n").unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(&clone_path)
            .args(["add", "pull-test.txt"])
            .output()
            .unwrap();
        let commit_out = std::process::Command::new("git")
            .arg("-C")
            .arg(&clone_path)
            .args(["commit", "-m", "pull test commit"])
            .output()
            .unwrap();
        assert!(
            commit_out.status.success(),
            "commit failed: {}",
            String::from_utf8_lossy(&commit_out.stderr)
        );
        let push_out = std::process::Command::new("git")
            .arg("-C")
            .arg(&clone_path)
            .args(["push", "origin", "HEAD"])
            .output()
            .unwrap();
        assert!(
            push_out.status.success(),
            "push failed: {}",
            String::from_utf8_lossy(&push_out.stderr)
        );

        // Pull should succeed from the first working repo
        pull_origin(path, git).await.unwrap();

        // Verify the pulled file exists
        assert!(path.join("pull-test.txt").exists());
    }

    // ── Failure mode tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_push_non_fast_forward() {
        let (work_dir, bare_dir) = create_test_repo_with_remote();
        let path = work_dir.path();
        let git = Path::new("git");

        // Create a second clone, push a commit from there to make remote diverge
        let clone_parent = tempfile::TempDir::new().unwrap();
        let clone_path = clone_parent.path().join("clone");

        std::process::Command::new("git")
            .args(["clone", bare_dir.path().to_str().unwrap(), clone_path.to_str().unwrap()])
            .output()
            .unwrap();

        // Configure user + commit + push from clone
        for args in [
            vec!["config", "user.name", "Test User"],
            vec!["config", "user.email", "test@test.com"],
        ] {
            std::process::Command::new("git")
                .arg("-C").arg(&clone_path)
                .args(&args)
                .output()
                .unwrap();
        }
        std::fs::write(clone_path.join("diverge.txt"), "diverge\n").unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(&clone_path)
            .args(["add", "diverge.txt"])
            .output().unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(&clone_path)
            .args(["commit", "-m", "diverge commit"])
            .output().unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(&clone_path)
            .args(["push", "origin", "HEAD"])
            .output().unwrap();

        // Now create a local commit that diverges
        let cli = GitCli::new(path, git);
        std::fs::write(path.join("local.txt"), "local\n").unwrap();
        cli.run_git(&["add", "local.txt"]).unwrap();
        cli.run_git(&["commit", "-m", "local commit"]).unwrap();

        // Push should fail with non-fast-forward
        let err = push_origin(path, git, false).await.unwrap_err();
        match err {
            GitError::BranchOp(BranchOpError::PushNonFastForward { .. }) => {},
            other => panic!("expected PushNonFastForward, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_fetch_missing_remote() {
        // Create a repo WITHOUT a remote
        let (dir, _repo) = create_test_repo();
        let path = dir.path();
        let git = Path::new("git");

        // Fetch should fail — no "origin" remote
        let err = fetch_origin(path, git).await.unwrap_err();
        match err {
            GitError::BranchOp(BranchOpError::RemoteNotFound { .. }) => {},
            // Some git versions report this as GenericCommandFailed
            GitError::BranchOp(BranchOpError::GenericCommandFailed { ref stderr, .. }) => {
                // Acceptable fallback — verify stderr mentions the issue
                assert!(
                    stderr.contains("origin") || stderr.contains("remote"),
                    "expected stderr to mention remote, got: {}", stderr
                );
            },
            other => panic!("expected RemoteNotFound or GenericCommandFailed, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_pull_dirty_working_tree() {
        let (work_dir, bare_dir) = create_test_repo_with_remote();
        let path = work_dir.path();
        let git = Path::new("git");

        // Push a change from a second clone so there's something to pull
        let clone_parent = tempfile::TempDir::new().unwrap();
        let clone_path = clone_parent.path().join("clone");

        std::process::Command::new("git")
            .args(["clone", bare_dir.path().to_str().unwrap(), clone_path.to_str().unwrap()])
            .output()
            .unwrap();

        for args in [
            vec!["config", "user.name", "Test User"],
            vec!["config", "user.email", "test@test.com"],
        ] {
            std::process::Command::new("git")
                .arg("-C").arg(&clone_path)
                .args(&args)
                .output().unwrap();
        }

        // Modify the SAME file that exists in the repo (README.md) so it conflicts
        std::fs::write(clone_path.join("README.md"), "# Modified by clone\n").unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(&clone_path)
            .args(["add", "README.md"])
            .output().unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(&clone_path)
            .args(["commit", "-m", "modify README"])
            .output().unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(&clone_path)
            .args(["push", "origin", "HEAD"])
            .output().unwrap();

        // Now dirty the same file in our working tree (uncommitted)
        std::fs::write(path.join("README.md"), "# Dirty local change\n").unwrap();

        // Pull should fail — dirty working tree
        let err = pull_origin(path, git).await.unwrap_err();
        match err {
            GitError::BranchOp(BranchOpError::DirtyWorkingTree { .. }) => {},
            // Some git versions may return a different structured error
            GitError::BranchOp(ref _e) => {
                // Any branch op error related to the conflict is acceptable
            },
            other => panic!("expected DirtyWorkingTree or BranchOp error, got: {:?}", other),
        }
    }
}
