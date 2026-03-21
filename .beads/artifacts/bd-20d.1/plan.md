# Changed Files Model & Workspace List — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use skill({ name: "executing-plans" }) to implement this plan task-by-task.

**Goal:** Build the production changed-files data model (dual-state per file) and file list UI for the `/repo/changes` workspace route.

**Architecture:** New `ChangedFileEntry` struct with separate `staged` and `unstaged` status per file, backed by `git2::Repository::statuses()`. New `changesStore` (separate from `diffStore`) for lightweight file-list metadata + selection. New `/repo/changes` route with file list, status badges, and empty/loading/error states. Layout updated with tab navigation (Summary / Changes).

**Tech Stack:** Rust (git2 0.20, tauri 2.0, serde, thiserror), Svelte 5 (runes, SvelteKit), TypeScript

---

## Must-Haves

**Goal:** Users can view and select changed files with accurate dual-state (staged/unstaged) status in a dedicated changes workspace.

### Observable Truths

1. User can navigate to `/repo/changes` and see a list of changed files from the active repo
2. Each file row shows whether changes are staged, unstaged, or both (partially staged)
3. User can select a file row and the selection persists across refreshes when the file still exists
4. Empty, loading, and error states are clear and trustworthy
5. Downstream staging work (bd-20d.2) can consume `changesStore.selectedFile` without changing the data contract

### Required Artifacts

| Artifact | Provides | Path |
|----------|----------|------|
| FileChangeKind enum | Shared change classification | `src-tauri/src/git/repository.rs` |
| ChangedFileEntry struct | Dual-state per-file model | `src-tauri/src/git/repository.rs` |
| changed_files() method | git2 statuses → ChangedFileEntry[] | `src-tauri/src/git/repository.rs` |
| get_changed_files command | IPC endpoint for frontend | `src-tauri/src/commands.rs` |
| changesStore | File list metadata + selection state | `src/lib/stores/changes.svelte.ts` |
| Changes route | File list UI surface | `src/routes/repo/changes/+page.svelte` |
| Layout nav tabs | Summary / Changes navigation | `src/routes/repo/+layout.svelte` |

### Key Links

| From | To | Via | Risk |
|------|-----|-----|------|
| changesStore | get_changed_files | `invoke()` IPC | Type mismatch if Rust serialization differs from TS interface |
| Changes route | changesStore | import + reactivity | Selection state lost if store resets unexpectedly |
| Layout tabs | SvelteKit router | `goto()` / `$page.url` | Active tab detection if URL structure changes |

### Task Dependencies

```
Task 1 (Backend model): needs nothing, creates ChangedFileEntry + get_changed_files command
Task 2 (Frontend store): needs Task 1, creates changesStore
Task 3 (Route + UI): needs Task 2, creates /repo/changes route + layout tabs

Wave 1: Task 1 (backend)
Wave 2: Task 2 (frontend)
Wave 3: Task 3 (UI + integration)
```

---

## Task 1: Backend dual-state file model

**Files:**
- Modify: `src-tauri/src/git/repository.rs` (add types + trait method + impl)
- Modify: `src-tauri/src/commands.rs` (add get_changed_files command)
- Modify: `src-tauri/src/lib.rs` (register command)

### Step 1: Add FileChangeKind enum and ChangedFileEntry struct

Add these types after the existing `DiffFileStatus` enum (~line 21) in `src-tauri/src/git/repository.rs`:

```rust
/// Classification of how a single file changed.
/// Used for the dual-state changed-files model (staged + unstaged per row).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Typechange,
}

/// A changed file with separate staged and unstaged status.
/// This is the canonical row model for the changes workspace.
/// Both `staged` and `unstaged` are Option — at least one will be Some.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ChangedFileEntry {
    /// Relative path within the repo (forward-slash separated)
    pub path: String,
    /// Status in the index (staged changes), None if no staged changes
    pub staged: Option<FileChangeKind>,
    /// Status in the working tree (unstaged changes), None if no unstaged changes
    pub unstaged: Option<FileChangeKind>,
}
```

### Step 2: Add changed_files() to GitRepository trait

Add to the `GitRepository` trait (after `status()`, around line 97):

```rust
fn changed_files(&self) -> Result<Vec<ChangedFileEntry>, GitError>;
```

### Step 3: Implement changed_files() on Git2Repository

