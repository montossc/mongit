<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { conflictStore } from '$lib/stores/conflict.svelte';
	import MergeEditor from '$lib/components/MergeEditor.svelte';
	import { listen } from '@tauri-apps/api/event';

	let unlisten: (() => void) | null = null;
	let mounted = true;

	onMount(() => {
		if (!repoStore.activeRepoPath) {
			goto('/');
			return;
		}

		conflictStore.loadMergeState(repoStore.activeRepoPath);

		// Listen for file system changes and refresh
		const setupListener = async () => {
			const cb = await listen('repo-changed', () => {
				conflictStore.refresh();
			});
			if (mounted) {
				unlisten = cb;
			} else {
				cb();
			}
		};
		setupListener();

		return () => {
			mounted = false;
			if (unlisten) unlisten();
		};
	});

	function handleFileSelect(path: string) {
		conflictStore.selectFile(path);
	}

	/** Get the filename from a path. */
	function fileName(path: string): string {
		const parts = path.split('/');
		return parts[parts.length - 1];
	}

	/** Get the directory from a path. */
	function fileDir(path: string): string {
		const lastSlash = path.lastIndexOf('/');
		return lastSlash > 0 ? path.substring(0, lastSlash + 1) : '';
	}
</script>

<div class="resolve-workspace">
	{#if conflictStore.loading}
		<div class="state-message">
			<div class="spinner"></div>
			<p>Loading merge state…</p>
		</div>

	{:else if !conflictStore.isMerging}
		<div class="state-message">
			<p class="state-label">No merge in progress</p>
			<p class="state-detail">Start a merge to see conflicts here</p>
			<button class="back-btn" onclick={() => goto('/repo/changes')}>Back to Changes</button>
		</div>

	{:else if conflictStore.conflictCount === 0}
		<div class="state-message">
			<p class="state-label">All conflicts resolved</p>
			<p class="state-detail">You can now commit the merge result</p>
			<button class="back-btn" onclick={() => goto('/repo/changes')}>Back to Changes</button>
		</div>

	{:else}
		<div class="resolve-split">
			<!-- Conflict file list (left panel) -->
			<div class="conflict-file-panel">
				<div class="panel-header">
					<span class="panel-title">Conflicts</span>
					<span class="conflict-count">{conflictStore.conflictCount}</span>
				</div>
				<div class="conflict-file-list" role="listbox" aria-label="Conflicted files">
					{#each conflictStore.conflictedFiles as file (file.path)}
						<button
							class="conflict-file-row"
							class:selected={conflictStore.selectedPath === file.path}
							role="option"
							aria-selected={conflictStore.selectedPath === file.path}
							onclick={() => handleFileSelect(file.path)}
						>
							<span class="conflict-icon" title="Conflicted">!</span>
							<span class="file-path">
								{#if fileDir(file.path)}
									<span class="file-dir">{fileDir(file.path)}</span>
								{/if}
								<span class="file-name">{fileName(file.path)}</span>
							</span>
						</button>
					{/each}
				</div>
			</div>

			<!-- Merge editor (right panel) -->
			<div class="merge-panel">
				{#if conflictStore.contentLoading}
					<div class="state-message">
						<div class="spinner"></div>
						<p>Loading conflict content…</p>
					</div>
				{:else if conflictStore.error}
					<div class="state-message error">
						<p class="state-label">Error</p>
						<p class="state-detail">{conflictStore.error}</p>
					</div>
				{:else if conflictStore.content}
					<MergeEditor
						base={conflictStore.content.base ?? ''}
						ours={conflictStore.content.ours}
						theirs={conflictStore.content.theirs}
					/>
				{:else}
					<div class="state-message">
						<p class="state-detail">Select a conflicted file to resolve</p>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.resolve-workspace {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* ── Split layout ──────────────────────────────────────────────── */

	.resolve-split {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.conflict-file-panel {
		width: 260px;
		min-width: 180px;
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.merge-panel {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* ── Panel header ──────────────────────────────────────────────── */

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--color-border);
		background: var(--color-bg-surface);
		flex-shrink: 0;
	}

	.panel-title {
		font-size: var(--text-body-sm-size);
		font-weight: 600;
		color: var(--color-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.conflict-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 20px;
		height: 20px;
		padding: 0 var(--space-2);
		background: var(--color-danger);
		color: white;
		border-radius: 10px;
		font-size: 11px;
		font-weight: 600;
		font-family: var(--font-mono);
	}

	/* ── File list ──────────────────────────────────────────────────── */

	.conflict-file-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-2) 0;
	}

	.conflict-file-row {
		display: flex;
		align-items: center;
		width: 100%;
		height: var(--size-row-default);
		padding: 0 var(--space-4);
		gap: var(--space-3);
		background: none;
		border: none;
		color: var(--color-text-primary);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		text-align: left;
		transition: background var(--transition-fast);
	}

	.conflict-file-row:hover {
		background: var(--color-bg-hover);
	}

	.conflict-file-row.selected {
		background: var(--color-bg-active);
	}

	.conflict-file-row:focus-visible {
		outline: var(--focus-ring-width) solid var(--focus-ring-color);
		outline-offset: calc(-1 * var(--focus-ring-width));
	}

	.conflict-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		border-radius: var(--radius-xs, 3px);
		background: var(--color-danger);
		color: white;
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 700;
		flex-shrink: 0;
	}

	.file-path {
		display: flex;
		align-items: baseline;
		min-width: 0;
		overflow: hidden;
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
	}

	.file-dir {
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex-shrink: 1;
	}

	.file-name {
		color: var(--color-text-primary);
		white-space: nowrap;
		flex-shrink: 0;
	}

	/* ── State messages ─────────────────────────────────────────────── */

	.state-message {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-3);
		color: var(--color-text-muted);
		padding: var(--space-8);
	}

	.state-message.error {
		color: var(--color-danger);
	}

	.state-label {
		font-size: var(--text-body-size);
		font-weight: 500;
		margin: 0;
	}

	.state-detail {
		font-size: var(--text-body-sm-size);
		margin: 0;
		max-width: 360px;
		text-align: center;
		word-break: break-word;
	}

	.state-message.error .state-detail {
		color: var(--color-text-secondary);
	}

	.back-btn {
		margin-top: var(--space-3);
		padding: var(--space-2) var(--space-5);
		font-size: var(--text-body-sm-size);
		background: none;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-secondary);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.back-btn:hover {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.spinner {
		width: 20px;
		height: 20px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
