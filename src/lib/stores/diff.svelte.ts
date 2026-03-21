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

	// Line-level selection state
	// Key format: "unstaged:{hunkIndex}" or "staged:{hunkIndex}"
	let lineSelections = $state<Map<string, Set<number>>>(new Map());

	/** Fetch both unstaged and staged diffs for a repository. */
	async function fetchDiff(path: string): Promise<boolean> {
		diffRequestId += 1;
		const thisRequest = diffRequestId;
		loading = true;
		error = null;
		repoPath = path;
		clearLineSelection();

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

	/** Clear all line selections. */
	function clearLineSelection(): void {
		lineSelections = new Map();
	}

	/** Select a file and fetch its full content for diff rendering. */
	async function selectFile(path: string): Promise<void> {
		selectedPath = path;
		clearLineSelection();
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

	/** Stage selected lines from a hunk. Returns true on success. */
	async function stageLines(
		filePath: string,
		hunkIndex: number,
		lineIndices: number[],
	): Promise<boolean> {
		if (staging || !repoPath || lineIndices.length === 0) return false;
		staging = true;
		stagingError = null;
		try {
			await invoke("stage_lines", {
				path: repoPath,
				filePath,
				hunkIndex,
				lineIndices,
			});
			return true;
		} catch (e) {
			stagingError = String(e);
			return false;
		} finally {
			staging = false;
		}
	}

	/** Unstage selected lines from a hunk. Returns true on success. */
	async function unstageLines(
		filePath: string,
		hunkIndex: number,
		lineIndices: number[],
	): Promise<boolean> {
		if (staging || !repoPath || lineIndices.length === 0) return false;
		staging = true;
		stagingError = null;
		try {
			await invoke("unstage_lines", {
				path: repoPath,
				filePath,
				hunkIndex,
				lineIndices,
			});
			return true;
		} catch (e) {
			stagingError = String(e);
			return false;
		} finally {
			staging = false;
		}
	}

	/** Toggle selection of a line within a hunk. */
	function toggleLineSelection(
		side: "unstaged" | "staged",
		hunkIndex: number,
		lineIndex: number,
	): void {
		const key = `${side}:${hunkIndex}`;
		const next = new Map(lineSelections);
		const current = next.get(key);
		if (current) {
			const updated = new Set(current);
			if (updated.has(lineIndex)) {
				updated.delete(lineIndex);
			} else {
				updated.add(lineIndex);
			}
			if (updated.size === 0) {
				next.delete(key);
			} else {
				next.set(key, updated);
			}
		} else {
			next.set(key, new Set([lineIndex]));
		}
		lineSelections = next;
	}

	/** Get selected line indices for a hunk. */
	function getSelectedLines(
		side: "unstaged" | "staged",
		hunkIndex: number,
	): Set<number> {
		return lineSelections.get(`${side}:${hunkIndex}`) ?? new Set();
	}

	/** Get count of selected change lines for a hunk. */
	function getSelectedChangeCount(
		side: "unstaged" | "staged",
		hunkIndex: number,
		hunk: DiffHunkInfo,
	): number {
		const sel = getSelectedLines(side, hunkIndex);
		let count = 0;
		for (const idx of sel) {
			const line = hunk.lines[idx];
			if (line && (line.origin === '+' || line.origin === '-')) {
				count++;
			}
		}
		return count;
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
		lineSelections = new Map();
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
		get lineSelections() {
			return lineSelections;
		},
		fetchDiff,
		selectFile,
		stageHunk,
		unstageHunk,
		stageLines,
		unstageLines,
		toggleLineSelection,
		getSelectedLines,
		getSelectedChangeCount,
		clearLineSelection,
		refresh,
		reset,
	};
}

export const diffStore = createDiffStore();
