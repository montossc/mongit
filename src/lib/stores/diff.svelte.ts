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
	let stagedFiles = $state<DiffFileEntry[]>([]);
	let selectedPath = $state<string | null>(null);
	let content = $state<FileContentPair | null>(null);
	let loading = $state(false);
	let loadingContent = $state(false);
	let error = $state<string | null>(null);
	let repoPath = $state("");
	let diffRequestId = 0; // Guard against stale repo-level diff responses
	let contentRequestId = 0; // Guard against out-of-order async responses

	// Staging mutation state
	let staging = $state(false);
	let stagingError = $state<string | null>(null);

	/** Fetch both unstaged and staged diffs for a repository. */
	async function fetchDiff(path: string): Promise<boolean> {
		diffRequestId += 1;
		const thisRequest = diffRequestId;
		loading = true;
		error = null;
		repoPath = path;

		try {
			const [nextFiles, nextStaged] = await Promise.all([
				invoke<DiffFileEntry[]>("get_diff_workdir", { path }),
				invoke<DiffFileEntry[]>("get_diff_index", { path }),
			]);
			if (thisRequest !== diffRequestId || repoPath !== path) {
				return false;
			}

			files = nextFiles;
			stagedFiles = nextStaged;

			// Validate selection against both unstaged and staged files
			const allPaths = new Set([
				...nextFiles.map((f) => f.path),
				...nextStaged.map((f) => f.path),
			]);

			if (allPaths.size > 0) {
				const stillValid = selectedPath && allPaths.has(selectedPath);
				if (!stillValid) {
					const firstPath =
						nextFiles[0]?.path ?? nextStaged[0]?.path ?? null;
					if (firstPath) {
						await selectFile(firstPath);
					} else {
						selectedPath = null;
						content = null;
					}
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
				stagedFiles = [];
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
			const result = await invoke<FileContentPair>(
				"get_file_content_for_diff",
				{
					path: repoPath,
					filePath,
				},
			);
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

	/** Stage a single hunk. Returns true on success. */
	async function stageHunk(
		filePath: string,
		hunkIndex: number,
	): Promise<boolean> {
		if (staging || !repoPath) return false;
		staging = true;
		stagingError = null;
		try {
			await invoke("stage_hunk", {
				path: repoPath,
				filePath,
				hunkIndex,
			});
			return true;
		} catch (e) {
			stagingError = String(e);
			return false;
		} finally {
			staging = false;
		}
	}

	/** Unstage a single hunk. Returns true on success. */
	async function unstageHunk(
		filePath: string,
		hunkIndex: number,
	): Promise<boolean> {
		if (staging || !repoPath) return false;
		staging = true;
		stagingError = null;
		try {
			await invoke("unstage_hunk", {
				path: repoPath,
				filePath,
				hunkIndex,
			});
			return true;
		} catch (e) {
			stagingError = String(e);
			return false;
		} finally {
			staging = false;
		}
	}

	/** Reset store to initial state. */
	function reset(): void {
		files = [];
		stagedFiles = [];
		selectedPath = null;
		content = null;
		loading = false;
		loadingContent = false;
		error = null;
		staging = false;
		stagingError = null;
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
		get stagedFiles() {
			return stagedFiles;
		},
		get selectedPath() {
			return selectedPath;
		},
		/** Unstaged hunks for the selected file. */
		get selectedFileUnstagedHunks(): DiffHunkInfo[] {
			if (!selectedPath) return [];
			return files.find((f) => f.path === selectedPath)?.hunks ?? [];
		},
		/** Staged hunks for the selected file. */
		get selectedFileStagedHunks(): DiffHunkInfo[] {
			if (!selectedPath) return [];
			return (
				stagedFiles.find((f) => f.path === selectedPath)?.hunks ?? []
			);
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
		get staging() {
			return staging;
		},
		get stagingError() {
			return stagingError;
		},
		get repoPath() {
			return repoPath;
		},
		fetchDiff,
		selectFile,
		stageHunk,
		unstageHunk,
		refresh,
		reset,
	};
}

export const diffStore = createDiffStore();
