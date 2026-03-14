<script lang="ts">
	import { onMount } from 'svelte';
	import type { CommitData, CommitNode, LayoutResult } from '$lib/graph/types';
	import { assignLanes, generateSyntheticCommits } from '$lib/graph/layout';
	import GraphCanvas from '$lib/graph/GraphCanvas.svelte';
	import CommitDetail from '$lib/graph/CommitDetail.svelte';
	import FpsOverlay from '$lib/graph/FpsOverlay.svelte';

	interface RefData {
		name: string;
		ref_type: 'LocalBranch' | 'RemoteBranch' | 'Tag' | 'Head';
		commit_id: string;
	}

	let repoPath = $state('');
	let layout = $state<LayoutResult | null>(null);
	let selectedNode = $state<CommitNode | null>(null);
	let error = $state<string | null>(null);
	let loading = $state(false);
	let showFps = $state(false);
	let scrollTop = $state(0);
	let canvasHeight = $state(0);
	let isTauri = $state(false);
	let commitCount = $state(0);
	let syntheticCount = $state(1000);

	onMount(() => {
		isTauri = '__TAURI_INTERNALS__' in window;
		if (isTauri) {
			repoPath = '.';
		}

		function handleGlobalKeydown(e: KeyboardEvent) {
			if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'P') {
				e.preventDefault();
				showFps = !showFps;
			}
		}

		window.addEventListener('keydown', handleGlobalKeydown);
		return () => window.removeEventListener('keydown', handleGlobalKeydown);
	});

	async function loadRepo() {
		if (!repoPath.trim()) return;
		error = null;
		loading = true;

		try {
			if (!isTauri) {
				error = 'Tauri IPC not available — use synthetic data for testing.';
				loading = false;
				return;
			}

			const { invoke } = await import('@tauri-apps/api/core');
			const [commits, refs] = await Promise.all([
				invoke<CommitData[]>('get_commit_log', { path: repoPath, max_count: 10000 }),
				invoke<RefData[]>('get_refs', { path: repoPath })
			]);

			commitCount = commits.length;
			layout = assignLanes(commits, refs);
			selectedNode = null;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			layout = null;
		} finally {
			loading = false;
		}
	}

	function loadSyntheticData() {
		error = null;
		loading = true;

		try {
			const commits = generateSyntheticCommits(syntheticCount, 5);
			commitCount = commits.length;

			const refs: RefData[] = [];
			if (commits.length > 0) {
				refs.push({ name: 'main', ref_type: 'Head', commit_id: commits[0].id });
				refs.push({ name: 'main', ref_type: 'LocalBranch', commit_id: commits[0].id });
			}
			if (commits.length > 10) {
				refs.push({ name: 'feature/graph', ref_type: 'LocalBranch', commit_id: commits[10].id });
			}
			if (commits.length > 50) {
				refs.push({ name: 'v0.1.0', ref_type: 'Tag', commit_id: commits[50].id });
			}
			if (commits.length > 100) {
				refs.push({ name: 'origin/main', ref_type: 'RemoteBranch', commit_id: commits[100].id });
			}

			layout = assignLanes(commits, refs);
			selectedNode = null;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			layout = null;
		} finally {
			loading = false;
		}
	}

	function handleSelectCommit(id: string) {
		if (!layout) return;
		selectedNode = layout.nodeMap.get(id) ?? null;
	}

	function handleNavigateToCommit(commitId: string) {
		if (!layout) return;
		selectedNode = layout.nodeMap.get(commitId) ?? null;
	}

	function handleContextAction(action: string, node: CommitNode) {
		switch (action) {
			case 'copy-hash':
				navigator.clipboard.writeText(node.data.id);
				break;
			case 'copy-message':
				navigator.clipboard.writeText(node.data.message);
				break;
			default:
				console.log(`Context action: ${action} on ${node.data.id.slice(0, 7)}`);
		}
	}
</script>

