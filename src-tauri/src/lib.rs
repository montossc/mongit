mod commands;
mod git;
mod recents;
mod watcher;

use watcher::WatcherState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(WatcherState::default())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_repo_status,
            commands::get_commit_log,
            commands::get_refs,
            commands::get_diff_workdir,
            commands::get_changed_files,
            commands::get_file_content_for_diff,
            commands::create_branch,
            commands::switch_branch,
            commands::delete_branch,
            commands::fetch,
            commands::pull,
            commands::push,
            commands::open_repo,
            commands::get_recent_repos,
            commands::remove_recent_repo,
            commands::stage_hunk,
            commands::unstage_hunk,
            commands::stage_lines,
            commands::unstage_lines,
            commands::get_diff_index,
            commands::get_merge_state,
            commands::get_conflict_content,
            watcher::watch_repo,
            watcher::stop_watching,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
