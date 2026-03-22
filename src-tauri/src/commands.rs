use std::path::PathBuf;

use serde::Serialize;

use crate::git::branch;
use crate::git::commit;
use crate::git::staging;
use crate::git::{Git2Repository, GitRepository};
use crate::git::repository::{ChangedFileEntry, CommitInfo, DiffFileEntry, FileContentPair, RefInfo};
use crate::git::commit::{AuthorInfo, CommitResult};
use crate::git::resolver::GitResolver;
use crate::recents::{self, RecentRepo};

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

#[tauri::command]
pub async fn get_changed_files(path: String) -> Result<Vec<ChangedFileEntry>, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Git2Repository::open(&path);
        repo.changed_files().map_err(|e| e.to_string())
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

/// Get ahead/behind commit counts relative to upstream tracking branch.
#[tauri::command]
pub async fn get_ahead_behind(path: String) -> Result<branch::AheadBehind, String> {
    let git = resolve_git()?;
    let path = PathBuf::from(path);
    branch::ahead_behind(&path, &git).await.map_err(String::from)
}

// ── Staging operation commands ─────────────────────────────────────────────────

/// Stage a single hunk from the working tree into the index.
#[tauri::command]
pub async fn stage_hunk(path: String, file_path: String, hunk_index: usize) -> Result<(), String> {
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

/// Stage selected lines from a single hunk into the index.
#[tauri::command]
pub async fn stage_lines(
    path: String,
    file_path: String,
    hunk_index: usize,
    line_indices: Vec<usize>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        staging::stage_lines(&path, &git, &file_path, hunk_index, &line_indices)
            .map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Unstage selected lines from the index back to the working tree.
#[tauri::command]
pub async fn unstage_lines(
    path: String,
    file_path: String,
    hunk_index: usize,
    line_indices: Vec<usize>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        staging::unstage_lines(&path, &git, &file_path, hunk_index, &line_indices)
            .map_err(String::from)
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

// ── Commit operation commands ──────────────────────────────────────────────────

/// Create a commit from staged changes.
///
/// If `amend` is true, amends the most recent commit.
/// Returns the SHA and summary of the created commit.
#[tauri::command]
pub async fn commit_changes(
    path: String,
    message: String,
    amend: bool,
) -> Result<CommitResult, String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        commit::commit_changes(&path, &git, &message, amend).map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Get the commit message of HEAD (for amend pre-fill).
#[tauri::command]
pub async fn get_head_message(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        commit::get_head_message(&path, &git).map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Get the configured author identity (name + email) from git config.
#[tauri::command]
pub async fn get_commit_defaults(path: String) -> Result<AuthorInfo, String> {
    tokio::task::spawn_blocking(move || {
        let git = resolve_git()?;
        let path = PathBuf::from(path);
        commit::get_author_config(&path, &git).map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn open_repo(app: tauri::AppHandle, path: String) -> Result<RecentRepo, String> {
    tokio::task::spawn_blocking(move || {
        let (abs_path, display_name) = recents::validate_repo_path(&path)?;
        let repos = recents::upsert_recent(&app, &abs_path, &display_name)?;
        repos
            .into_iter()
            .next()
            .ok_or_else(|| "Failed to store recent repository".to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn get_recent_repos(app: tauri::AppHandle) -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(move || recents::load_and_validate(&app))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn remove_recent_repo(
    app: tauri::AppHandle,
    path: String,
) -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(move || recents::remove_recent(&app, &path))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}
