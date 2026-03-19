# Repo Selection and Persistence — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use skill({ name: "executing-plans" }) to implement this plan task-by-task.

**Goal:** Build the product entry point for mongit: opening repos via picker/path/drag-drop, persisting a 10-item LRU recents list in Tauri app data, and routing into the `/repo` workspace.

**Architecture:** A new Rust `recents` module handles repo validation and LRU persistence (JSON in app data dir). Three new Tauri commands expose this to the frontend. A Svelte 5 runes store orchestrates the open→validate→persist→navigate lifecycle. The home route (`/`) replaces the spike surface with entry UI; `/repo` provides a minimal workspace shell.

**Tech Stack:** Tauri 2.0, Svelte 5 (runes), Rust, git2, tauri-plugin-dialog, serde_json

---

## Must-Haves

**Goal:** User can open a local Git repository, see recent repos across restarts, and land in the repo workspace.

### Observable Truths

1. User can open a repo via native folder picker, manual path entry, or drag-drop
2. Successful open navigates to `/repo` with active repo state hydrated
3. Recent repos persist across app restarts (10-item LRU in Tauri app data)
4. Stale recent repos remain visible with retry/remove actions
5. Invalid paths show specific error messages without navigation

### Required Artifacts

| Artifact | Provides | Path |
|----------|----------|------|
| Recents persistence module | RecentRepo struct, JSON read/write, LRU logic, path validation | `src-tauri/src/recents.rs` |
| Backend commands | `open_repo`, `get_recent_repos`, `remove_recent_repo` | `src-tauri/src/commands.rs` |
| Command registration | New commands + dialog plugin | `src-tauri/src/lib.rs` |
| Dialog plugin config | Native folder picker capability | `Cargo.toml`, `tauri.conf.json`, `package.json` |
| Repo store | Active repo state, recents, open lifecycle, navigation | `src/lib/stores/repo.svelte.ts` |
| Home route | Entry surface: picker, path input, drag-drop, recents list | `src/routes/+page.svelte` |
| Repo route shell | Workspace handoff target with guard | `src/routes/repo/+layout.svelte`, `src/routes/repo/+page.svelte` |

### Key Links

| From | To | Via | Risk |
|------|-----|-----|------|
| Home route | Backend commands | `invoke()` IPC | Type mismatch TS↔Rust |
| Repo store | SvelteKit routing | `goto('/repo')` | Navigate before state ready |
| Recents module | Filesystem | `app_data_dir()` JSON | Corrupt JSON, permissions |
| Home route | Dialog plugin | `@tauri-apps/plugin-dialog` | Plugin not registered |
| Repo route guard | Repo store | `repoStore.activeRepoPath` | Direct URL access without open |

### Task Dependencies

```
Task 1 (Backend): needs nothing
  creates: src-tauri/src/recents.rs, modifies commands.rs + lib.rs + configs
  
Task 2 (Store): needs Task 1
  creates: src/lib/stores/repo.svelte.ts

Task 3 (Home UI): needs Task 2
  modifies: src/routes/+page.svelte

Task 4 (Repo Shell): needs Task 2
  creates: src/routes/repo/+layout.svelte, src/routes/repo/+page.svelte

Wave 1: Task 1
Wave 2: Task 2
Wave 3: Task 3 + Task 4 (parallel — no file overlap)
```

---

## Task 1: Backend repo validation and recents persistence

**Tier:** worker

**Files:**
- Create: `src-tauri/src/recents.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `package.json`

**Handoff Contract:**
- **Produces:** Three Tauri commands (`open_repo`, `get_recent_repos`, `remove_recent_repo`) callable via `invoke()` from frontend
- **Consumed By:** Task 2 (Frontend repo store)

### Step 1: Add dialog plugin dependencies

**Modify `src-tauri/Cargo.toml`** — add dialog plugin after line 17 (`tauri-plugin-shell = "2"`):

```toml
tauri-plugin-dialog = "2"
```

**Modify `package.json`** — add to `dependencies` (alongside existing `@tauri-apps/api`):

```json
"@tauri-apps/plugin-dialog": "^2.2.0"
```

Run: `cd src-tauri && cargo check` (to verify Cargo.toml is valid)
Run: `pnpm install` (to install the frontend plugin package)

### Step 2: Register dialog plugin and add config

**Modify `src-tauri/src/lib.rs`** — add `mod recents;` declaration after `mod git;` (line 2), and add dialog plugin registration after shell plugin (line 10):

Current lib.rs line 10: `.plugin(tauri_plugin_shell::init())`

After edit, lib.rs should be:

```rust
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
            watcher::watch_repo,
            watcher::stop_watching,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Modify `src-tauri/tauri.conf.json`** — add `"dialog": {}` to plugins object (after `"shell"` block):

