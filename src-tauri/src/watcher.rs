use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer, RecommendedCache,
};
use git2::Repository;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};

const WATCH_DEBOUNCE_MS: u64 = 300;

/// Type alias for the file watcher handle
pub type WatcherHandle = Debouncer<RecommendedWatcher, RecommendedCache>;

/// Managed state wrapping the optional watcher
pub type WatcherState = Mutex<Option<WatcherHandle>>;

fn canonicalize_repo_path(path: &str) -> Result<PathBuf, String> {
    let watch_path = PathBuf::from(path);
    if !watch_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let canonical = watch_path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {e}"))?;

    Repository::open(&canonical)
        .map_err(|e| format!("Not a git repository: {e}"))?;

    Ok(canonical)
}

/// Determine whether a file change at the given path should emit a `repo-changed` event.
///
/// Rules:
/// - SUPPRESS: target/, node_modules/, .git/objects/, .git/logs/
/// - ALLOW: .git/index, .git/HEAD, .git/refs/* (staging, branch switch, commit)
/// - ALLOW: everything else in the working tree
fn should_emit_for_path(path: &Path) -> bool {
    let components: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    // Suppress: target/, node_modules/
    if components.iter().any(|c| c == "target" || c == "node_modules") {
        return false;
    }

    // Handle .git paths
    if let Some(git_idx) = components.iter().position(|c| c == ".git") {
        if let Some(next) = components.get(git_idx + 1) {
            // Suppress: .git/objects/, .git/logs/
            if next == "objects" || next == "logs" {
                return false;
            }
            // Allow: .git/index, .git/HEAD, .git/refs/ etc.
            return true;
        }
        // .git itself changed — allow
        return true;
    }

    true
}

fn should_emit_for_paths(paths: &[PathBuf]) -> bool {
    paths.iter().any(|p| should_emit_for_path(p))
}

/// Start watching a repository for file changes.
///
/// Emits `repo-changed` Tauri events to the frontend when relevant files change.
/// If already watching a different path, the old watcher is dropped first.
#[tauri::command]
pub async fn watch_repo(
    app: AppHandle,
    path: String,
    watcher_state: State<'_, WatcherState>,
) -> Result<(), String> {
    let watch_path = canonicalize_repo_path(&path)?;

    let app_clone = app.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(WATCH_DEBOUNCE_MS),
        None,
        move |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    let should_emit = events
                        .iter()
                        .any(|e| should_emit_for_paths(&e.paths));
                    if should_emit {
                        let _ = app_clone.emit("repo-changed", ());
                    }
                }
                Err(errors) => {
                    eprintln!("[watcher] debouncer errors: {:?}", errors);
                    let _ = app_clone.emit("repo-watcher-error", format!("{:?}", errors));
                }
            }
        },
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    debouncer
        .watch(&watch_path, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch path: {}", e))?;

    // Replace old watcher (dropping it stops the old one)
    let mut state = watcher_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {}", e))?;
    *state = Some(debouncer);

    Ok(())
}

/// Stop watching the current repository.
#[tauri::command]
pub async fn stop_watching(
    watcher_state: State<'_, WatcherState>,
) -> Result<(), String> {
    let mut state = watcher_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {}", e))?;
    *state = None; // Dropping ActiveWatcher (and debouncer handle) stops the watcher
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_should_emit_working_tree_file() {
        assert!(should_emit_for_path(&PathBuf::from("/repo/src/main.rs")));
    }

    #[test]
    fn test_should_not_emit_git_objects() {
        assert!(!should_emit_for_path(&PathBuf::from("/repo/.git/objects/ab/cdef123")));
    }

    #[test]
    fn test_should_not_emit_git_logs() {
        assert!(!should_emit_for_path(&PathBuf::from("/repo/.git/logs/HEAD")));
    }

    #[test]
    fn test_should_emit_git_index() {
        assert!(should_emit_for_path(&PathBuf::from("/repo/.git/index")));
    }

    #[test]
    fn test_should_emit_git_head() {
        assert!(should_emit_for_path(&PathBuf::from("/repo/.git/HEAD")));
    }

    #[test]
    fn test_should_emit_git_refs() {
        assert!(should_emit_for_path(&PathBuf::from("/repo/.git/refs/heads/main")));
    }

    #[test]
    fn test_should_not_emit_node_modules() {
        assert!(!should_emit_for_path(&PathBuf::from("/repo/node_modules/pkg/index.js")));
    }

    #[test]
    fn test_should_not_emit_target() {
        assert!(!should_emit_for_path(&PathBuf::from("/repo/target/debug/build")));
    }

    #[test]
    fn test_should_emit_git_directory_root() {
        assert!(should_emit_for_path(&PathBuf::from("/repo/.git")));
    }

    #[test]
    fn test_should_not_emit_nested_target_path() {
        assert!(!should_emit_for_path(&PathBuf::from(
            "/repo/packages/ui/target/tmp/output"
        )));
    }

    #[test]
    fn test_should_emit_for_paths_mixed_batch() {
        let paths = vec![
            PathBuf::from("/repo/node_modules/pkg/index.js"),
            PathBuf::from("/repo/src/main.rs"),
        ];
        assert!(should_emit_for_paths(&paths));
    }

    #[test]
    fn test_should_not_emit_for_paths_all_suppressed() {
        let paths = vec![
            PathBuf::from("/repo/node_modules/pkg/index.js"),
            PathBuf::from("/repo/target/debug/build"),
            PathBuf::from("/repo/.git/objects/ab/cdef123"),
        ];
        assert!(!should_emit_for_paths(&paths));
    }

    #[test]
    fn test_watch_debounce_constant_is_expected() {
        assert_eq!(WATCH_DEBOUNCE_MS, 300);
    }

    #[test]
    fn test_canonicalize_repo_path_rejects_missing_path() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let missing = dir.path().join("missing-child-path");
        let missing_str = missing.to_str().expect("utf-8 temp path");

        let err = canonicalize_repo_path(missing_str).unwrap_err();
        assert!(err.contains("Path does not exist"));
    }

    #[test]
    fn test_canonicalize_repo_path_rejects_non_repo_path() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = dir.path().to_str().expect("utf-8 temp path");

        let err = canonicalize_repo_path(path).unwrap_err();
        assert!(err.contains("Not a git repository"));
    }

    #[test]
    fn test_canonicalize_repo_path_accepts_git_repo() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        git2::Repository::init(dir.path()).expect("init repo");

        let path = dir.path().to_str().expect("utf-8 temp path");
        let canonical = canonicalize_repo_path(path).expect("repo path should be valid");

        assert!(canonical.is_absolute());
        assert_eq!(canonical, dir.path().canonicalize().expect("canonicalize"));
    }

}
