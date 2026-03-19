use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::Manager;

pub const RECENTS_FILE: &str = "recent-repos.json";
pub const MAX_RECENTS: usize = 10;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RecentRepo {
    pub path: String,
    pub name: String,
    pub last_accessed: i64,
    pub valid: bool,
}

pub fn recents_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {e}"))?;

    Ok(app_data_dir.join(RECENTS_FILE))
}

pub fn load_raw(app: &tauri::AppHandle) -> Result<Vec<RecentRepo>, String> {
    let file_path = recents_file_path(app)?;

    if !file_path.exists() {
        return Ok(vec![]);
    }

    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Warning: Failed to read recents file at {}: {e} — starting fresh",
                file_path.display()
            );
            return Ok(vec![]);
        }
    };

    match serde_json::from_str(&content) {
        Ok(repos) => Ok(repos),
        Err(e) => {
            eprintln!(
                "Warning: Corrupted recents file at {}: {e} — starting fresh",
                file_path.display()
            );
            Ok(vec![])
        }
    }
}

pub fn save(app: &tauri::AppHandle, repos: &[RecentRepo]) -> Result<(), String> {
    let file_path = recents_file_path(app)?;

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create app data directory {}: {e}",
                parent.display()
            )
        })?;
    }

    let json = serde_json::to_string_pretty(repos)
        .map_err(|e| format!("Failed to serialize recents list: {e}"))?;

    // Atomic write: write to temp file, then rename to avoid corruption on crash
    let tmp_path = file_path.with_extension("json.tmp");
    fs::write(&tmp_path, &json).map_err(|e| {
        format!(
            "Failed to write temp recents file at {}: {e}",
            tmp_path.display()
        )
    })?;

    fs::rename(&tmp_path, &file_path).map_err(|e| {
        format!(
            "Failed to rename temp recents file to {}: {e}",
            file_path.display()
        )
    })
}

pub fn upsert_into_list(repos: &mut Vec<RecentRepo>, path: &str, name: &str, now_i64: i64) {
    remove_from_list(repos, path);

    repos.insert(
        0,
        RecentRepo {
            path: path.to_string(),
            name: name.to_string(),
            last_accessed: now_i64,
            valid: true,
        },
    );

    if repos.len() > MAX_RECENTS {
        repos.truncate(MAX_RECENTS);
    }
}

pub fn remove_from_list(repos: &mut Vec<RecentRepo>, path: &str) {
    repos.retain(|repo| repo.path != path);
}

pub fn validate_entries(repos: &mut Vec<RecentRepo>) {
    for repo in repos.iter_mut() {
        repo.valid = is_valid_git_repo(&repo.path);
    }
}

pub fn is_valid_git_repo(path: &str) -> bool {
    git2::Repository::open(path).is_ok()
}

pub fn validate_repo_path(path: &str) -> Result<(String, String), String> {
    let input = Path::new(path);

    if !input.exists() {
        return Err("Repository path does not exist".to_string());
    }

    if !input.is_dir() {
        return Err("Repository path is not a directory".to_string());
    }

    if !is_valid_git_repo(path) {
        return Err("Selected folder is not a Git repository".to_string());
    }

    let abs = input
        .canonicalize()
        .map_err(|e| format!("Failed to resolve absolute repository path: {e}"))?;

    let abs_path = abs.to_string_lossy().to_string();
    let display_name = abs
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| abs_path.clone());

    Ok((abs_path, display_name))
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn upsert_recent(
    app: &tauri::AppHandle,
    path: &str,
    name: &str,
) -> Result<Vec<RecentRepo>, String> {
    let mut repos = load_raw(app)?;
    upsert_into_list(&mut repos, path, name, now_secs());
    save(app, &repos)?;
    Ok(repos)
}

pub fn load_and_validate(app: &tauri::AppHandle) -> Result<Vec<RecentRepo>, String> {
    let mut repos = load_raw(app)?;
    let before = repos.clone();
    validate_entries(&mut repos);

    if repos != before {
        save(app, &repos)?;
    }

    Ok(repos)
}

pub fn remove_recent(app: &tauri::AppHandle, path: &str) -> Result<Vec<RecentRepo>, String> {
    let mut repos = load_raw(app)?;
    remove_from_list(&mut repos, path);
    save(app, &repos)?;
    Ok(repos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(path: &str, name: &str, ts: i64, valid: bool) -> RecentRepo {
        RecentRepo {
            path: path.to_string(),
            name: name.to_string(),
            last_accessed: ts,
            valid,
        }
    }

    #[test]
    fn upsert_adds_at_front() {
        let mut repos = vec![repo("/a", "a", 1, true)];

        upsert_into_list(&mut repos, "/b", "b", 2);

        assert_eq!(repos[0], repo("/b", "b", 2, true));
        assert_eq!(repos[1], repo("/a", "a", 1, true));
    }

    #[test]
    fn upsert_moves_existing_to_front() {
        let mut repos = vec![repo("/a", "a", 1, true), repo("/b", "b", 2, true)];

        upsert_into_list(&mut repos, "/b", "b2", 3);

        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0], repo("/b", "b2", 3, true));
        assert_eq!(repos[1], repo("/a", "a", 1, true));
    }

    #[test]
    fn upsert_enforces_lru_cap() {
        let mut repos = Vec::new();

        for i in 0..(MAX_RECENTS + 2) {
            let path = format!("/repo-{i}");
            let name = format!("repo-{i}");
            upsert_into_list(&mut repos, &path, &name, i as i64);
        }

        assert_eq!(repos.len(), MAX_RECENTS);
        assert_eq!(repos[0].path, "/repo-11");
        assert_eq!(repos[MAX_RECENTS - 1].path, "/repo-2");
    }

    #[test]
    fn remove_deletes_entry() {
        let mut repos = vec![repo("/a", "a", 1, true), repo("/b", "b", 2, true)];

        remove_from_list(&mut repos, "/a");

        assert_eq!(repos, vec![repo("/b", "b", 2, true)]);
    }

    #[test]
    fn remove_noop_for_missing() {
        let original = vec![repo("/a", "a", 1, true)];
        let mut repos = original.clone();

        remove_from_list(&mut repos, "/missing");

        assert_eq!(repos, original);
    }

    #[test]
    fn validate_entries_marks_invalid() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        git2::Repository::init(temp.path()).expect("git repo should initialize");

        let valid_path = temp.path().to_string_lossy().to_string();
        let invalid_path = temp.path().join("not-a-repo").to_string_lossy().to_string();

        let mut repos = vec![
            repo(&valid_path, "valid", 1, false),
            repo(&invalid_path, "invalid", 1, true),
        ];

        validate_entries(&mut repos);

        assert!(repos[0].valid);
        assert!(!repos[1].valid);
    }

    #[test]
    fn validate_repo_path_rejects_nonexistent() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let missing = temp.path().join("missing-repo");

        let result = validate_repo_path(&missing.to_string_lossy());
        assert!(result.is_err());
    }

    #[test]
    fn validate_repo_path_rejects_non_git_dir() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");

        let result = validate_repo_path(&temp.path().to_string_lossy());
        assert!(result.is_err());
    }

    #[test]
    fn validate_repo_path_accepts_valid_repo() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        git2::Repository::init(temp.path()).expect("git repo should initialize");

        let result = validate_repo_path(&temp.path().to_string_lossy());
        assert!(result.is_ok());

        let (abs_path, name) = result.expect("valid repo should pass");
        assert!(abs_path.contains(name.as_str()));
    }
}