```json
"plugins": {
    "shell": {
        "open": true
    },
    "dialog": {}
}
```

### Step 3: Create the recents persistence module

**Create `src-tauri/src/recents.rs`** with the following content:

```rust
//! Recent repository persistence — 10-item LRU in Tauri app data.
//!
//! Storage: `{app_data_dir}/recent-repos.json`
//! Format: JSON array of `RecentRepo` sorted by `last_accessed` descending.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::Manager;

const RECENTS_FILE: &str = "recent-repos.json";
const MAX_RECENTS: usize = 10;

/// A recently opened repository entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecentRepo {
    /// Absolute filesystem path.
    pub path: String,
    /// Display name (directory basename).
    pub name: String,
    /// Unix timestamp (seconds) of last successful open.
    pub last_accessed: i64,
    /// Whether the path currently resolves to a valid Git repo.
    pub valid: bool,
}

// ── File I/O ────────────────────────────────────────────────────────────

/// Resolve the path to the recents JSON file, creating the directory if needed.
fn recents_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create app data dir: {e}"))?;
    Ok(data_dir.join(RECENTS_FILE))
}

/// Load recent repos from disk (no validation).
fn load_raw(app: &tauri::AppHandle) -> Result<Vec<RecentRepo>, String> {
    let path = recents_file_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read recents: {e}"))?;
    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse recents: {e}"))
}

/// Save recent repos to disk.
fn save(app: &tauri::AppHandle, repos: &[RecentRepo]) -> Result<(), String> {
    let path = recents_file_path(app)?;
    let contents =
        serde_json::to_string_pretty(repos).map_err(|e| format!("Failed to serialize: {e}"))?;
    fs::write(&path, contents).map_err(|e| format!("Failed to write recents: {e}"))
}

// ── Core logic (pure, testable) ─────────────────────────────────────────

/// Insert or update an entry at the front of the list, enforcing the LRU cap.
pub fn upsert_into_list(repos: &mut Vec<RecentRepo>, path: &str, name: &str, now: i64) {
    repos.retain(|r| r.path != path);
    repos.insert(
        0,
        RecentRepo {
            path: path.to_string(),
            name: name.to_string(),
            last_accessed: now,
            valid: true,
        },
    );
    repos.truncate(MAX_RECENTS);
}

/// Remove an entry by path.
pub fn remove_from_list(repos: &mut Vec<RecentRepo>, path: &str) {
    repos.retain(|r| r.path != path);
}

/// Validate each entry's path against the filesystem.
pub fn validate_entries(repos: &mut Vec<RecentRepo>) {
    for repo in repos.iter_mut() {
        repo.valid = is_valid_git_repo(&repo.path);
    }
}

// ── Path validation ─────────────────────────────────────────────────────

/// Check whether a path points to a valid Git repository.
pub fn is_valid_git_repo(path: &str) -> bool {
    git2::Repository::open(path).is_ok()
}

/// Validate a path and return `(absolute_path, display_name)`.
///
/// Errors are user-facing strings.
pub fn validate_repo_path(path: &str) -> Result<(String, String), String> {
    let p = Path::new(path);

    if !p.exists() {
        return Err(format!("Path does not exist: {path}"));
    }
    if !p.is_dir() {
        return Err(format!("Path is not a directory: {path}"));
    }

    let canonical = p
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path: {e}"))?;
    let abs_path = canonical.to_string_lossy().to_string();

    git2::Repository::open(&abs_path)
        .map_err(|e| format!("Not a valid Git repository: {e}"))?;

    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| abs_path.clone());

    Ok((abs_path, name))
}

// ── Public API (Tauri-integrated) ───────────────────────────────────────

/// Current unix timestamp in seconds.
fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Add or update a recent repo entry. Returns the updated list.
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

/// Load all recents with current validity.
pub fn load_and_validate(app: &tauri::AppHandle) -> Result<Vec<RecentRepo>, String> {
    let mut repos = load_raw(app)?;
    validate_entries(&mut repos);
    Ok(repos)
}

/// Remove a recent repo by path. Returns the updated list.
pub fn remove_recent(app: &tauri::AppHandle, path: &str) -> Result<Vec<RecentRepo>, String> {
    let mut repos = load_raw(app)?;
    remove_from_list(&mut repos, path);
    save(app, &repos)?;
    Ok(repos)
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_repo(path: &str, name: &str, ts: i64) -> RecentRepo {
        RecentRepo {
            path: path.to_string(),
            name: name.to_string(),
            last_accessed: ts,
            valid: true,
        }
    }

    #[test]
    fn upsert_adds_new_entry_at_front() {
        let mut list = vec![make_repo("/a", "a", 100)];
        upsert_into_list(&mut list, "/b", "b", 200);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].path, "/b");
        assert_eq!(list[0].last_accessed, 200);
    }

    #[test]
    fn upsert_moves_existing_to_front() {
        let mut list = vec![
            make_repo("/a", "a", 300),
            make_repo("/b", "b", 200),
            make_repo("/c", "c", 100),
        ];
        upsert_into_list(&mut list, "/c", "c", 400);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].path, "/c");
        assert_eq!(list[0].last_accessed, 400);
    }

    #[test]
    fn upsert_enforces_lru_cap() {
        let mut list: Vec<RecentRepo> = (0..10)
            .map(|i| make_repo(&format!("/repo{i}"), &format!("repo{i}"), i as i64))
            .collect();
        assert_eq!(list.len(), 10);

        upsert_into_list(&mut list, "/new", "new", 100);
        assert_eq!(list.len(), MAX_RECENTS);
        assert_eq!(list[0].path, "/new");
        // The oldest entry (/repo0, ts=0) should have been evicted
        assert!(!list.iter().any(|r| r.path == "/repo0"));
    }

    #[test]
    fn remove_deletes_matching_entry() {
        let mut list = vec![
            make_repo("/a", "a", 100),
            make_repo("/b", "b", 200),
        ];
        remove_from_list(&mut list, "/a");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].path, "/b");
    }

    #[test]
    fn remove_no_op_for_missing_path() {
        let mut list = vec![make_repo("/a", "a", 100)];
        remove_from_list(&mut list, "/nonexistent");
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn validate_entries_marks_invalid_paths() {
        let mut list = vec![
            make_repo("/nonexistent/repo", "repo", 100),
        ];
        validate_entries(&mut list);
        assert!(!list[0].valid);
    }

    #[test]
    fn validate_repo_path_rejects_nonexistent() {
        let result = validate_repo_path("/this/path/does/not/exist/at/all");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn validate_repo_path_rejects_non_git_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = validate_repo_path(dir.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a valid Git repository"));
    }

    #[test]
    fn validate_repo_path_accepts_valid_repo() {
        let dir = tempfile::tempdir().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        let result = validate_repo_path(dir.path().to_str().unwrap());
        assert!(result.is_ok());
        let (abs_path, name) = result.unwrap();
        assert!(!abs_path.is_empty());
        assert!(!name.is_empty());
    }
}
```

