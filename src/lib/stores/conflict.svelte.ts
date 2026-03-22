import { invoke } from "@tauri-apps/api/core";

// ── Types matching Rust serialization ────────────────────────────────────

export interface ConflictFileEntry {
	path: string;
}

export interface MergeState {
	is_merging: boolean;
	incoming_sha: string | null;
	conflicted_files: ConflictFileEntry[];
}

export interface ConflictContent {
	file_path: string;
	base: string | null;
	ours: string;
	theirs: string;
}

// ── Store ────────────────────────────────────────────────────────────────

function createConflictStore() {
	let mergeState = $state<MergeState | null>(null);
	let selectedPath = $state<string | null>(null);
	let content = $state<ConflictContent | null>(null);
	let loading = $state(false);
	let contentLoading = $state(false);
	let error = $state<string | null>(null);
	let repoPath = $state("");

	/**
	 * Load merge state for a repository.
	 */
	async function loadMergeState(path: string): Promise<void> {
		loading = true;
		error = null;
		repoPath = path;

		try {
			const result = await invoke<MergeState>("get_merge_state", { path });
			mergeState = result;

			// Auto-select first conflicted file if none selected
			if (
				result.conflicted_files.length > 0 &&
				(!selectedPath ||
					!result.conflicted_files.some((f) => f.path === selectedPath))
			) {
				selectedPath = result.conflicted_files[0].path;
			} else if (result.conflicted_files.length === 0) {
				selectedPath = null;
				content = null;
			}
		} catch (e) {
			error = String(e);
			mergeState = null;
		} finally {
			loading = false;
		}
	}

	/**
	 * Load conflict content for a specific file.
	 */
	async function loadConflictContent(
		path: string,
		filePath: string,
	): Promise<void> {
		contentLoading = true;
		error = null;

		try {
			const result = await invoke<ConflictContent>("get_conflict_content", {
				path,
				filePath,
			});
			content = result;
			selectedPath = filePath;
		} catch (e) {
			error = String(e);
			content = null;
		} finally {
			contentLoading = false;
		}
	}

	/**
	 * Select a conflicted file and load its content.
	 */
	function selectFile(filePath: string): void {
		selectedPath = filePath;
		if (repoPath) {
			loadConflictContent(repoPath, filePath);
		}
	}

	/**
	 * Refresh merge state for the current repo.
	 */
	async function refresh(): Promise<void> {
		if (!repoPath || loading) return;
		await loadMergeState(repoPath);
	}

	/** Reset to initial state. */
	function reset(): void {
		mergeState = null;
		selectedPath = null;
		content = null;
		loading = false;
		contentLoading = false;
		error = null;
		repoPath = "";
	}

	return {
		get mergeState() {
			return mergeState;
		},
		get isMerging() {
			return mergeState?.is_merging ?? false;
		},
		get conflictedFiles() {
			return mergeState?.conflicted_files ?? [];
		},
		get conflictCount() {
			return mergeState?.conflicted_files.length ?? 0;
		},
		get selectedPath() {
			return selectedPath;
		},
		get content() {
			return content;
		},
		get loading() {
			return loading;
		},
		get contentLoading() {
			return contentLoading;
		},
		get error() {
			return error;
		},
		get repoPath() {
			return repoPath;
		},
		loadMergeState,
		loadConflictContent,
		selectFile,
		refresh,
		reset,
	};
}

export const conflictStore = createConflictStore();
