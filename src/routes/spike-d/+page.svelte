<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import DiffViewer from '$lib/components/DiffViewer.svelte';
	import MergeEditor from '$lib/components/MergeEditor.svelte';
	import WatcherMonitor from '$lib/components/WatcherMonitor.svelte';
	import BenchmarkPanel from '$lib/components/BenchmarkPanel.svelte';

	const tabs = [
		{ id: 'diff', label: 'Diff Viewer' },
		{ id: 'merge', label: 'Merge Editor' },
		{ id: 'watcher', label: 'File Watcher' },
		{ id: 'benchmarks', label: 'Benchmarks' },
	] as const;

	type TabId = (typeof tabs)[number]['id'];

	let activeTab = $state<TabId>('diff');

	// Track which tabs have been visited so we can lazy-mount
	// but keep alive for the session (avoids re-creating CM6 instances)
	let mounted = $state<Set<TabId>>(new Set(['diff']));

	function selectTab(id: TabId) {
		activeTab = id;
		mounted = new Set([...mounted, id]);
	}

	function handleKeydown(e: KeyboardEvent) {
		// Cmd+1-4 to switch tabs
		if (e.metaKey && e.key >= '1' && e.key <= '4') {
			e.preventDefault();
			const idx = parseInt(e.key) - 1;
			if (idx < tabs.length) {
				selectTab(tabs[idx].id);
			}
		}
	}

	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKeydown);
	});
</script>

<main class="spike-layout">
	<header class="spike-header">
		<div class="header-left">
			<a href="/" class="back-link">&larr; Back</a>
			<h1 class="spike-title">Spike D — CodeMirror 6 + File Watcher</h1>
			<span class="spike-badge">bd-2n2</span>
		</div>
	</header>

	<div class="tab-bar" role="tablist">
		{#each tabs as tab, idx}
			<button
				role="tab"
				class="tab"
				class:active={activeTab === tab.id}
				aria-selected={activeTab === tab.id}
				onclick={() => selectTab(tab.id)}
			>
				{tab.label}
				<span class="tab-shortcut">⌘{idx + 1}</span>
			</button>
		{/each}
	</div>

	<div class="tab-content">
		{#if mounted.has('diff')}
			<div class="tab-panel" class:visible={activeTab === 'diff'} role="tabpanel">
				<DiffViewer />
			</div>
		{/if}

		{#if mounted.has('merge')}
			<div class="tab-panel" class:visible={activeTab === 'merge'} role="tabpanel">
				<MergeEditor />
			</div>
		{/if}

		{#if mounted.has('watcher')}
			<div class="tab-panel" class:visible={activeTab === 'watcher'} role="tabpanel">
				<WatcherMonitor />
			</div>
		{/if}

		{#if mounted.has('benchmarks')}
			<div class="tab-panel" class:visible={activeTab === 'benchmarks'} role="tabpanel">
				<BenchmarkPanel />
			</div>
		{/if}
	</div>
</main>

<style>
	.spike-layout {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--color-bg);
		color: var(--color-text-primary);
	}

	.spike-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-6);
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
		-webkit-app-region: drag;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		-webkit-app-region: no-drag;
	}

	.back-link {
		font-size: 12px;
		color: var(--color-accent);
		text-decoration: none;
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-sm);
		transition: background var(--transition-fast);
	}

	.back-link:hover {
		background: var(--color-bg-hover);
	}

	.spike-title {
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
		white-space: nowrap;
	}

	.spike-badge {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--color-accent);
		background: var(--color-accent-muted);
		padding: 2px 6px;
		border-radius: var(--radius-full);
		white-space: nowrap;
	}

	.tab-bar {
		display: flex;
		gap: 0;
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
		padding: 0 var(--space-6);
		flex-shrink: 0;
	}

	.tab {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text-secondary);
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		padding: var(--space-4) var(--space-5);
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: var(--space-3);
		transition:
			color var(--transition-fast),
			border-color var(--transition-fast);
		white-space: nowrap;
	}

	.tab:hover {
		color: var(--color-text-primary);
	}

	.tab.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}

	.tab-shortcut {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--color-text-muted);
		background: var(--color-bg-elevated);
		padding: 1px 4px;
		border-radius: 3px;
	}

	.tab.active .tab-shortcut {
		color: var(--color-accent);
		background: var(--color-accent-muted);
	}

	.tab-content {
		flex: 1;
		min-height: 0;
		position: relative;
	}

	.tab-panel {
		position: absolute;
		inset: 0;
		overflow: auto;
		display: none;
	}

	.tab-panel.visible {
		display: block;
	}
</style>