### Step 4: Add Tauri commands to commands.rs

**Modify `src-tauri/src/commands.rs`** — add import for recents module after line 8 (`use crate::git::resolver::GitResolver;`):

```rust
use crate::recents::{self, RecentRepo};
```

Then append the following three commands after line 178 (end of `push` command):

```rust
// ── Repo selection and recents commands ─────────────────────────────────────────

/// Open a repository: validate the path, save to recents, return the entry.
/// The frontend should call `get_repo_status` separately for status hydration.
#[tauri::command]
pub async fn open_repo(app: tauri::AppHandle, path: String) -> Result<RecentRepo, String> {
    // Validate and canonicalize
    let (abs_path, name) = tokio::task::spawn_blocking(move || recents::validate_repo_path(&path))
        .await
        .map_err(|e| format!("Task join error: {e}"))??;

    // Save to recents
    let abs_path_clone = abs_path.clone();
    let name_clone = name.clone();
    let repos = tokio::task::spawn_blocking(move || {
        recents::upsert_recent(&app, &abs_path_clone, &name_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    // Return the entry we just inserted (always first after upsert)
    repos
        .into_iter()
        .next()
        .ok_or_else(|| "Failed to create recent entry".to_string())
}

/// Load all recent repositories with current validity state.
#[tauri::command]
pub async fn get_recent_repos(app: tauri::AppHandle) -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(move || recents::load_and_validate(&app))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}

/// Remove a repository from the recents list.
#[tauri::command]
pub async fn remove_recent_repo(
    app: tauri::AppHandle,
    path: String,
) -> Result<Vec<RecentRepo>, String> {
    tokio::task::spawn_blocking(move || recents::remove_recent(&app, &path))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}
```

