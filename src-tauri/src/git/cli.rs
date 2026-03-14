#![allow(dead_code)] // Public API; consumers not yet wired

use std::path::{Path, PathBuf};
use std::process::Command;

use super::GitError;

/// Write operations backed by the git CLI (shell-out pattern).
///
/// git2 lacks support for hooks, GPG signing, and credential helpers,
/// so all mutating operations go through the real git binary.
/// Each call spawns a new `git -C <path>` process.
pub struct GitCli {
    path: PathBuf,
}

impl GitCli {
    /// Create a new GitCli writer for the given working directory.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
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
    fn run_git(&self, args: &[&str]) -> Result<String, GitError> {
        let path_str = self.path.to_string_lossy();
        let output = Command::new("git")
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
}
