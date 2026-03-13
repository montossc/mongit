use serde::Serialize;

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

/// Get the status of a git repository
#[tauri::command]
pub fn get_repo_status(path: String) -> Result<RepoStatus, String> {
    let repo = git2::Repository::open(&path).map_err(|e| format!("Failed to open repo: {}", e))?;

    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from));

    let statuses = repo
        .statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ))
        .map_err(|e| format!("Failed to get status: {}", e))?;

    let mut changed = 0;
    let mut staged = 0;

    for entry in statuses.iter() {
        let status = entry.status();
        if status.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_NEW
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE,
        ) {
            changed += 1;
        }
        if status.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }
    }

    Ok(RepoStatus {
        is_valid: true,
        branch,
        changed_files: changed,
        staged_files: staged,
    })
}
