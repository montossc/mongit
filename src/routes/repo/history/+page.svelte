<script lang="ts">
	import { onMount } from 'svelte';
	import { repoStore } from '$lib/stores/repo.svelte';
	import type { CommitData, RefData, LayoutResult } from '$lib/graph/types';
	import { assignLanes } from '$lib/graph/layout';

	// ── State ──
	let layout = $state<LayoutResult | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	// ── Request-ID guard (prevents stale async responses) ──
	let loadRequestId = 0;

	// ── Track which repo path we loaded for ──
	let loadedRepoPath = $state<string | null>(null);

	async function loadGraphData(repoPath: string): Promise<void> {
		loadRequestId += 1;
		const thisRequest = loadRequestId;

		loading = true;
		error = null;

		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const [commits, refs] = await Promise.all([
				invoke<CommitData[]>('get_commit_log', { path: repoPath, max_count: 10000 }),
				invoke<RefData[]>('get_refs', { path: repoPath })
			]);

			// Stale check: bail if a newer load started
			if (thisRequest !== loadRequestId) return;

			if (commits.length === 0) {
				layout = null;
				loadedRepoPath = repoPath;
				loading = false;
				return;
			}

			layout = assignLanes(commits, refs);
			loadedRepoPath = repoPath;
		} catch (e) {
			// Only set error if still the current request
			if (thisRequest !== loadRequestId) return;
			error = e instanceof Error ? e.message : String(e);
			layout = null;
		} finally {
			if (thisRequest === loadRequestId) {
				loading = false;
			}
		}
	}

	// ── Reactive: reload when active repo changes ──
	$effect(() => {
		const repoPath = repoStore.activeRepoPath;
		if (repoPath && repoPath !== loadedRepoPath) {
			loadGraphData(repoPath);
		} else if (!repoPath) {
			// Repo closed — clear state
			layout = null;
			loadedRepoPath = null;
			error = null;
		}
	});

	// ── Initial load on mount ──
	onMount(() => {
		if (repoStore.activeRepoPath) {
			loadGraphData(repoStore.activeRepoPath);
		}
	});
</script>

<div class="history-workspace">
	{#if loading}
		<div class="history-state">
			<p class="state-message">Loading history…</p>
		</div>
	{:else if error}
		<div class="history-state history-state--error">
			<p class="state-message">Failed to load history</p>
			<p class="state-detail">{error}</p>
			<button class="retry-btn" onclick={() => {
				if (repoStore.activeRepoPath) loadGraphData(repoStore.activeRepoPath);
			}}>
				Retry
			</button>
		</div>
	{:else if !layout}
		<div class="history-state">
			<p class="state-message">No commits found</p>
			<p class="state-hint">This repository has no commit history yet.</p>
		</div>
	{:else}
		<div class="history-state">
			<p class="state-message">Graph loaded — {layout.nodes.length} commits</p>
			<p class="state-hint">Selection binding will be connected next.</p>
		</div>
	{/if}
</div>

<style>
	.history-workspace {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.history-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-3);
		color: var(--color-text-muted);
	}

	.history-state--error {
		color: var(--color-danger);
	}

	.state-message {
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		margin: 0;
	}

	.state-detail {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		color: var(--color-text-muted);
		margin: 0;
		max-width: 400px;
		text-align: center;
		word-break: break-word;
	}

	.state-hint {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		margin: 0;
	}

	.retry-btn {
		padding: var(--space-2) var(--space-4);
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-primary);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.retry-btn:hover {
		background: var(--color-bg-hover);
	}
</style>
