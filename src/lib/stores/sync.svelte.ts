import { invoke } from "@tauri-apps/api/core";

// ── Types matching Rust serialization ────────────────────────────────────

export interface AheadBehind {
	ahead: number;
	behind: number;
	upstream: string | null;
}

/** Possible sync operation types for status display. */
export type SyncOp = "fetch" | "pull" | "push";

/** Result of a successful sync operation. */
export interface SyncResult {
	op: SyncOp;
	output: string;
	timestamp: number;
}

/** Structured sync error from backend (BranchOpError discriminated union). */
interface SyncErrorPayload {
	kind: string;
	message?: string;
	count?: number;
	remote?: string;
	branch?: string;
	stderr?: string;
}

// ── Store ────────────────────────────────────────────────────────────────

function createSyncStore() {
	let fetching = $state(false);
	let pulling = $state(false);
	let pushing = $state(false);
	let error = $state<string | null>(null);
	let lastResult = $state<SyncResult | null>(null);
	let aheadBehind = $state<AheadBehind | null>(null);
	let _successTimer: ReturnType<typeof setTimeout> | null = null;

	/** Whether any sync operation is currently in progress. */
	const busy = $derived(fetching || pulling || pushing);

	/** Refresh ahead/behind tracking counts. */
	async function refreshAheadBehind(repoPath: string): Promise<void> {
		try {
			aheadBehind = await invoke<AheadBehind>("get_ahead_behind", {
				path: repoPath,
			});
		} catch {
			// Non-critical — tracking display is optional
			aheadBehind = null;
		}
	}

	/** Clear any displayed error. */
	function clearError(): void {
		error = null;
	}

	/** Clear the last result (used to dismiss success status). */
	function clearLastResult(): void {
		lastResult = null;
	}

	/**
	 * Show success status briefly, then auto-clear.
	 */
	function showSuccess(op: SyncOp, output: string): void {
		if (_successTimer) clearTimeout(_successTimer);
		lastResult = { op, output, timestamp: Date.now() };
		_successTimer = setTimeout(() => {
			lastResult = null;
			_successTimer = null;
		}, 3000);
	}

	/**
	 * Fetch latest refs and objects from origin.
	 */
	async function fetchOrigin(repoPath: string): Promise<boolean> {
		if (busy) return false;

		fetching = true;
		error = null;

		try {
			const output = await invoke<string>("fetch", { path: repoPath });
			showSuccess("fetch", output);
			// Refresh tracking info after fetch
			await refreshAheadBehind(repoPath);
			return true;
		} catch (e) {
			error = formatSyncError(e);
			return false;
		} finally {
			fetching = false;
		}
	}

	/**
	 * Pull changes from origin into the current branch.
	 */
	async function pullOrigin(repoPath: string): Promise<boolean> {
		if (busy) return false;

		pulling = true;
		error = null;

		try {
			const output = await invoke<string>("pull", { path: repoPath });
			showSuccess("pull", output);
			// Refresh tracking info after pull
			await refreshAheadBehind(repoPath);
			return true;
		} catch (e) {
			error = formatSyncError(e);
			return false;
		} finally {
			pulling = false;
		}
	}

	/**
	 * Push current branch to origin.
	 */
	async function pushOrigin(
		repoPath: string,
		forceWithLease: boolean = false,
	): Promise<boolean> {
		if (busy) return false;

		pushing = true;
		error = null;

		try {
			const output = await invoke<string>("push", {
				path: repoPath,
				forceWithLease,
			});
			showSuccess("push", output);
			// Refresh tracking info after push
			await refreshAheadBehind(repoPath);
			return true;
		} catch (e) {
			error = formatSyncError(e);
			return false;
		} finally {
			pushing = false;
		}
	}

	/** Reset to initial state. */
	function reset(): void {
		fetching = false;
		pulling = false;
		pushing = false;
		error = null;
		lastResult = null;
		aheadBehind = null;
		if (_successTimer) {
			clearTimeout(_successTimer);
			_successTimer = null;
		}
	}

	return {
		get fetching() {
			return fetching;
		},
		get pulling() {
			return pulling;
		},
		get pushing() {
			return pushing;
		},
		get busy() {
			return busy;
		},
		get error() {
			return error;
		},
		get lastResult() {
			return lastResult;
		},
		get aheadBehind() {
			return aheadBehind;
		},
		fetchOrigin,
		pullOrigin,
		pushOrigin,
		refreshAheadBehind,
		clearError,
		clearLastResult,
		reset,
	};
}

/**
 * Parse error from Tauri invoke into a user-friendly message.
 * Backend sends structured JSON for typed errors (BranchOpError).
 */
function formatSyncError(e: unknown): string {
	const raw = String(e);
	try {
		const parsed: SyncErrorPayload = JSON.parse(raw);
		switch (parsed.kind) {
			case "NetworkError":
				return "Network error \u2014 check your connection";
			case "AuthFailure":
				return "Authentication failed \u2014 check credentials";
			case "MergeConflicts":
				return `Pull created merge conflicts in ${parsed.count ?? "some"} file(s)`;
			case "BranchesDiverged":
				return "Branches have diverged \u2014 pull first, then push";
			case "NoUpstreamBranch":
				return "No upstream branch configured";
			case "PushNonFastForward":
				return "Push rejected \u2014 pull first to integrate remote changes";
			case "ProtectedBranch":
				return "Remote rejected push to protected branch";
			case "RemoteNotFound":
				return `Remote '${parsed.remote ?? "origin"}' not found`;
			case "GenericCommandFailed":
				return parsed.stderr ?? parsed.message ?? "Sync operation failed";
			default:
				return parsed.message ?? raw;
		}
	} catch {
		return raw;
	}
}

export const syncStore = createSyncStore();
