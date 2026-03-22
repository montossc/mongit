/**
 * Initial command definitions for mongit.
 *
 * Registers all built-in commands covering navigation, git operations,
 * and view actions. Call `registerBuiltinCommands()` once at app startup.
 */

import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";
import { repoStore } from "$lib/stores/repo.svelte";
import { setTheme, theme } from "$lib/stores/theme.svelte";
import { commandRegistry } from "./registry.svelte";
import type { Command, CommandContext } from "./types";

// ── Helpers ──────────────────────────────────────────────────────────────

/** Predicate: command requires an open repo. */
const requiresRepo = (ctx: CommandContext): boolean => ctx.hasRepo;

/** Always enabled. */
const always = (): boolean => true;

// ── Command definitions ──────────────────────────────────────────────────

const builtinCommands: Command[] = [
	// ── Navigation ────────────────────────────────────────────────────
	{
		id: "nav.summary",
		label: "Go to Summary",
		category: "navigation",
		enabled: requiresRepo,
		execute: () => {
			goto("/repo");
		},
	},
	{
		id: "nav.changes",
		label: "Go to Changes",
		category: "navigation",
		enabled: requiresRepo,
		execute: () => {
			goto("/repo/changes");
		},
	},
	{
		id: "nav.home",
		label: "Go to Home",
		category: "navigation",
		enabled: always,
		execute: () => {
			goto("/");
		},
	},

	// ── Git operations ───────────────────────────────────────────────
	{
		id: "git.fetch",
		label: "Fetch from Remote",
		category: "git",
		shortcutHint: undefined,
		enabled: requiresRepo,
		execute: async (ctx) => {
			if (!ctx.repoPath) return;
			await invoke("fetch", { path: ctx.repoPath });
		},
	},
	{
		id: "git.pull",
		label: "Pull from Remote",
		category: "git",
		enabled: requiresRepo,
		execute: async (ctx) => {
			if (!ctx.repoPath) return;
			await invoke("pull", { path: ctx.repoPath });
		},
	},
	{
		id: "git.push",
		label: "Push to Remote",
		category: "git",
		enabled: requiresRepo,
		execute: async (ctx) => {
			if (!ctx.repoPath) return;
			await invoke("push", { path: ctx.repoPath, forceWithLease: false });
		},
	},
	{
		id: "git.create-branch",
		label: "Create Branch…",
		category: "git",
		enabled: requiresRepo,
		execute: async (ctx) => {
			if (!ctx.repoPath) return;
			// Navigate to summary which has branch UI
			await goto("/repo");
		},
	},

	// ── View ─────────────────────────────────────────────────────────
	{
		id: "view.toggle-theme",
		label: "Toggle Dark/Light Theme",
		category: "view",
		enabled: always,
		execute: () => {
			const current = theme;
			if (current === "dark") {
				setTheme("light");
			} else {
				setTheme("dark");
			}
		},
	},

	// ── General ──────────────────────────────────────────────────────
	{
		id: "general.open-repo",
		label: "Open Repository…",
		category: "general",
		enabled: always,
		execute: async () => {
			await repoStore.openFolderPicker();
		},
	},
];

// ── Registration ─────────────────────────────────────────────────────────

/** Register all built-in commands. Call once at app startup. */
export function registerBuiltinCommands(): void {
	commandRegistry.registerAll(builtinCommands);
}