### Step 5: Verify backend

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

Run: `cd src-tauri && cargo test`
Expected: all tests pass, including the new recents module tests

---

## Task 2: Frontend repo store and open lifecycle

**Tier:** worker

**Files:**
- Create: `src/lib/stores/repo.svelte.ts`

**Handoff Contract:**
- **Produces:** `repoStore` singleton with active repo state, recents, and open/remove/retry/picker actions
- **Consumed By:** Task 3 (Home route), Task 4 (Repo route shell)

### Step 1: Create the repo store

**Create `src/lib/stores/repo.svelte.ts`:**

```typescript
/**
 * Repo store — Svelte 5 runes-based repository selection and recents management.
 *
 * Owns:
 *   - Active repo path/name/status
 *   - Recent repos list (loaded from Tauri app data)
 *   - Open lifecycle: validate → persist → hydrate status → navigate
 *   - Error and loading state
 */

import { invoke } from '@tauri-apps/api/core';
import { goto } from '$app/navigation';

// ── Types matching Rust serialization ────────────────────────────────────

export interface RecentRepo {
	path: string;
	name: string;
	last_accessed: number;
	valid: boolean;
}

export interface RepoStatus {
	is_valid: boolean;
	branch: string | null;
	changed_files: number;
	staged_files: number;
}

// ── Store ────────────────────────────────────────────────────────────────

function createRepoStore() {
	let activeRepoPath = $state<string | null>(null);
	let activeRepoName = $state<string | null>(null);
	let repoStatus = $state<RepoStatus | null>(null);
	let recentRepos = $state<RecentRepo[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	/** Load the recent repos list from backend (with validity checks). */
	async function loadRecentRepos(): Promise<void> {
		try {
			recentRepos = await invoke<RecentRepo[]>('get_recent_repos');
		} catch (e) {
			// Non-fatal: log but don't block UI
			console.error('Failed to load recent repos:', e);
		}
	}

	/**
	 * Open a repository by path.
	 *
	 * Flow: validate → save to recents → hydrate status → navigate to /repo.
	 */
	async function openRepo(path: string): Promise<void> {
		const trimmed = path.trim();
		if (!trimmed) return;

		loading = true;
		error = null;

		try {
			// Validate path + save to recents (single backend call)
			const entry = await invoke<RecentRepo>('open_repo', { path: trimmed });

			// Hydrate repo status using existing command
			const status = await invoke<RepoStatus>('get_repo_status', {
				path: entry.path,
			});

			// Commit to store state
			activeRepoPath = entry.path;
			activeRepoName = entry.name;
			repoStatus = status;

			// Refresh recents list
			await loadRecentRepos();

			// Navigate to workspace
			await goto('/repo');
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	/** Open native folder picker, then open the selected directory. */
	async function openFolderPicker(): Promise<void> {
		try {
			const { open } = await import('@tauri-apps/plugin-dialog');
			const selected = await open({
				directory: true,
				multiple: false,
				title: 'Select Git Repository',
			});
			if (selected) {
				await openRepo(selected as string);
			}
		} catch (e) {
			error = String(e);
		}
	}

	/** Retry opening a stale recent repo (same flow as openRepo). */
	async function retryRecentRepo(path: string): Promise<void> {
		await openRepo(path);
	}

	/** Remove a repo from the recents list. */
	async function removeRecentRepo(path: string): Promise<void> {
		try {
			recentRepos = await invoke<RecentRepo[]>('remove_recent_repo', { path });
		} catch (e) {
			console.error('Failed to remove recent repo:', e);
		}
	}

	/** Clear the current error message. */
	function clearError(): void {
		error = null;
	}

	return {
		get activeRepoPath() {
			return activeRepoPath;
		},
		get activeRepoName() {
			return activeRepoName;
		},
		get repoStatus() {
			return repoStatus;
		},
		get recentRepos() {
			return recentRepos;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		loadRecentRepos,
		openRepo,
		openFolderPicker,
		retryRecentRepo,
		removeRecentRepo,
		clearError,
	};
}

export const repoStore = createRepoStore();
```

