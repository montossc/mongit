/**
 * Command system type definitions for the command palette.
 *
 * These types define the contract for registering and executing
 * commands from a central, keyboard-first entry point.
 */

/** Categories for grouping commands in the palette. */
export type CommandCategory =
	| "navigation"
	| "git"
	| "staging"
	| "view"
	| "general";

/** Display labels for each category. */
export const CATEGORY_LABELS: Record<CommandCategory, string> = {
	navigation: "Navigation",
	git: "Git",
	staging: "Staging",
	view: "View",
	general: "General",
};

/** Display order for categories in the palette. */
export const CATEGORY_ORDER: CommandCategory[] = [
	"navigation",
	"git",
	"staging",
	"view",
	"general",
];

/** Context passed to command `enabled` and `execute` functions. */
export interface CommandContext {
	/** The active repository path, or null if no repo is open. */
	repoPath: string | null;
	/** The current SvelteKit route pathname. */
	currentRoute: string;
	/** Whether a repository is currently open. */
	hasRepo: boolean;
}

/** A registered command in the palette. */
export interface Command {
	/** Unique identifier (e.g., "git.fetch", "nav.changes"). */
	id: string;
	/** Human-readable label shown in the palette. */
	label: string;
	/** Category for grouping in results. */
	category: CommandCategory;
	/** Optional keyboard shortcut hint (display only — binding is separate). */
	shortcutHint?: string;
	/** Whether this command is currently available. Return false to hide from palette. */
	enabled: (ctx: CommandContext) => boolean;
	/** Execute the command action. */
	execute: (ctx: CommandContext) => void | Promise<void>;
}
