import { invoke } from "@tauri-apps/api/core";

// ── Types matching Rust serialization ────────────────────────────────────

export interface CommitResult {
	sha: string;
	summary: string;
}

export interface AuthorInfo {
	name: string;
	email: string;
}

/** Structured commit error from backend (discriminated union). */
interface CommitErrorPayload {
	kind: string;
	message?: string;
	hook?: string;
	stderr?: string;
}

// ── Store ────────────────────────────────────────────────────────────────

function createCommitStore() {
	let message = $state("");
	let amend = $state(false);
	let committing = $state(false);
	let error = $state<string | null>(null);
	let author = $state<AuthorInfo | null>(null);
	let lastCommit = $state<CommitResult | null>(null);

	/**
	 * Load author defaults from git config.
	 * Called once when the changes workspace mounts.
	 */
	async function loadAuthor(repoPath: string): Promise<void> {
		try {
			const result = await invoke<AuthorInfo>("get_commit_defaults", {
				path: repoPath,
			});
			author = result;
		} catch {
			// Non-critical — author display is optional
			author = null;
		}
	}

	/**
	 * Create a commit from staged changes.
	 * Returns true on success, false on error.
	 */
	async function commit(repoPath: string): Promise<boolean> {
		if (committing) return false;

		// Client-side validation
		if (message.trim() === "") {
			error = "Commit message cannot be empty";
			return false;
		}

		committing = true;
		error = null;

		try {
			const result = await invoke<CommitResult>("commit_changes", {
				path: repoPath,
				message: message.trim(),
				amend,
			});
			lastCommit = result;
			// Reset form on success
			message = "";
			amend = false;
			error = null;
			return true;
		} catch (e) {
			error = formatError(e);
			return false;
		} finally {
			committing = false;
		}
	}

	/**
	 * Load the HEAD commit message for amend pre-fill.
	 */
	async function loadHeadMessage(repoPath: string): Promise<void> {
		try {
			const headMsg = await invoke<string>("get_head_message", {
				path: repoPath,
			});
			message = headMsg;
		} catch {
			// If we can't get the head message, leave the field as-is
		}
	}

	/**
	 * Toggle amend mode. When toggling ON, pre-fills the message
	 * with the previous commit message.
	 */
	async function toggleAmend(repoPath: string): Promise<void> {
		amend = !amend;
		if (amend) {
			await loadHeadMessage(repoPath);
		}
	}

	/** Reset to initial state. */
	function reset(): void {
		message = "";
		amend = false;
		committing = false;
		error = null;
		lastCommit = null;
	}

	/** Set the commit message directly. */
	function setMessage(msg: string): void {
		message = msg;
	}

	return {
		get message() {
			return message;
		},
		get amend() {
			return amend;
		},
		get committing() {
			return committing;
		},
		get error() {
			return error;
		},
		get author() {
			return author;
		},
		get lastCommit() {
			return lastCommit;
		},
		commit,
		loadAuthor,
		toggleAmend,
		reset,
		setMessage,
	};
}

/**
 * Parse error from Tauri invoke into a user-friendly message.
 * Backend sends structured JSON for typed errors (CommitError).
 */
function formatError(e: unknown): string {
	const raw = String(e);
	try {
		const parsed: CommitErrorPayload = JSON.parse(raw);
		switch (parsed.kind) {
			case "NothingStaged":
				return "Nothing to commit — stage some changes first";
			case "EmptyMessage":
				return "Commit message cannot be empty";
			case "HookFailed":
				return `${parsed.hook ?? "Hook"} hook failed: ${parsed.message ?? ""}`.trim();
			case "MergeInProgress":
				return "Merge in progress — resolve conflicts first";
			case "AmendNoCommits":
				return "Cannot amend — no commits yet";
			case "GenericCommitFailed":
				return parsed.stderr ?? parsed.message ?? "Commit failed";
			default:
				return parsed.message ?? raw;
		}
	} catch {
		return raw;
	}
}

export const commitStore = createCommitStore();
