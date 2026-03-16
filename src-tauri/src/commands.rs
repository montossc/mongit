use std::path::PathBuf;

use serde::Serialize;

use crate::git::branch;
use crate::git::{Git2Repository, GitRepository};
use crate::git::repository::{CommitInfo, DiffFileEntry, FileContentPair, RefInfo};
use crate::git::resolver::GitResolver;

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

/// Get the working directory diff for a repository.
/// Returns a list of changed files with hunk-level detail.
#[tauri::command]
pub async fn get_diff_workdir(path: String) -> Result<Vec<DiffFileEntry>, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Git2Repository::open(&path);
        repo.diff_workdir().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Get file content for diff rendering (original from HEAD, modified from working tree).
#[tauri::command]
pub async fn get_file_content_for_diff(
    path: String,
    file_path: String,
) -> Result<FileContentPair, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Git2Repository::open(&path);
        repo.file_content_for_diff(&file_path)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

// ── Branch operation commands ──────────────────────────────────────────────────────
/// Resolve the git binary path via GitResolver.
/// Called at the start of every branch command.
fn resolve_git() -> Result<PathBuf, String> {
    let resolved = GitResolver::resolve().map_err(String::from)?;
    Ok(resolved.path)
}

/// Create a new local branch.
#[tauri::command]
pub async fn create_branch(
    path: String,
    branch_name: String,
    start_point: Option<String>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        branch::create_branch(&path, &git, &branch_name, start_point.as_deref())
            .map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Switch to an existing branch.
#[tauri::command]
pub async fn switch_branch(path: String, branch_name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        branch::switch_branch(&path, &git, &branch_name).map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Delete a local branch with optional force.
#[tauri::command]
pub async fn delete_branch(
    path: String,
    branch_name: String,
    force: bool,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        branch::delete_branch(&path, &git, &branch_name, force).map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Fetch latest refs and objects from origin.
#[tauri::command]
pub async fn fetch(path: String) -> Result<String, String> {
    let git = resolve_git()?;
    let path = PathBuf::from(path);
    branch::fetch_origin(&path, &git)
        .await
        .map_err(String::from)
}

/// Pull changes from origin into the current branch.
#[tauri::command]
pub async fn pull(path: String) -> Result<String, String> {
    let git = resolve_git()?;
    let path = PathBuf::from(path);
    branch::pull_origin(&path, &git)
        .await
        .map_err(String::from)
}

/// Push current branch to origin.
/// If `force_with_lease` is true, uses `--force-with-lease` (never bare `--force`).
#[tauri::command]
pub async fn push(path: String, force_with_lease: bool) -> Result<String, String> {
    let git = resolve_git()?;
    let path = PathBuf::from(path);
    branch::push_origin(&path, &git, force_with_lease)
        .await
        .map_err(String::from)
}
