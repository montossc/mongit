#![allow(dead_code)] // Public API; consumers not yet wired

use std::process::Command;

use super::GitError;

pub struct GitCli;

impl GitCli {
    pub fn create_branch(repo_path: &str, branch_name: &str) -> Result<(), GitError> {
        Self::run_git(repo_path, &["branch", branch_name])?;
        Ok(())
    }

    pub fn switch_branch(repo_path: &str, branch_name: &str) -> Result<(), GitError> {
        Self::run_git(repo_path, &["switch", branch_name])?;
        Ok(())
    }

    fn run_git(repo_path: &str, args: &[&str]) -> Result<String, GitError> {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(args)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(GitError::Cli(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
}