Add the implementation inside `impl GitRepository for Git2Repository` (after the `status()` method, around line 203):

```rust
fn changed_files(&self) -> Result<Vec<ChangedFileEntry>, GitError> {
    let repo = self.repo()?;
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_typechange(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    let mut entries = Vec::new();
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let bits = entry.status();

        let staged = if bits.intersects(git2::Status::INDEX_NEW) {
            Some(FileChangeKind::Added)
        } else if bits.intersects(git2::Status::INDEX_MODIFIED) {
            Some(FileChangeKind::Modified)
        } else if bits.intersects(git2::Status::INDEX_DELETED) {
            Some(FileChangeKind::Deleted)
        } else if bits.intersects(git2::Status::INDEX_RENAMED) {
            Some(FileChangeKind::Renamed)
        } else if bits.intersects(git2::Status::INDEX_TYPECHANGE) {
            Some(FileChangeKind::Typechange)
        } else {
            None
        };

        let unstaged = if bits.intersects(git2::Status::WT_NEW) {
            Some(FileChangeKind::Added)
        } else if bits.intersects(git2::Status::WT_MODIFIED) {
            Some(FileChangeKind::Modified)
        } else if bits.intersects(git2::Status::WT_DELETED) {
            Some(FileChangeKind::Deleted)
        } else if bits.intersects(git2::Status::WT_RENAMED) {
            Some(FileChangeKind::Renamed)
        } else if bits.intersects(git2::Status::WT_TYPECHANGE) {
            Some(FileChangeKind::Typechange)
        } else {
            None
        };

        if staged.is_some() || unstaged.is_some() {
            entries.push(ChangedFileEntry { path, staged, unstaged });
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}
```

### Step 4: Add get_changed_files command

Add to `src-tauri/src/commands.rs` (after the existing `get_diff_workdir` command):

```rust
#[tauri::command]
pub async fn get_changed_files(path: String) -> Result<Vec<ChangedFileEntry>, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Git2Repository::open(&path);
        repo.changed_files().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
```

Add the `ChangedFileEntry` import at the top of commands.rs:
```rust
use crate::git::repository::ChangedFileEntry;
```

### Step 5: Register command in lib.rs

Add `commands::get_changed_files` to the `tauri::generate_handler![]` macro in `src-tauri/src/lib.rs`.

### Step 6: Write backend tests

Add tests to `src-tauri/src/git/mod.rs` (in the `#[cfg(test)]` module), using the existing `create_test_repo` helper:

```rust
#[test]
fn test_changed_files_modified_file() {
    let (dir, bare_repo) = create_test_repo();
    let repo = Git2Repository::open(dir.path());

    // Modify the committed file without staging
    std::fs::write(dir.path().join("initial.txt"), "modified content").unwrap();

    let files = repo.changed_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "initial.txt");
    assert_eq!(files[0].staged, None);
    assert_eq!(files[0].unstaged, Some(FileChangeKind::Modified));
}

#[test]
fn test_changed_files_new_untracked_file() {
    let (dir, _) = create_test_repo();
    let repo = Git2Repository::open(dir.path());

    std::fs::write(dir.path().join("new.txt"), "new content").unwrap();

    let files = repo.changed_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "new.txt");
    assert_eq!(files[0].staged, None);
    assert_eq!(files[0].unstaged, Some(FileChangeKind::Added));
}

#[test]
fn test_changed_files_staged_only() {
    let (dir, _) = create_test_repo();
    let repo = Git2Repository::open(dir.path());

    // Create a new file and stage it
    std::fs::write(dir.path().join("staged.txt"), "staged content").unwrap();
    let bare = git2::Repository::open(dir.path()).unwrap();
    let mut index = bare.index().unwrap();
    index.add_path(std::path::Path::new("staged.txt")).unwrap();
    index.write().unwrap();

    let files = repo.changed_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "staged.txt");
    assert_eq!(files[0].staged, Some(FileChangeKind::Added));
    assert_eq!(files[0].unstaged, None);
}

#[test]
fn test_changed_files_partially_staged() {
    let (dir, _) = create_test_repo();
    let repo = Git2Repository::open(dir.path());

    // Stage a modification
    std::fs::write(dir.path().join("initial.txt"), "staged version").unwrap();
    let bare = git2::Repository::open(dir.path()).unwrap();
    let mut index = bare.index().unwrap();
    index.add_path(std::path::Path::new("initial.txt")).unwrap();
    index.write().unwrap();

    // Then modify again (unstaged)
    std::fs::write(dir.path().join("initial.txt"), "unstaged version").unwrap();

    let files = repo.changed_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "initial.txt");
    assert_eq!(files[0].staged, Some(FileChangeKind::Modified));
    assert_eq!(files[0].unstaged, Some(FileChangeKind::Modified));
}

#[test]
fn test_changed_files_deleted() {
    let (dir, _) = create_test_repo();
    let repo = Git2Repository::open(dir.path());

    std::fs::remove_file(dir.path().join("initial.txt")).unwrap();

    let files = repo.changed_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "initial.txt");
    assert_eq!(files[0].staged, None);
    assert_eq!(files[0].unstaged, Some(FileChangeKind::Deleted));
}

#[test]
fn test_changed_files_clean_repo() {
    let (dir, _) = create_test_repo();
    let repo = Git2Repository::open(dir.path());

    let files = repo.changed_files().unwrap();
    assert!(files.is_empty());
}

#[test]
fn test_changed_files_invalid_path() {
    let repo = Git2Repository::open("/nonexistent/path");
    assert!(repo.changed_files().is_err());
}

#[test]
fn test_changed_files_sorted_by_path() {
    let (dir, _) = create_test_repo();
    let repo = Git2Repository::open(dir.path());

    std::fs::write(dir.path().join("z-file.txt"), "z").unwrap();
    std::fs::write(dir.path().join("a-file.txt"), "a").unwrap();
    std::fs::write(dir.path().join("m-file.txt"), "m").unwrap();

    let files = repo.changed_files().unwrap();
    assert_eq!(files.len(), 3);
    assert_eq!(files[0].path, "a-file.txt");
    assert_eq!(files[1].path, "m-file.txt");
    assert_eq!(files[2].path, "z-file.txt");
}
```

### Step 7: Run verification

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Expected: All existing tests pass + 8 new tests pass. Zero errors from `cargo check`.

### Step 8: Commit

```bash
git add src-tauri/src/git/repository.rs src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/src/git/mod.rs
git commit -m "feat(bd-20d.1): add dual-state changed-files model and command

- Add FileChangeKind enum and ChangedFileEntry struct
- Implement changed_files() using git2 statuses API
- Add get_changed_files IPC command
- 8 backend tests covering all file states"
```

---

## Task 2: Workspace list state and selection store

**Files:**
- Create: `src/lib/stores/changes.svelte.ts`

**Depends on:** Task 1 (backend types must match)

### Step 1: Create the changes store

Create `src/lib/stores/changes.svelte.ts`:

```typescript
import { invoke } from "@tauri-apps/api/core";

// ── Types matching Rust ChangedFileEntry serialization ───────────────────

export type FileChangeKind =
	| "Added"
	| "Modified"
	| "Deleted"
	| "Renamed"
	| "Typechange";

export interface ChangedFileEntry {
	path: string;
	staged: FileChangeKind | null;
	unstaged: FileChangeKind | null;
}

// ── Store ────────────────────────────────────────────────────────────────

function createChangesStore() {
	let files = $state<ChangedFileEntry[]>([]);
	let selectedPath = $state<string | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let repoPath = $state("");
	let requestId = 0;

	/**
	 * Load changed files for a repository.
	 * Returns true if the response was applied (not stale).
	 */
	async function loadFiles(path: string): Promise<boolean> {
		requestId += 1;
		const thisRequest = requestId;
		loading = true;
		error = null;
		repoPath = path;

		try {
			const result = await invoke<ChangedFileEntry[]>("get_changed_files", {
				path,
			});

			// Guard: discard if a newer request was issued or repo changed
			if (thisRequest !== requestId || repoPath !== path) return false;

			files = result;

			// Validate/reassign selection
			if (files.length > 0) {
				const stillValid =
					selectedPath && files.some((f) => f.path === selectedPath);
				if (!stillValid) {
					selectedPath = files[0].path;
				}
			} else {
				selectedPath = null;
			}

			return true;
		} catch (e) {
			if (thisRequest === requestId && repoPath === path) {
				error = String(e);
				files = [];
				selectedPath = null;
			}
			return false;
		} finally {
			if (thisRequest === requestId) {
				loading = false;
			}
		}
	}

	/** Select a file by path. Does not trigger content loading (that's for downstream). */
	function selectFile(path: string): void {
		selectedPath = path;
	}

	/** Re-fetch files for the current repo. No-op if no repo or already loading. */
	async function refresh(): Promise<void> {
		if (!repoPath || loading) return;
		await loadFiles(repoPath);
	}

	/** Reset to initial state. */
	function reset(): void {
		files = [];
		selectedPath = null;
		loading = false;
		error = null;
		repoPath = "";
	}

	return {
		get files() {
			return files;
		},
		get selectedPath() {
			return selectedPath;
		},
		/** The full ChangedFileEntry for the selected file, or null. */
		get selectedFile() {
			return files.find((f) => f.path === selectedPath) ?? null;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		get repoPath() {
			return repoPath;
		},
		loadFiles,
		selectFile,
		refresh,
		reset,
	};
}

export const changesStore = createChangesStore();
```