### Step 2: Verify

Run: `pnpm check`
Expected: 0 errors, 0 warnings

---

## Task 3: Home route repo-entry surface

**Tier:** worker

**Files:**
- Modify: `src/routes/+page.svelte` (full replacement — current file is spike code)

**Handoff Contract:**
- **Produces:** Product home surface with picker, path input, drag-drop, and recents list
- **Consumed By:** End user (visual/functional)

### Step 1: Replace the home route

**Replace `src/routes/+page.svelte`** entirely with the following. The current 521-line spike surface is replaced with a focused repo-entry home page. Graph components (`src/lib/graph/`) are untouched — they'll be used in the repo workspace later.

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { Button, Input } from '$lib/components/ui';

	let manualPath = $state('');
	let dragOver = $state(false);

	onMount(() => {
		repoStore.loadRecentRepos();

		let unlisten: (() => void) | undefined;

		async function setupDragDrop() {
			try {
				const { getCurrentWebviewWindow } = await import(
					'@tauri-apps/api/webviewWindow'
				);
				const webview = getCurrentWebviewWindow();
				unlisten = await webview.onDragDropEvent((event) => {
					if (event.payload.type === 'hover') {
						dragOver = true;
					} else if (event.payload.type === 'drop') {
						dragOver = false;
						const paths = event.payload.paths;
						if (paths.length > 0) {
							repoStore.openRepo(paths[0]);
						}
					} else if (event.payload.type === 'cancel') {
						dragOver = false;
					}
				});
			} catch {
				// Drag-drop unavailable outside Tauri — silently skip
			}
		}

		setupDragDrop();

		return () => {
			unlisten?.();
		};
	});

	function handleManualOpen() {
		if (manualPath.trim()) {
			repoStore.openRepo(manualPath.trim());
			manualPath = '';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			handleManualOpen();
		}
	}

	function formatDate(timestamp: number): string {
		return new Date(timestamp * 1000).toLocaleDateString(undefined, {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
		});
	}
</script>

