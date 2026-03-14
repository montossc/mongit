use serde::Serialize;

use crate::git::{Git2Repository, GitRepository};
use crate::git::repository::{CommitInfo, RefInfo};

/// Basic greet command to test IPC
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Tauri IPC is working.", name)
}

/// Repository status response
#[derive(Debug, Serialize)]
pub struct RepoStatus {
    pub is_valid: bool,
    pub branch: Option<String>,
    pub changed_files: usize,
    pub staged_files: usize,
}

/// Get the status of a git repository.
/// Delegates to Git2Repository for all git2 operations.
#[tauri::command]
pub async fn get_repo_status(path: String) -> Result<RepoStatus, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Git2Repository::open(&path);
        let status = repo.status()?;
        let branch = repo.current_branch()?;

        Ok(RepoStatus {
            is_valid: true,
            branch,
            changed_files: status.changed_files,
            staged_files: status.staged_files,
        })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Get commit log for graph rendering (all branches).
/// max_count is capped at 50,000 to prevent unbounded resource usage.
#[tauri::command]
pub async fn get_commit_log(path: String, max_count: usize) -> Result<Vec<CommitInfo>, String> {
    let capped = max_count.min(50_000);
    tokio::task::spawn_blocking(move || {
        let repo = Git2Repository::open(&path);
        repo.log_all_branches(capped).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Get all refs (branches, tags, HEAD) for graph labels.
#[tauri::command]
pub async fn get_refs(path: String) -> Result<Vec<RefInfo>, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Git2Repository::open(&path);
        repo.refs().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
