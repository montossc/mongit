/**
 * Repo store — Svelte 5 runes-based repository selection and recents management.
 *
 * Owns:
 *   - Active repo path/name/status
 *   - Recent repos list (loaded from Tauri app data)
 *   - Open lifecycle: validate → persist → hydrate status → navigate
 *   - Error and loading state
 */

import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";

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
			recentRepos = await invoke<RecentRepo[]>("get_recent_repos");
		} catch (e) {
			// Non-fatal: log but don't block UI
			console.error("Failed to load recent repos:", e);
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
			const entry = await invoke<RecentRepo>("open_repo", { path: trimmed });

			// Hydrate repo status using existing command
			const status = await invoke<RepoStatus>("get_repo_status", {
				path: entry.path,
			});

			// Commit to store state
			activeRepoPath = entry.path;
			activeRepoName = entry.name;
			repoStatus = status;

			// Refresh recents list
			await loadRecentRepos();

			// Navigate to workspace
			await goto("/repo");
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	/** Open native folder picker, then open the selected directory. */
	async function openFolderPicker(): Promise<void> {
		try {
			const { open } = await import("@tauri-apps/plugin-dialog");
			const selected = await open({
				directory: true,
				multiple: false,
				title: "Select Git Repository",
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
			recentRepos = await invoke<RecentRepo[]>("remove_recent_repo", { path });
		} catch (e) {
			console.error("Failed to remove recent repo:", e);
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
