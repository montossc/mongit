/**
 * Command registry — Svelte 5 runes-based singleton store.
 *
 * Holds a Map<string, Command> and provides reactive access
 * to the full command list, filtered results, and execution.
 */

import {
	CATEGORY_ORDER,
	type Command,
	type CommandCategory,
	type CommandContext,
} from "./types";

// ── Registry state ──────────────────────────────────────────────────────

const commands = $state(new Map<string, Command>());

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
 * Get commands filtered by search query, grouped by category.
 * Only includes commands where `enabled(ctx)` returns true.
 * Results are ordered by category (CATEGORY_ORDER), then alphabetically by label.
 */
function search(
	query: string,
	ctx: CommandContext,
): { category: CommandCategory; commands: Command[] }[] {
	const needle = query.toLowerCase().trim();

	// Filter: enabled + matches query
	const matches = getAll().filter((cmd) => {
		if (!cmd.enabled(ctx)) return false;
		if (!needle) return true; // empty query shows all enabled
		return cmd.label.toLowerCase().includes(needle);
	});

	// Group by category, respecting CATEGORY_ORDER
	const groups: { category: CommandCategory; commands: Command[] }[] = [];

	for (const cat of CATEGORY_ORDER) {
		const inCategory = matches
			.filter((cmd) => cmd.category === cat)
			.sort((a, b) => a.label.localeCompare(b.label));

		if (inCategory.length > 0) {
			groups.push({ category: cat, commands: inCategory });
		}
	}

	return groups;
}

/** Execute a command by ID. Returns true if found and executed. */
async function execute(id: string, ctx: CommandContext): Promise<boolean> {
	const cmd = commands.get(id);
	if (!cmd || !cmd.enabled(ctx)) return false;
	await cmd.execute(ctx);
	return true;
}

export const commandRegistry = {
	register,
	registerAll,
	deregister,
	getAll,
	getById,
	search,
	execute,
};
