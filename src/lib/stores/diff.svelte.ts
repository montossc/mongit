import { invoke } from "@tauri-apps/api/core";

// ── Types matching Rust serialization ────────────────────────────────────

export interface DiffLineInfo {
	origin: string; // ' ' | '+' | '-' | '\\'
	content: string;
	old_lineno: number | null;
	new_lineno: number | null;
}

export interface DiffHunkInfo {
	old_start: number;
	old_lines: number;
	new_start: number;
	new_lines: number;
	header: string;
	lines: DiffLineInfo[];
}

export type DiffFileStatus = "Added" | "Modified" | "Deleted" | "Renamed";

export interface DiffFileEntry {
	path: string;
	status: DiffFileStatus;
	hunks: DiffHunkInfo[];
}

export interface FileContentPair {
	original: string;
	modified: string;
}

// ── Store ────────────────────────────────────────────────────────────────

function createDiffStore() {
	let files = $state<DiffFileEntry[]>([]);
	let selectedPath = $state<string | null>(null);
	let content = $state<FileContentPair | null>(null);
	let loading = $state(false);
	let loadingContent = $state(false);
	let error = $state<string | null>(null);
	let repoPath = $state("");
	let diffRequestId = 0; // Guard against stale repo-level diff responses
	let contentRequestId = 0; // Guard against out-of-order async responses

	/** Fetch the list of changed files for a repository. */
	async function fetchDiff(path: string): Promise<boolean> {
		diffRequestId += 1;
		const thisRequest = diffRequestId;
		loading = true;
		error = null;
		repoPath = path;

		try {
			const nextFiles = await invoke<DiffFileEntry[]>("get_diff_workdir", { path });
			if (thisRequest !== diffRequestId || repoPath !== path) {
				return false;
			}

			files = nextFiles;

			if (files.length > 0) {
				const stillValid =
					selectedPath && files.some((f) => f.path === selectedPath);
				if (!stillValid) {
					await selectFile(files[0].path);
				} else {
					await fetchContent(selectedPath!);
				}
			} else {
				selectedPath = null;
				content = null;
			}

			return true;
		} catch (e) {
			if (thisRequest === diffRequestId && repoPath === path) {
				error = String(e);
				files = [];
				selectedPath = null;
				content = null;
			}
			return false;
		} finally {
			if (thisRequest === diffRequestId) {
				loading = false;
			}
		}
	}

	/** Select a file and fetch its full content for diff rendering. */
	async function selectFile(path: string): Promise<void> {
		selectedPath = path;
		await fetchContent(path);
	}

	/** Internal: fetch file content pair with race-condition guard. */
	async function fetchContent(filePath: string): Promise<void> {
		if (!repoPath) return;
		contentRequestId += 1;
		const thisRequest = contentRequestId;
		loadingContent = true;
		try {
			const result = await invoke<FileContentPair>("get_file_content_for_diff", {
				path: repoPath,
				filePath,
			});
			// Only apply if this is still the latest request
			if (thisRequest === contentRequestId) {
				content = result;
			}
		} catch (e) {
			if (thisRequest === contentRequestId) {
				error = String(e);
				content = null;
			}
		} finally {
			if (thisRequest === contentRequestId) {
				loadingContent = false;
			}
		}
	}

	/** Reset store to initial state. */
	function reset(): void {
		files = [];
		selectedPath = null;
		content = null;
		loading = false;
		loadingContent = false;
		error = null;
		repoPath = "";
	}

	/** Re-fetch diff for the current repo (no-op if no repo loaded or already loading). */
	async function refresh(): Promise<void> {
		if (!repoPath || loading) return;
		await fetchDiff(repoPath);
	}

	return {
		get files() {
			return files;
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
		get loadingContent() {
			return loadingContent;
		},
		get error() {
			return error;
		},
		get repoPath() {
			return repoPath;
		},
		fetchDiff,
		selectFile,
		refresh,
		reset,
	};
}

export const diffStore = createDiffStore();
