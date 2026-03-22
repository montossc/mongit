/**
 * Command registry — Svelte 5 runes-based singleton store.
 *
 * Holds a Map<string, Command> and provides reactive access
 * to the full command list, filtered results, and execution.
 * Features: fuzzy search with match indices, recently-used tracking.
 */

import {
	CATEGORY_ORDER,
	type Command,
	type CommandCategory,
	type CommandContext,
} from "./types";

// ── Types ────────────────────────────────────────────────────────────────

/** A command with fuzzy match metadata for UI highlighting. */
export interface MatchedCommand {
	cmd: Command;
	/** Indices of matched characters in the label (for highlighting). Empty if no query. */
	matchIndices: number[];
	/** Fuzzy match score (lower is better). 0 for exact/no-query matches. */
	score: number;
}

export interface SearchGroup {
	category: CommandCategory;
	commands: MatchedCommand[];
}

// ── Registry state ──────────────────────────────────────────────────────

const commands = $state(new Map<string, Command>());

// ── Recently-used tracking ──────────────────────────────────────────────

const RECENT_STORAGE_KEY = "mongit:recent-commands";
const RECENT_MAX = 20;

function loadRecentIds(): string[] {
	try {
		const raw = localStorage.getItem(RECENT_STORAGE_KEY);
		if (!raw) return [];
		const parsed = JSON.parse(raw);
		if (Array.isArray(parsed)) return parsed.filter((id): id is string => typeof id === "string");
		return [];
	} catch {
		return [];
	}
}

function saveRecentIds(ids: string[]): void {
	try {
		localStorage.setItem(RECENT_STORAGE_KEY, JSON.stringify(ids));
	} catch {
		// localStorage quota exceeded — silently ignore
	}
}

let recentIds = $state<string[]>(loadRecentIds());

function trackRecent(commandId: string): void {
	// Move to front, deduplicate, cap
	const updated = [commandId, ...recentIds.filter((id) => id !== commandId)].slice(0, RECENT_MAX);
	recentIds = updated;
	saveRecentIds(updated);
}

// ── Fuzzy matching ──────────────────────────────────────────────────────

/**
 * Character-walk fuzzy match. Returns match indices if all characters
 * in `needle` appear in `haystack` in order (case-insensitive).
 * Returns null if no match.
 */
function fuzzyMatch(needle: string, haystack: string): { indices: number[]; score: number } | null {
	const nLower = needle.toLowerCase();
	const hLower = haystack.toLowerCase();
	const indices: number[] = [];
	let hIdx = 0;

	for (let nIdx = 0; nIdx < nLower.length; nIdx++) {
		const char = nLower[nIdx];
		let found = false;
		while (hIdx < hLower.length) {
			if (hLower[hIdx] === char) {
				indices.push(hIdx);
				hIdx++;
				found = true;
				break;
			}
			hIdx++;
		}
		if (!found) return null;
	}

	// Score: prefer consecutive matches and matches at word boundaries.
	// Lower score = better match.
	let score = 0;
	for (let i = 1; i < indices.length; i++) {
		const gap = indices[i] - indices[i - 1] - 1;
		score += gap; // Penalize gaps between matched characters
	}
	// Penalize matches that start late
	score += indices.length > 0 ? indices[0] : 0;

	return { indices, score };
}

// ── Public API ──────────────────────────────────────────────────────────

/** Register a command. Overwrites if the ID already exists. */
function register(command: Command): void {
	commands.set(command.id, command);
}

/** Register multiple commands at once. */
function registerAll(cmds: Command[]): void {
	for (const cmd of cmds) {
		commands.set(cmd.id, cmd);
	}
}

/** Remove a command by ID. */
function deregister(id: string): void {
	commands.delete(id);
}

/** Get all registered commands as an array. */
function getAll(): Command[] {
	return Array.from(commands.values());
}

/** Get a command by ID. */
function getById(id: string): Command | undefined {
	return commands.get(id);
}

/**
 * Get commands filtered by fuzzy search query, grouped by category.
 * Only includes commands where `enabled(ctx)` returns true.
 * Results are ordered by category (CATEGORY_ORDER), then by match score.
 */
function search(
	query: string,
	ctx: CommandContext,
): SearchGroup[] {
	const needle = query.trim();

	// Filter: enabled + fuzzy matches query
	const matches: MatchedCommand[] = [];

	for (const cmd of getAll()) {
		if (!cmd.enabled(ctx)) continue;

		if (!needle) {
			// Empty query: show all enabled commands
			matches.push({ cmd, matchIndices: [], score: 0 });
			continue;
		}

		const result = fuzzyMatch(needle, cmd.label);
		if (result) {
			matches.push({ cmd, matchIndices: result.indices, score: result.score });
		}
	}

	// Group by category, respecting CATEGORY_ORDER
	const groups: SearchGroup[] = [];

	for (const cat of CATEGORY_ORDER) {
		const inCategory = matches
			.filter((m) => m.cmd.category === cat)
			.sort((a, b) => {
				if (needle) return a.score - b.score; // Best match first
				return a.cmd.label.localeCompare(b.cmd.label); // Alphabetical when no query
			});

		if (inCategory.length > 0) {
			groups.push({ category: cat, commands: inCategory });
		}
	}

	return groups;
}

/**
 * Get recently-used commands, filtered by enabled state.
 * Returns up to `limit` commands in MRU order.
 */
function getRecent(ctx: CommandContext, limit: number = 5): Command[] {
	const recent: Command[] = [];
	for (const id of recentIds) {
		if (recent.length >= limit) break;
		const cmd = commands.get(id);
		if (cmd && cmd.enabled(ctx)) {
			recent.push(cmd);
		}
	}
	return recent;
}

/** Execute a command by ID. Returns true if found and executed. */
async function execute(id: string, ctx: CommandContext): Promise<boolean> {
	const cmd = commands.get(id);
	if (!cmd || !cmd.enabled(ctx)) return false;
	await cmd.execute(ctx);
	trackRecent(id);
	return true;
}

export const commandRegistry = {
	register,
	registerAll,
	deregister,
	getAll,
	getById,
	search,
	getRecent,
	execute,
};
