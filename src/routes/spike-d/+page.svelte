<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import DiffViewer from '$lib/components/DiffViewer.svelte';
	import MergeEditor from '$lib/components/MergeEditor.svelte';
	import WatcherMonitor from '$lib/components/WatcherMonitor.svelte';
	import BenchmarkPanel from '$lib/components/BenchmarkPanel.svelte';
	import { diffStore, type DiffFileStatus } from '$lib/stores/diff.svelte';

	const tabs = [
		{ id: 'diff', label: 'Diff Viewer' },
		{ id: 'merge', label: 'Merge Editor' },
		{ id: 'watcher', label: 'File Watcher' },
		{ id: 'benchmarks', label: 'Benchmarks' }
	] as const;

	type TabId = (typeof tabs)[number]['id'];

	let activeTab = $state<TabId>('diff');
	let repoPathInput = $state('');

	function selectTab(id: TabId) {
		activeTab = id;
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

	async function loadDiff() {
		if (!repoPathInput.trim()) return;
		await diffStore.fetchDiff(repoPathInput.trim());
	}

	function statusIcon(status: DiffFileStatus): string {
		switch (status) {
			case 'Added':
				return 'A';
			case 'Modified':
				return 'M';
			case 'Deleted':
				return 'D';
			case 'Renamed':
				return 'R';
			default:
				return '?';
		}
	}

	function statusClass(status: DiffFileStatus): string {
		switch (status) {
			case 'Added':
				return 'status-added';
			case 'Deleted':
				return 'status-deleted';
			default:
				return 'status-modified';
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
			<span class="spike-badge">bd-htm</span>
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
		{#if activeTab === 'diff'}
			<div class="tab-panel visible" role="tabpanel">
				<div class="diff-layout">
					<!-- Sidebar: repo input + file list -->
					<aside class="diff-sidebar">
						<div class="repo-input-group">
							<input
								type="text"
								class="repo-input"
								placeholder="Repository path..."
								bind:value={repoPathInput}
								onkeydown={(e) => e.key === 'Enter' && loadDiff()}
							/>
							<button
								class="load-btn"
								onclick={loadDiff}
								disabled={diffStore.loading || !repoPathInput.trim()}
							>
								{diffStore.loading ? 'Loading...' : 'Load'}
							</button>
						</div>

						{#if diffStore.error}
							<div class="diff-error">{diffStore.error}</div>
						{/if}

						{#if diffStore.files.length > 0}
							<div class="file-list-header">
								Changed files ({diffStore.files.length})
							</div>
							<ul class="file-list">
								{#each diffStore.files as file}
									<li>
										<button
											class="file-item"
											class:selected={diffStore.selectedPath === file.path}
											onclick={() => diffStore.selectFile(file.path)}
										>
											<span class="file-status {statusClass(file.status)}"
												>{statusIcon(file.status)}</span
											>
											<span class="file-path">{file.path}</span>
										</button>
									</li>
								{/each}
							</ul>
						{:else if !diffStore.loading && diffStore.repoPath}
							<div class="diff-empty-msg">No changed files</div>
						{/if}
					</aside>

					<!-- Main: diff viewer -->
					<div class="diff-main">
						{#if diffStore.loadingContent}
							<div class="diff-loading">Loading diff...</div>
						{:else if diffStore.content && diffStore.selectedPath}
							<DiffViewer
								original={diffStore.content.original}
								modified={diffStore.content.modified}
								filename={diffStore.selectedPath}
							/>
						{:else}
							<DiffViewer />
						{/if}
					</div>
				</div>
			</div>
		{:else if activeTab === 'merge'}
			<div class="tab-panel visible" role="tabpanel">
				<MergeEditor />
			</div>
		{:else if activeTab === 'watcher'}
			<div class="tab-panel visible" role="tabpanel">
				<WatcherMonitor />
			</div>
		{:else if activeTab === 'benchmarks'}
			<div class="tab-panel visible" role="tabpanel">
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

	/* ── Diff layout: sidebar + main ──────────────────────────────── */

	.diff-layout {
		display: flex;
		height: 100%;
	}

	.diff-sidebar {
		width: 280px;
		min-width: 200px;
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		background: var(--color-bg-surface);
		overflow-y: auto;
	}

	.repo-input-group {
		display: flex;
		gap: var(--space-2);
		padding: var(--space-3);
		border-bottom: 1px solid var(--color-border);
	}

	.repo-input {
		flex: 1;
		font-family: var(--font-mono);
		font-size: 11px;
		padding: var(--space-2) var(--space-3);
		background: var(--color-bg);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		outline: none;
	}

	.repo-input:focus {
		border-color: var(--color-accent);
	}

	.load-btn {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 500;
		padding: var(--space-2) var(--space-3);
		background: var(--color-accent);
		color: white;
		border: none;
		border-radius: var(--radius-sm);
		cursor: pointer;
		white-space: nowrap;
	}

	.load-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.diff-error {
		padding: var(--space-3);
		font-size: 11px;
		color: var(--color-diff-removed-text);
		background: var(--color-bg);
	}

	.file-list-header {
		padding: var(--space-3);
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-secondary);
		border-bottom: 1px solid var(--color-border);
	}

	.file-list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.file-item {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		width: 100%;
		padding: var(--space-2) var(--space-3);
		font-size: 11px;
		background: transparent;
		border: none;
		color: var(--color-text-primary);
		cursor: pointer;
		text-align: left;
		transition: background var(--transition-fast);
	}

	.file-item:hover {
		background: var(--color-bg-hover);
	}

	.file-item.selected {
		background: var(--color-accent-muted);
	}

	.file-status {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 700;
		width: 16px;
		text-align: center;
		flex-shrink: 0;
	}

	.status-added {
		color: var(--color-diff-added-text);
	}

	.status-deleted {
		color: var(--color-diff-removed-text);
	}

	.status-modified {
		color: var(--color-accent);
	}

	.file-path {
		font-family: var(--font-mono);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.diff-empty-msg {
		padding: var(--space-4);
		font-size: 12px;
		color: var(--color-text-muted);
		text-align: center;
	}

	.diff-main {
		flex: 1;
		min-width: 0;
		padding: var(--space-4);
	}

	.diff-loading {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--color-text-muted);
		font-size: 13px;
	}
</style>