<main class="home" class:drag-over={dragOver}>
	<div class="home-content">
		<!-- Branding -->
		<header class="home-header">
			<svg
				class="home-logo"
				width="40"
				height="40"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<circle cx="12" cy="12" r="3" />
				<path d="M12 3v6m0 6v6" />
				<circle cx="6" cy="18" r="2" />
				<circle cx="18" cy="6" r="2" />
				<path d="M6 16v-3a3 3 0 0 1 3-3h6a3 3 0 0 1 3 3v-3" />
			</svg>
			<h1 class="home-title">mongit</h1>
			<p class="home-subtitle">Git client for macOS</p>
		</header>

		<!-- Open Repository Section -->
		<section class="open-section">
			<div class="open-actions">
				<Button
					variant="primary"
					size="prominent"
					onclick={() => repoStore.openFolderPicker()}
					disabled={repoStore.loading}
				>
					Open Repository…
				</Button>

				<div class="separator">
					<span class="separator-text">or enter path</span>
				</div>

				<div class="path-row">
					<Input
						bind:value={manualPath}
						placeholder="/path/to/repository"
						mono
						onkeydown={handleKeydown}
						disabled={repoStore.loading}
					/>
					<Button
						variant="secondary"
						onclick={handleManualOpen}
						disabled={repoStore.loading || !manualPath.trim()}
					>
						Open
					</Button>
				</div>
			</div>

			<p class="drag-hint">
				{#if dragOver}
					Drop to open repository
				{:else}
					You can also drag and drop a folder here
				{/if}
			</p>
		</section>

		<!-- Error -->
		{#if repoStore.error}
			<div class="error-banner" role="alert">
				<span class="error-text">{repoStore.error}</span>
				<button class="error-dismiss" onclick={() => repoStore.clearError()}>
					✕
				</button>
			</div>
		{/if}

		<!-- Recent Repositories -->
		{#if repoStore.recentRepos.length > 0}
			<section class="recents-section">
				<h2 class="section-title">Recent</h2>
				<ul class="recents-list">
					{#each repoStore.recentRepos as repo (repo.path)}
						<li class="recent-item" class:stale={!repo.valid}>
							<button
								class="recent-button"
								onclick={() =>
									repo.valid
										? repoStore.openRepo(repo.path)
										: repoStore.retryRecentRepo(repo.path)}
								disabled={repoStore.loading}
							>
								<div class="recent-info">
									<span class="recent-name">{repo.name}</span>
									<span class="recent-path">{repo.path}</span>
								</div>
								<div class="recent-meta">
									{#if !repo.valid}
										<span class="stale-badge">Not found</span>
									{/if}
									<span class="recent-date"
										>{formatDate(repo.last_accessed)}</span
									>
								</div>
							</button>
							{#if !repo.valid}
								<button
									class="remove-btn"
									onclick={(e) => {
										e.stopPropagation();
										repoStore.removeRecentRepo(repo.path);
									}}
									title="Remove from recents"
								>
									✕
								</button>
							{/if}
						</li>
					{/each}
				</ul>
			</section>
		{/if}
	</div>

	<!-- Loading overlay -->
	{#if repoStore.loading}
		<div class="loading-overlay">
			<div class="spinner"></div>
			<p>Opening repository…</p>
		</div>
	{/if}

	<!-- Drag-drop overlay -->
	{#if dragOver}
		<div class="drop-overlay">
			<div class="drop-icon">
				<svg
					width="48"
					height="48"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path
						d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"
					/>
				</svg>
			</div>
			<p>Drop folder to open</p>
		</div>
	{/if}
</main>

<style>
	.home {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100vh;
		background: var(--color-bg);
		color: var(--color-text-primary);
		position: relative;
		-webkit-app-region: drag;
	}

	.home-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-8);
		max-width: 480px;
		width: 100%;
		padding: var(--space-8);
		-webkit-app-region: no-drag;
	}

	/* ── Header ─────────────────────────── */

	.home-header {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-3);
	}

	.home-logo {
		color: var(--color-accent);
		opacity: 0.8;
	}

	.home-title {
		font-family: var(--font-display);
		font-size: 28px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
		letter-spacing: -0.5px;
	}

	.home-subtitle {
		font-size: var(--text-body-sm-size);
		color: var(--color-text-muted);
		margin: 0;
	}

	/* ── Open section ───────────────────── */

	.open-section {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-5);
		width: 100%;
	}

	.open-actions {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-4);
		width: 100%;
	}

	.separator {
		display: flex;
		align-items: center;
		width: 100%;
		gap: var(--space-4);
	}

	.separator::before,
	.separator::after {
		content: '';
		flex: 1;
		height: 1px;
		background: var(--color-border);
	}

	.separator-text {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.path-row {
		display: flex;
		gap: var(--space-3);
		width: 100%;
	}

	.path-row :global(input) {
		flex: 1;
	}

	.drag-hint {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		margin: 0;
	}

	/* ── Error ──────────────────────────── */

	.error-banner {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		width: 100%;
		padding: var(--space-4) var(--space-5);
		background: var(--color-danger-muted);
		border: 1px solid color-mix(in srgb, var(--color-danger) 30%, transparent);
		border-radius: var(--radius-md);
	}

	.error-text {
		flex: 1;
		font-size: var(--text-body-sm-size);
		color: var(--color-danger);
		word-break: break-word;
	}

	.error-dismiss {
		background: none;
		border: none;
		color: var(--color-danger);
		cursor: pointer;
		padding: var(--space-1);
		font-size: 14px;
		opacity: 0.7;
		flex-shrink: 0;
	}

	.error-dismiss:hover {
		opacity: 1;
	}

	/* ── Recents ────────────────────────── */

	.recents-section {
		width: 100%;
	}

	.section-title {
		font-size: var(--text-body-sm-size);
		font-weight: 600;
		color: var(--color-text-secondary);
		margin: 0 0 var(--space-3);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.recents-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
		background: var(--color-border);
		border-radius: var(--radius-md);
		overflow: hidden;
	}

	.recent-item {
		display: flex;
		align-items: center;
		background: var(--color-bg-surface);
	}

	.recent-button {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-4) var(--space-5);
		background: none;
		border: none;
		cursor: pointer;
		text-align: left;
		color: var(--color-text-primary);
		transition: background var(--transition-fast);
		min-width: 0;
	}

	.recent-button:hover:not(:disabled) {
		background: var(--color-bg-hover);
	}

	.recent-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.recent-info {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		min-width: 0;
	}

	.recent-name {
		font-size: var(--text-body-size);
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.recent-path {
		font-family: var(--font-mono);
		font-size: var(--text-mono-xs-size);
		color: var(--color-text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.recent-meta {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-shrink: 0;
	}

	.recent-date {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.stale-badge {
		font-size: var(--text-caption-size);
		color: var(--color-warning);
		padding: var(--space-1) var(--space-3);
		background: var(--color-warning-muted);
		border-radius: var(--radius-sm);
		white-space: nowrap;
	}

	.recent-item.stale .recent-name {
		color: var(--color-text-secondary);
	}

	.remove-btn {
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		padding: var(--space-3);
		font-size: 14px;
		flex-shrink: 0;
		opacity: 0.5;
		transition: opacity var(--transition-fast);
	}

	.remove-btn:hover {
		opacity: 1;
		color: var(--color-danger);
	}

	/* ── Loading overlay ────────────────── */

	.loading-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-4);
		background: rgba(15, 17, 23, 0.8);
		color: var(--color-text-secondary);
		font-size: 13px;
		z-index: 10;
	}

	.spinner {
		width: 24px;
		height: 24px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* ── Drop overlay ───────────────────── */

	.drop-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-4);
		background: rgba(15, 17, 23, 0.9);
		border: 2px dashed var(--color-accent);
		border-radius: var(--radius-lg);
		margin: var(--space-4);
		color: var(--color-accent);
		font-size: 16px;
		font-weight: 500;
		z-index: 20;
	}

	.drop-icon {
		opacity: 0.8;
	}

	.home.drag-over {
		/* Subtle visual hint when dragging */
	}
</style>
```

### Step 2: Verify

Run: `pnpm check`
Expected: 0 errors, 0 warnings

---

## Task 4: Repo route shell handoff

**Tier:** worker  
**Parallel with:** Task 3 (no file overlap)

**Files:**
- Create: `src/routes/repo/+layout.svelte`
- Create: `src/routes/repo/+page.svelte`

**Handoff Contract:**
- **Produces:** Stable `/repo` route boundary for workspace navigation
- **Consumed By:** bd-6ew.2 (summary widgets), future workspace features

### Step 1: Create the repo route layout

**Create `src/routes/repo/+layout.svelte`:**

This layout wraps all `/repo/*` routes, reads the active repo from the store, and redirects to home if no repo is loaded (guard pattern).

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { repoStore } from '$lib/stores/repo.svelte';

	let { children } = $props();

	onMount(() => {
		// Guard: redirect to home if no repo is active
		if (!repoStore.activeRepoPath) {
			goto('/');
		}
	});
</script>

{#if repoStore.activeRepoPath}
	<div class="repo-shell">
		<header class="repo-toolbar">
			<div class="repo-toolbar-left">
				<button class="back-btn" onclick={() => goto('/')} title="Back to home">
					<svg
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<path d="M19 12H5M12 19l-7-7 7-7" />
					</svg>
				</button>
				<h1 class="repo-name">{repoStore.activeRepoName}</h1>
				{#if repoStore.repoStatus?.branch}
					<span class="branch-label">{repoStore.repoStatus.branch}</span>
				{/if}
			</div>
		</header>

		<div class="repo-content">
			{@render children()}
		</div>
	</div>
{:else}
	<div class="repo-guard">
		<p>No repository loaded.</p>
		<button onclick={() => goto('/')}>Go to Home</button>
	</div>
{/if}

<style>
	.repo-shell {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--color-bg);
		color: var(--color-text-primary);
	}

	.repo-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-6);
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
		-webkit-app-region: drag;
	}

	.repo-toolbar-left {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		-webkit-app-region: no-drag;
	}

	.back-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		background: none;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-secondary);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.back-btn:hover {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.repo-name {
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 700;
		color: var(--color-accent);
		margin: 0;
	}

	.branch-label {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		color: var(--color-text-secondary);
		padding: var(--space-1) var(--space-3);
		background: var(--color-bg-elevated);
		border-radius: var(--radius-sm);
	}

	.repo-content {
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.repo-guard {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100vh;
		gap: var(--space-4);
		color: var(--color-text-muted);
	}

	.repo-guard button {
		background: var(--color-accent);
		color: white;
		border: none;
		padding: var(--space-3) var(--space-5);
		border-radius: var(--radius-sm);
		cursor: pointer;
		font-size: var(--text-body-sm-size);
	}
</style>
```

### Step 2: Create the repo default page

**Create `src/routes/repo/+page.svelte`:**

Minimal placeholder page. The summary widgets will be added by bd-6ew.2.

```svelte
<script lang="ts">
	import { repoStore } from '$lib/stores/repo.svelte';
</script>

<div class="repo-landing">
	<div class="repo-summary">
		<h2>Repository opened</h2>

		{#if repoStore.repoStatus}
			<div class="status-grid">
				<div class="status-item">
					<span class="status-label">Branch</span>
					<span class="status-value mono">
						{repoStore.repoStatus.branch ?? 'detached'}
					</span>
				</div>
				<div class="status-item">
					<span class="status-label">Changed files</span>
					<span class="status-value">{repoStore.repoStatus.changed_files}</span>
				</div>
				<div class="status-item">
					<span class="status-label">Staged files</span>
					<span class="status-value">{repoStore.repoStatus.staged_files}</span>
				</div>
			</div>
		{/if}

		<p class="placeholder-note">
			Workspace features will be added in upcoming updates.
		</p>
	</div>
</div>

<style>
	.repo-landing {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		padding: var(--space-8);
	}

	.repo-summary {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-6);
		max-width: 400px;
		text-align: center;
	}

	.repo-summary h2 {
		font-size: 18px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0;
	}

	.status-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-4);
		width: 100%;
	}

	.status-item {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		padding: var(--space-4);
		background: var(--color-bg-surface);
		border-radius: var(--radius-md);
		border: 1px solid var(--color-border);
	}

	.status-label {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.status-value {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.status-value.mono {
		font-family: var(--font-mono);
		font-size: var(--text-mono-size);
	}

	.placeholder-note {
		font-size: var(--text-body-sm-size);
		color: var(--color-text-muted);
		margin: 0;
	}
</style>
```

### Step 3: Verify

Run: `pnpm check`
Expected: 0 errors, 0 warnings

---

## Final Verification

After all tasks complete, run full verification:

```bash
# Frontend typecheck
pnpm check

# Frontend build
pnpm build

# Backend typecheck
cd src-tauri && cargo check

# Backend tests
cd src-tauri && cargo test
```

All must pass with 0 errors.
