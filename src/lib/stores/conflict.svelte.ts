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
	let resolvedPaths = $state<Set<string>>(new Set());
	let resolving = $state(false);

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

	/**
	 * Resolve a conflict file: write content to working tree and stage in index.
	 */
	async function resolveFile(filePath: string, resolvedContent: string): Promise<void> {
		if (!repoPath) return;
		resolving = true;
		error = null;

		try {
			await invoke("resolve_conflict", {
				path: repoPath,
				filePath,
				content: resolvedContent,
			});
			resolvedPaths = new Set([...resolvedPaths, filePath]);
			// Refresh merge state to update conflict list
			await loadMergeState(repoPath);
		} catch (e) {
			error = String(e);
		} finally {
			resolving = false;
		}
	}

	/**
	 * Abort the current merge and reset to pre-merge state.
	 */
	async function abortMerge(): Promise<void> {
		if (!repoPath) return;
		loading = true;
		error = null;

		try {
			await invoke("abort_merge", { path: repoPath });
			reset();
			// Reload to confirm merge is gone
			await loadMergeState(repoPath);
		} catch (e) {
			error = String(e);
			loading = false;
		}
	}

	/**
	 * Complete the merge by creating a merge commit.
	 * Returns the new commit SHA.
	 */
	async function completeMerge(message?: string): Promise<string | null> {
		if (!repoPath) return null;
		loading = true;
		error = null;

		try {
			const sha = await invoke<string>("complete_merge", {
				path: repoPath,
				message: message ?? null,
			});
			reset();
			// Reload to confirm merge is done
			await loadMergeState(repoPath);
			return sha;
		} catch (e) {
			error = String(e);
			loading = false;
			return null;
		}
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
		resolvedPaths = new Set();
		resolving = false;
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
		get resolvedPaths() {
			return resolvedPaths;
		},
		get resolvedCount() {
			return resolvedPaths.size;
		},
		get resolving() {
			return resolving;
		},
		loadMergeState,
		loadConflictContent,
		selectFile,
		refresh,
		resolveFile,
		abortMerge,
		completeMerge,
		reset,
	};
}

export const conflictStore = createConflictStore();