### Step 2: Run verification

```bash
pnpm check
```

Expected: 0 errors, 0 warnings from svelte-check.

### Step 3: Commit

```bash
git add src/lib/stores/changes.svelte.ts
git commit -m "feat(bd-20d.1): add changes store with selection and race guards

- New changesStore separate from diffStore
- Lightweight file-list metadata (no hunk/diff content)
- Request-ID race guard pattern
- Selection auto-assigns on load, validates on refresh"
```

---

## Task 3: Repo changes workspace route and file list UI

**Files:**
- Create: `src/routes/repo/changes/+page.svelte`
- Modify: `src/routes/repo/+layout.svelte` (add tab navigation)
- Modify: `src/routes/repo/+page.svelte` (add link to changes)

**Depends on:** Task 2 (changesStore must exist)

### Step 1: Add tab navigation to repo layout

Modify `src/routes/repo/+layout.svelte` to add tab navigation between Summary and Changes. Add `page` import and a nav bar below the toolbar:

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { repoStore } from '$lib/stores/repo.svelte';

	let { children } = $props();

	onMount(() => {
		if (!repoStore.activeRepoPath) {
			goto('/');
		}
	});

	const tabs = [
		{ label: 'Summary', href: '/repo' },
		{ label: 'Changes', href: '/repo/changes' },
	] as const;
</script>

