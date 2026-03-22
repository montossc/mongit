/**
 * Initial command definitions for mongit.
 *
 * Registers all built-in commands covering navigation, git operations,
 * staging, view actions, and general utilities.
 * Call `registerBuiltinCommands()` once at app startup.
 */

import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";
import { repoStore } from "$lib/stores/repo.svelte";
import { setTheme, theme } from "$lib/stores/theme.svelte";
import { changesStore } from "$lib/stores/changes.svelte";
import { commandRegistry } from "./registry.svelte";
import type { Command, CommandContext } from "./types";

// ── Helpers ──────────────────────────────────────────────────────────────

/** Predicate: command requires an open repo. */
const requiresRepo = (ctx: CommandContext): boolean => ctx.hasRepo;

/** Always enabled. */
const always = (): boolean => true;

// ── Command definitions ────────────────────────────────────────────────────

const builtinCommands: Command[] = [
	// ── Navigation ────────────────────────────────────────────────────────────
	{
		id: "nav.summary",
		label: "Go to Summary",
		description: "View repository overview with branch and status info",
		category: "navigation",
		enabled: requiresRepo,
		execute: () => {
			goto("/repo");
		},
	},
	{
		id: "nav.changes",
		label: "Go to Changes",
		description: "View working tree changes and staging area",
		category: "navigation",
		enabled: requiresRepo,
		execute: () => {
			goto("/repo/changes");
		},
	},
	{
		id: "nav.history",
		label: "Go to History",
		description: "View commit graph and history",
		category: "navigation",
		enabled: requiresRepo,
		execute: () => {
			goto("/repo/history");
		},
	},
	{
		id: "nav.home",
		label: "Go to Home",
		description: "Return to repository picker",
		category: "navigation",
		enabled: always,
		execute: () => {
			goto("/");
		},
	},

	// ── Git operations ───────────────────────────────────────────────────────
	{
		id: "git.fetch",
		label: "Fetch from Remote",
		description: "Download objects and refs from origin",
		category: "git",
		enabled: requiresRepo,
		execute: async (ctx) => {
			if (!ctx.repoPath) return;
			await invoke("fetch", { path: ctx.repoPath });
		},
	},
	{
		id: "git.pull",
		label: "Pull from Remote",
		description: "Fetch and merge changes from origin into current branch",
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
		description: "Push current branch commits to origin",
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
		description: "Create a new local branch from current HEAD",
		category: "git",
		enabled: requiresRepo,
		execute: async (ctx) => {
			if (!ctx.repoPath) return;
			// Navigate to summary which has branch UI
			await goto("/repo");
		},
	},
	{
		id: "git.refresh-status",
		label: "Refresh Repository Status",
		description: "Re-read repository state from disk",
		category: "git",
		enabled: requiresRepo,
		execute: async (ctx) => {
			if (!ctx.repoPath) return;
			await repoStore.openRepo(ctx.repoPath);
		},
	},

	// ── Staging ──────────────────────────────────────────────────────────────
	{
		id: "staging.refresh",
		label: "Refresh Changed Files",
		description: "Re-scan working tree for changed files",
		category: "staging",
		enabled: requiresRepo,
		execute: async () => {
			await changesStore.refresh();
		},
	},

	// ── View ─────────────────────────────────────────────────────────────────
	{
		id: "view.toggle-theme",
		label: "Toggle Dark/Light Theme",
		description: "Switch between dark and light color scheme",
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

	// ── General ──────────────────────────────────────────────────────────────
	{
		id: "general.open-repo",
		label: "Open Repository…",
		description: "Open a local Git repository from disk",
		category: "general",
		enabled: always,
		execute: async () => {
			await repoStore.openFolderPicker();
		},
	},
	{
		id: "general.close-repo",
		label: "Close Repository",
		description: "Close the current repository and return home",
		category: "general",
		enabled: requiresRepo,
		execute: () => {
			goto("/");
		},
	},
];

// ── Registration ─────────────────────────────────────────────────────────

/** Register all built-in commands. Call once at app startup. */
export function registerBuiltinCommands(): void {
	commandRegistry.registerAll(builtinCommands);
}
