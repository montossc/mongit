/**
 * Toast notification store — Svelte 5 runes-based.
 *
 * Provides a stack of toast messages with auto-dismiss.
 * Used by CommandPalette (error feedback) and available app-wide.
 */

export type ToastVariant = "success" | "error" | "info";

export interface Toast {
	id: number;
	message: string;
	variant: ToastVariant;
}

// ── State ────────────────────────────────────────────────────────────────

let toasts = $state<Toast[]>([]);
let nextId = 0;

const AUTO_DISMISS_MS = 3000;

// ── Public API ───────────────────────────────────────────────────────────

/** Add a toast notification. Returns the toast ID for manual dismissal. */
function addToast(message: string, variant: ToastVariant = "info"): number {
	const id = nextId++;
	toasts = [...toasts, { id, message, variant }];

	// Auto-dismiss after timeout
	setTimeout(() => {
		dismiss(id);
	}, AUTO_DISMISS_MS);

	return id;
}

/** Dismiss a specific toast by ID. */
function dismiss(id: number): void {
	toasts = toasts.filter((t) => t.id !== id);
}

/** Convenience: show an error toast. */
function error(message: string): number {
	return addToast(message, "error");
}

/** Convenience: show a success toast. */
function success(message: string): number {
	return addToast(message, "success");
}

/** Convenience: show an info toast. */
function info(message: string): number {
	return addToast(message, "info");
}

export const toastStore = {
	get toasts() {
		return toasts;
	},
	addToast,
	dismiss,
	error,
	success,
	info,
};
