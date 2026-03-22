/**
 * Shortcut binding engine for mongit.
 *
 * Maps key combos to command IDs and dispatches through the command registry.
 * Suppresses shortcuts in text inputs, CodeMirror editors, and when the
 * command palette is open.
 */

import { commandRegistry } from "./registry.svelte";
import type { CommandContext } from "./types";

// ── Types ────────────────────────────────────────────────────────────────

/** A keyboard shortcut bound to a command. */
export interface ShortcutBinding {
	/** Key value (KeyboardEvent.key), case-insensitive. */
	key: string;
	/** Requires Meta/CMD key. */
	meta?: boolean;
	/** Requires Shift key. */
	shift?: boolean;
	/** Requires Alt/Option key. */
	alt?: boolean;
	/** Command ID to execute via registry, or PALETTE_TOGGLE_ID for palette toggle. */
	commandId: string;
	/** Display hint shown in palette (e.g., "⌘1", "⌘⇧P"). */
	hint: string;
}

/** Special command ID for toggling the command palette. */
export const PALETTE_TOGGLE_ID = "__palette_toggle__";

// ── Internal state ───────────────────────────────────────────────────────

let bindings: ShortcutBinding[] = [];
let keydownHandler: ((e: KeyboardEvent) => void) | null = null;
let contextGetter: (() => CommandContext) | null = null;

// Palette callbacks — set by CommandPalette via setPaletteCallbacks()
let isPaletteOpenFn: (() => boolean) | null = null;
let togglePaletteFn: (() => void) | null = null;

// ── Suppression ──────────────────────────────────────────────────────────

/** Returns true if the target is a text input element. */
function isTextInput(target: HTMLElement): boolean {
	if (
		target.tagName === "INPUT" ||
		target.tagName === "TEXTAREA" ||
		target.isContentEditable
	) {
		return true;
	}
	return false;
}

/** Returns true if the target is inside a CodeMirror editor. */
function isCodeMirror(target: HTMLElement): boolean {
	return !!target.closest(".cm-editor");
}

// ── Matching ─────────────────────────────────────────────────────────────

function matchesBinding(e: KeyboardEvent, binding: ShortcutBinding): boolean {
	if (e.key.toLowerCase() !== binding.key.toLowerCase()) return false;
	if (!!binding.meta !== e.metaKey) return false;
	if (!!binding.shift !== e.shiftKey) return false;
	if (!!binding.alt !== e.altKey) return false;
	return true;
}

// ── Core handler ─────────────────────────────────────────────────────────

function handleKeydown(e: KeyboardEvent): void {
	const target = e.target;
	if (!(target instanceof HTMLElement)) return;

	// Find matching binding
	const binding = bindings.find((b) => matchesBinding(e, b));
	if (!binding) return;

	// Palette toggle: only suppress in CodeMirror (allow in text inputs)
	if (binding.commandId === PALETTE_TOGGLE_ID) {
		if (isCodeMirror(target)) return;
		e.preventDefault();
		togglePaletteFn?.();
		return;
	}

	// All other shortcuts: suppress when palette is open
	if (isPaletteOpenFn?.()) return;

	// Suppress in text inputs and CodeMirror
	if (isTextInput(target) || isCodeMirror(target)) return;

	// Execute through registry (handles enabled check)
	const ctx = contextGetter?.();
	if (!ctx) return;

	e.preventDefault();
	commandRegistry.execute(binding.commandId, ctx).catch((err: unknown) => {
		// Import would create circular dependency; use console as fallback.
		// Toast feedback is handled by the palette path; shortcuts log to console.
		console.error(`[shortcut] Command "${binding.commandId}" failed:`, err);
	});
}

// ── Public API ───────────────────────────────────────────────────────────

/**
 * Connect the command palette to the shortcut manager.
 * Call from CommandPalette on mount to enable palette toggle and suppression.
 */
export function setPaletteCallbacks(
	isPaletteOpen: () => boolean,
	togglePalette: () => void,
): void {
	isPaletteOpenFn = isPaletteOpen;
	togglePaletteFn = togglePalette;
}

/**
 * Register keyboard shortcuts and attach the global keydown listener.
 * Also populates `shortcutHint` on matching commands in the registry.
 * Call once at app startup (typically in +layout.svelte), after registerBuiltinCommands().
 */
export function registerShortcuts(
	shortcutBindings: ShortcutBinding[],
	getContext: () => CommandContext,
): void {
	bindings = shortcutBindings;
	contextGetter = getContext;

	// Auto-populate shortcutHint on commands that have bindings
	for (const binding of bindings) {
		if (binding.commandId === PALETTE_TOGGLE_ID) continue;
		const cmd = commandRegistry.getById(binding.commandId);
		if (cmd) {
			cmd.shortcutHint = binding.hint;
		}
	}

	keydownHandler = handleKeydown;
	window.addEventListener("keydown", keydownHandler);
}

/** Remove the global keydown listener and clear all state. */
export function destroyShortcuts(): void {
	if (keydownHandler) {
		window.removeEventListener("keydown", keydownHandler);
		keydownHandler = null;
	}
	bindings = [];
	contextGetter = null;
	isPaletteOpenFn = null;
	togglePaletteFn = null;
}

/** Get the display hint for a command's shortcut binding, if any. */
export function getShortcutHint(commandId: string): string | undefined {
	return bindings.find((b) => b.commandId === commandId)?.hint;
}

// ── Core shortcut definitions ────────────────────────────────────────────

export const CORE_SHORTCUTS: ShortcutBinding[] = [
	// Palette toggle
	{ key: "k", meta: true, commandId: PALETTE_TOGGLE_ID, hint: "⌘K" },

	// Navigation
	{ key: "1", meta: true, commandId: "nav.summary", hint: "⌘1" },
	{ key: "2", meta: true, commandId: "nav.changes", hint: "⌘2" },
	{ key: "3", meta: true, commandId: "nav.history", hint: "⌘3" },

	// Git operations (CMD+Shift)
	{
		key: "p",
		meta: true,
		shift: true,
		commandId: "git.push",
		hint: "⌘⇧P",
	},
	{
		key: "f",
		meta: true,
		shift: true,
		commandId: "git.fetch",
		hint: "⌘⇧F",
	},
	{
		key: "u",
		meta: true,
		shift: true,
		commandId: "git.pull",
		hint: "⌘⇧U",
	},
	{
		key: "b",
		meta: true,
		shift: true,
		commandId: "git.create-branch",
		hint: "⌘⇧B",
	},
	{
		key: "r",
		meta: true,
		shift: true,
		commandId: "git.refresh-status",
		hint: "⌘⇧R",
	},

	// View
	{
		key: "t",
		meta: true,
		shift: true,
		commandId: "view.toggle-theme",
		hint: "⌘⇧T",
	},

	// General
	{ key: "o", meta: true, commandId: "general.open-repo", hint: "⌘O" },
];
