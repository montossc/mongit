#![allow(dead_code)] // Public API; consumers not yet wired

use std::path::{Path, PathBuf};
use std::process::Command;

use super::GitError;

/// Write operations backed by the git CLI (shell-out pattern).
///
/// git2 lacks support for hooks, GPG signing, and credential helpers,
/// so all mutating operations go through the real git binary.
/// Each call spawns a new `git -C <path>` process.
///
/// The `git_executable` field holds the resolved absolute path to the git
/// binary (from `GitResolver`), ensuring deterministic invocations.
pub struct GitCli {
    path: PathBuf,
    git_executable: PathBuf,
}

impl GitCli {
    /// Create a new GitCli writer for the given working directory.
    ///
    /// `git_executable` should be a resolved path from `GitResolver::resolve()`.
    /// Falls back to `"git"` only in tests.
    pub fn new(path: impl Into<PathBuf>, git_executable: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            git_executable: git_executable.into(),
        }
    }

    /// Access the stored path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create a local branch.
    ///
    /// If `start_point` is provided, the branch starts from that ref/commit
    /// (e.g. "origin/main", "abc1234"). Otherwise starts from HEAD.
    pub fn create_branch(
        &self,
        branch_name: &str,
        start_point: Option<&str>,
    ) -> Result<(), GitError> {
        let mut args = vec!["branch", "--", branch_name];
        if let Some(sp) = start_point {
            args.push(sp);
        }
        self.run_git(&args)?;
        Ok(())
    }

    /// Switch to an existing branch.
    pub fn switch_branch(&self, branch_name: &str) -> Result<(), GitError> {
        self.run_git(&["switch", "--", branch_name])?;
        Ok(())
    }

    /// Run a git command in the repository directory.
    ///
    /// Returns stdout on success, or a structured `CommandFailed` error
    /// with the command string, stderr, and exit code.
    pub(crate) fn run_git(&self, args: &[&str]) -> Result<String, GitError> {
        let path_str = self.path.to_string_lossy();
        let output = Command::new(&self.git_executable)
            .arg("-C")
            .arg(path_str.as_ref())
            .args(args)
            .output()?;

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

    /// Run a git command asynchronously using `tokio::process::Command`.
    ///
    /// Designed for remote operations (fetch, pull, push) that may block for
    /// seconds on network I/O. The thread is released during I/O wait.
    ///
    /// Always sets `GIT_TERMINAL_PROMPT=0` to prevent credential prompts
    /// from hanging the subprocess.
    pub(crate) async fn run_git_async(&self, args: &[&str]) -> Result<String, GitError> {
        let path_str = self.path.to_string_lossy();
        let output = tokio::process::Command::new(&self.git_executable)
            .arg("-C")
            .arg(path_str.as_ref())
            .args(args)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .await?;

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
}
