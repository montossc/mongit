mod commands;
mod git;
mod watcher;

use std::sync::Mutex;
use watcher::ActiveWatcher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(None::<ActiveWatcher>))
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_repo_status,
            commands::get_commit_log,
            commands::get_refs,
            commands::create_branch,
            commands::switch_branch,
            commands::delete_branch,
            commands::fetch,
            commands::pull,
            commands::push,
            watcher::watch_repo,
            watcher::stop_watching,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
