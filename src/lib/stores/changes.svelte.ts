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

		// Clear stale data when switching repos
		if (path !== repoPath) {
			files = [];
			selectedPath = null;
		}
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
