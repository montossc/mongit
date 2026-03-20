<script lang="ts">
	import { onMount } from 'svelte';
	import { repoStore } from '$lib/stores/repo.svelte';
	import type { CommitData, CommitNode, RefData, LayoutResult } from '$lib/graph/types';
	import { assignLanes } from '$lib/graph/layout';
	import GraphCanvas from '$lib/graph/GraphCanvas.svelte';
	import CommitDetail from '$lib/graph/CommitDetail.svelte';

	// ── Graph state ──
	let layout = $state<LayoutResult | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	// ── Selection state (route-owned, passed to GraphCanvas as controlled prop) ──
	let selectedId = $state<string | null>(null);
	const selectedNode = $derived<CommitNode | null>(
		selectedId && layout ? layout.nodeMap.get(selectedId) ?? null : null
	);

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
				selectedId = null;
				loadedRepoPath = repoPath;
				loading = false;
				return;
			}

			const newLayout = assignLanes(commits, refs);

			// Selection lifecycle: preserve if commit still exists, clear if gone
			if (selectedId && !newLayout.nodeMap.has(selectedId)) {
				selectedId = null;
			}

			layout = newLayout;
			loadedRepoPath = repoPath;
		} catch (e) {
			// Only set error if still the current request
			if (thisRequest !== loadRequestId) return;
			error = e instanceof Error ? e.message : String(e);
			layout = null;
			selectedId = null;
		} finally {
			if (thisRequest === loadRequestId) {
				loading = false;
			}
		}
	}

	// ── Selection handlers ──
	function handleSelectCommit(id: string): void {
		selectedId = id;
	}

	function handleNavigateToCommit(commitId: string): void {
		if (!layout) return;
		// Only navigate if target commit exists in current layout
		if (layout.nodeMap.has(commitId)) {
			selectedId = commitId;
		}
	}

	// ── Reactive: reload on repo change, clear on repo close ──
	$effect(() => {
		const repoPath = repoStore.activeRepoPath;
		if (repoPath && repoPath !== loadedRepoPath) {
			// New repo — clear stale selection before loading
			selectedId = null;
			loadGraphData(repoPath);
		} else if (!repoPath) {
			layout = null;
			loadedRepoPath = null;
			selectedId = null;
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
		<div class="history-graph-area">
			<GraphCanvas
				{layout}
				{selectedId}
				onSelectCommit={handleSelectCommit}
			/>
		</div>
		<aside class="history-detail-panel">
			<CommitDetail
				node={selectedNode}
				onNavigateToCommit={handleNavigateToCommit}
			/>
		</aside>
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

	/* ── Graph + Detail layout ── */
	.history-graph-area {
		flex: 1;
		min-width: 0;
		overflow: hidden;
	}

	.history-detail-panel {
		width: 320px;
		flex-shrink: 0;
		border-left: 1px solid var(--color-border);
		overflow-y: auto;
		background: var(--color-bg-surface);
	}
</style>