{#if repoStore.activeRepoPath}
	<div class="repo-shell">
		<header class="repo-toolbar">
			<div class="repo-toolbar-left">
				<button class="back-btn" onclick={() => goto('/')} title="Back to home">
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M19 12H5M12 19l-7-7 7-7" />
					</svg>
				</button>
				<h1 class="repo-name">{repoStore.activeRepoName}</h1>
				{#if repoStore.repoStatus?.branch}
					<span class="branch-label">{repoStore.repoStatus.branch}</span>
				{/if}
			</div>
		</header>

		<nav class="repo-tabs">
			{#each tabs as tab}
				<a
					href={tab.href}
					class="repo-tab"
					class:active={page.url.pathname === tab.href}
					onclick={(e) => { e.preventDefault(); goto(tab.href); }}
				>
					{tab.label}
				</a>
			{/each}
		</nav>

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

	.repo-tabs {
		display: flex;
		gap: 0;
		padding: 0 var(--space-6);
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	.repo-tab {
		padding: var(--space-3) var(--space-5);
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		color: var(--color-text-secondary);
		text-decoration: none;
		border-bottom: 2px solid transparent;
		transition: color var(--transition-fast), border-color var(--transition-fast);
		cursor: pointer;
	}

	.repo-tab:hover {
		color: var(--color-text-primary);
	}

	.repo-tab.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
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

### Step 2: Create the changes workspace route

Create `src/routes/repo/changes/+page.svelte`:

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { changesStore, type ChangedFileEntry, type FileChangeKind } from '$lib/stores/changes.svelte';
	import { listen } from '@tauri-apps/api/event';

	let unlisten: (() => void) | null = null;

	onMount(() => {
		// Load files when the route mounts
		if (repoStore.activeRepoPath) {
			changesStore.loadFiles(repoStore.activeRepoPath);
		}

		// Listen for file system changes and refresh
		const setupListener = async () => {
			unlisten = await listen('repo-changed', () => {
				changesStore.refresh();
			});
		};
		setupListener();

		return () => {
			if (unlisten) unlisten();
		};
	});

	/** Map a FileChangeKind to a short display label. */
	function kindLabel(kind: FileChangeKind): string {
		switch (kind) {
			case 'Added': return 'A';
			case 'Modified': return 'M';
			case 'Deleted': return 'D';
			case 'Renamed': return 'R';
			case 'Typechange': return 'T';
		}
	}

	/** Map a FileChangeKind to a CSS modifier class. */
	function kindClass(kind: FileChangeKind): string {
		switch (kind) {
			case 'Added': return 'added';
			case 'Modified': return 'modified';
			case 'Deleted': return 'deleted';
			case 'Renamed': return 'renamed';
			case 'Typechange': return 'typechange';
		}
	}

	/** Get the filename from a path. */
	function fileName(path: string): string {
		const parts = path.split('/');
		return parts[parts.length - 1];
	}

	/** Get the directory from a path, or empty string. */
	function fileDir(path: string): string {
		const lastSlash = path.lastIndexOf('/');
		return lastSlash > 0 ? path.substring(0, lastSlash + 1) : '';
	}
</script>

<div class="changes-workspace">
	{#if changesStore.loading && changesStore.files.length === 0}
		<!-- Loading state (only when no cached files) -->
		<div class="state-message">
			<div class="spinner"></div>
			<p>Loading changed files…</p>
		</div>

	{:else if changesStore.error}
		<!-- Error state -->
		<div class="state-message error">
			<p class="state-label">Error loading changes</p>
			<p class="state-detail">{changesStore.error}</p>
			<button class="retry-btn" onclick={() => changesStore.refresh()}>Retry</button>
		</div>

	{:else if changesStore.files.length === 0}
		<!-- Empty state: clean repo -->
		<div class="state-message">
			<p class="state-label">No changes</p>
			<p class="state-detail">Working tree is clean</p>
		</div>

	{:else}
		<!-- File list -->
		<div class="file-list" role="listbox" aria-label="Changed files">
			{#each changesStore.files as file (file.path)}
				<button
					class="file-row"
					class:selected={changesStore.selectedPath === file.path}
					role="option"
					aria-selected={changesStore.selectedPath === file.path}
					onclick={() => changesStore.selectFile(file.path)}
				>
					<span class="file-path">
						{#if fileDir(file.path)}
							<span class="file-dir">{fileDir(file.path)}</span>
						{/if}
						<span class="file-name">{fileName(file.path)}</span>
					</span>

					<span class="file-badges">
						{#if file.staged}
							<span class="status-badge staged {kindClass(file.staged)}" title="Staged: {file.staged}">
								{kindLabel(file.staged)}
							</span>
						{/if}
						{#if file.unstaged}
							<span class="status-badge unstaged {kindClass(file.unstaged)}" title="Unstaged: {file.unstaged}">
								{kindLabel(file.unstaged)}
							</span>
						{/if}
					</span>
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.changes-workspace {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* ── State messages ─────────────────────────────────────────────── */

	.state-message {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-3);
		color: var(--color-text-muted);
		padding: var(--space-8);
	}

	.state-message.error {
		color: var(--color-danger);
	}

	.state-label {
		font-size: var(--text-body-size);
		font-weight: 500;
		margin: 0;
	}

	.state-detail {
		font-size: var(--text-body-sm-size);
		margin: 0;
		max-width: 360px;
		text-align: center;
		word-break: break-word;
	}

	.state-message.error .state-detail {
		color: var(--color-text-secondary);
	}

	.retry-btn {
		margin-top: var(--space-3);
		padding: var(--space-2) var(--space-5);
		font-size: var(--text-body-sm-size);
		background: none;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-secondary);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.retry-btn:hover {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.spinner {
		width: 20px;
		height: 20px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	/* ── File list ──────────────────────────────────────────────────── */

	.file-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-2) 0;
	}

	.file-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		height: var(--size-row-default);
		padding: 0 var(--space-5);
		background: none;
		border: none;
		color: var(--color-text-primary);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		text-align: left;
		transition: background var(--transition-fast);
	}

	.file-row:hover {
		background: var(--color-bg-hover);
	}

	.file-row.selected {
		background: var(--color-bg-active);
	}

	.file-row:focus-visible {
		outline: var(--focus-ring-width) solid var(--focus-ring-color);
		outline-offset: calc(-1 * var(--focus-ring-width));
	}

	/* ── File path ──────────────────────────────────────────────────── */

	.file-path {
		display: flex;
		align-items: baseline;
		min-width: 0;
		overflow: hidden;
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
	}

	.file-dir {
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex-shrink: 1;
	}

	.file-name {
		color: var(--color-text-primary);
		white-space: nowrap;
		flex-shrink: 0;
	}

	/* ── Status badges ──────────────────────────────────────────────── */

	.file-badges {
		display: flex;
		gap: var(--space-1);
		flex-shrink: 0;
		margin-left: var(--space-3);
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		border-radius: var(--radius-xs, 3px);
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		line-height: 1;
	}

	/* Staged badges: filled background */
	.status-badge.staged.added     { background: var(--color-success); color: white; }
	.status-badge.staged.modified  { background: var(--color-info); color: white; }
	.status-badge.staged.deleted   { background: var(--color-danger); color: white; }
	.status-badge.staged.renamed   { background: var(--color-warning); color: white; }
	.status-badge.staged.typechange { background: var(--color-text-muted); color: white; }

	/* Unstaged badges: outline style */
	.status-badge.unstaged.added     { border: 1px solid var(--color-success); color: var(--color-success); }
	.status-badge.unstaged.modified  { border: 1px solid var(--color-info); color: var(--color-info); }
	.status-badge.unstaged.deleted   { border: 1px solid var(--color-danger); color: var(--color-danger); }
	.status-badge.unstaged.renamed   { border: 1px solid var(--color-warning); color: var(--color-warning); }
	.status-badge.unstaged.typechange { border: 1px solid var(--color-text-muted); color: var(--color-text-muted); }
</style>
```

### Step 3: Add "View changes" link to repo landing page

In `src/routes/repo/+page.svelte`, find the section that shows the changed/staged file counts (the state cards area) and add a link/button that navigates to `/repo/changes`. Add this inside the state-message or cards section:

Look for the existing changed files count display and add a "View changes" link after the state cards section. Add something like:

```svelte
{#if (repoStore.repoStatus?.changed_files ?? 0) > 0 || (repoStore.repoStatus?.staged_files ?? 0) > 0}
	<button class="view-changes-link" onclick={() => goto('/repo/changes')}>
		View changes →
	</button>
{/if}
```

With minimal styling:
```css
.view-changes-link {
	margin-top: var(--space-4);
	padding: var(--space-3) var(--space-5);
	background: none;
	border: 1px solid var(--color-border);
	border-radius: var(--radius-sm);
	color: var(--color-accent);
	font-size: var(--text-body-sm-size);
	cursor: pointer;
	transition: background var(--transition-fast);
}

.view-changes-link:hover {
	background: var(--color-bg-hover);
}
```

### Step 4: Run verification

```bash
pnpm check
pnpm build
```

Expected: 0 errors from svelte-check. Build succeeds.

### Step 5: Commit

```bash
git add src/routes/repo/changes/+page.svelte src/routes/repo/+layout.svelte src/routes/repo/+page.svelte
git commit -m "feat(bd-20d.1): add changes workspace route with file list UI

- New /repo/changes route with file list, status badges
- Tab navigation (Summary / Changes) in repo layout
- Loading, empty, error states with retry
- File selection with keyboard accessibility
- Status badges: filled for staged, outline for unstaged
- View changes link from repo landing page"
```

---

## Final Verification

Run all checks to confirm nothing is broken:

```bash
cd src-tauri && cargo check && cargo test
pnpm check
pnpm build
```

**Expected results:**
- `cargo check`: 0 errors
- `cargo test`: All tests pass (existing + 8 new)
- `pnpm check`: 0 errors
- `pnpm build`: Success

**Handoff contract for bd-20d.2:**
- `changesStore.selectedFile` → `ChangedFileEntry | null` — downstream can consume this to show diff + attach staging actions
- `changesStore.selectedPath` → `string | null` — file identity for diff loading
- `changesStore.files` → `ChangedFileEntry[]` — full file list for batch operations
- `/repo/changes` route structure is stable — staging UI attaches here, not on a new route
- `ChangedFileEntry.staged` and `.unstaged` fields are `Option<FileChangeKind>` — staging actions will mutate these states