<main class="app-layout">
	<header class="toolbar">
		<div class="toolbar-left">
			<h1 class="app-title">mongit</h1>

			{#if isTauri}
				<div class="input-group">
					<input
						type="text"
						class="repo-input"
						bind:value={repoPath}
						placeholder="Repository path..."
						onkeydown={(e) => e.key === 'Enter' && loadRepo()}
					/>
					<button class="btn btn-primary" onclick={loadRepo} disabled={loading}>
						{loading ? 'Loading...' : 'Open'}
					</button>
				</div>
			{/if}

			<div class="input-group">
				<input
					type="number"
					class="count-input"
					bind:value={syntheticCount}
					min={10}
					max={100000}
					step={1000}
				/>
				<button class="btn btn-secondary" onclick={loadSyntheticData} disabled={loading}>
					Synthetic
				</button>
			</div>
		</div>

		<div class="toolbar-right">
			{#if layout}
				<span class="stat">{commitCount.toLocaleString()} commits</span>
				<span class="stat">{layout.laneCount} lanes</span>
				<span class="stat">{layout.layoutTimeMs.toFixed(1)}ms layout</span>
			{/if}
			<button
				class="btn btn-ghost"
				class:active={showFps}
				onclick={() => (showFps = !showFps)}
				title="Toggle FPS overlay (Cmd+Shift+P)"
			>
				FPS
			</button>
		</div>
	</header>

	{#if error}
		<div class="error-banner">{error}</div>
	{/if}

	<div class="content">
		{#if layout}
			<div class="graph-panel">
				<GraphCanvas
					{layout}
					onSelectCommit={handleSelectCommit}
					onContextAction={handleContextAction}
				/>
				<FpsOverlay {layout} {scrollTop} {canvasHeight} visible={showFps} />
			</div>
			<aside class="detail-panel">
				<CommitDetail node={selectedNode} onNavigateToCommit={handleNavigateToCommit} />
			</aside>
		{:else if !loading}
			<div class="empty-state">
				<div class="empty-icon">
					<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
						<circle cx="12" cy="12" r="3" />
						<path d="M12 3v6m0 6v6" />
						<circle cx="6" cy="18" r="2" />
						<circle cx="18" cy="6" r="2" />
						<path d="M6 16v-3a3 3 0 0 1 3-3h6a3 3 0 0 1 3 3v-3" />
					</svg>
				</div>
				<h2>No repository loaded</h2>
				<p>Open a git repository or generate synthetic data to test the graph renderer.</p>
				<button class="btn btn-primary" onclick={loadSyntheticData}>
					Generate {syntheticCount.toLocaleString()} synthetic commits
				</button>
			</div>
		{/if}

		{#if loading}
			<div class="loading-overlay">
				<div class="spinner"></div>
				<p>Loading commits...</p>
			</div>
		{/if}
	</div>
</main>

<style>
	.app-layout {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--color-bg);
		color: var(--color-text-primary);
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-6);
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
		gap: var(--space-4);
		flex-shrink: 0;
		-webkit-app-region: drag;
	}

	.toolbar-left,
	.toolbar-right {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		-webkit-app-region: no-drag;
	}

	.app-title {
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 700;
		color: var(--color-accent);
		margin: 0;
		white-space: nowrap;
	}

	.input-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.repo-input {
		font-family: var(--font-mono);
		font-size: 12px;
		padding: var(--space-2) var(--space-4);
		background: var(--color-bg);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-primary);
		width: 240px;
		outline: none;
	}

	.repo-input:focus {
		border-color: var(--color-accent);
	}

	.count-input {
		font-family: var(--font-mono);
		font-size: 12px;
		padding: var(--space-2) var(--space-3);
		background: var(--color-bg);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-primary);
		width: 80px;
		outline: none;
		appearance: textfield;
		-moz-appearance: textfield;
	}

	.count-input::-webkit-inner-spin-button,
	.count-input::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}

	.count-input:focus {
		border-color: var(--color-accent);
	}

	.btn {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		padding: var(--space-2) var(--space-4);
		border-radius: var(--radius-sm);
		border: 1px solid transparent;
		cursor: pointer;
		white-space: nowrap;
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: var(--color-accent);
		color: var(--color-bg);
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.btn-secondary {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
		border-color: var(--color-border);
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--color-bg-active);
	}

	.btn-ghost {
		background: transparent;
		color: var(--color-text-secondary);
		border-color: var(--color-border);
	}

	.btn-ghost:hover {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.btn-ghost.active {
		background: var(--color-accent-muted);
		color: var(--color-accent);
		border-color: var(--color-accent);
	}

	.stat {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--color-text-muted);
	}

	.error-banner {
		padding: var(--space-3) var(--space-6);
		background: var(--color-danger-muted);
		color: var(--color-danger);
		font-size: 12px;
		border-bottom: 1px solid var(--color-danger);
		flex-shrink: 0;
	}

	.content {
		display: flex;
		flex: 1;
		min-height: 0;
		position: relative;
	}

	.graph-panel {
		flex: 1;
		position: relative;
		min-width: 0;
	}

	.detail-panel {
		width: 320px;
		flex-shrink: 0;
		overflow-y: auto;
		border-left: 1px solid var(--color-border);
		background: var(--color-bg-surface);
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-4);
		color: var(--color-text-muted);
	}

	.empty-icon {
		color: var(--color-text-muted);
		opacity: 0.4;
	}

	.empty-state h2 {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text-secondary);
		margin: 0;
	}

	.empty-state p {
		font-size: 13px;
		margin: 0;
		max-width: 300px;
		text-align: center;
		line-height: 1.5;
	}

	.loading-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-4);
		background: rgba(15, 17, 23, 0.8);
		color: var(--color-text-secondary);
		font-size: 13px;
	}

	.spinner {
		width: 24px;
		height: 24px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
