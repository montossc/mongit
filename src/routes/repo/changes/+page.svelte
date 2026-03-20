<script lang="ts">
	import { onMount } from 'svelte';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { changesStore, type FileChangeKind } from '$lib/stores/changes.svelte';
	import { listen } from '@tauri-apps/api/event';

	let unlisten: (() => void) | null = null;
	let mounted = true;

	onMount(() => {
		// Load files when the route mounts
		if (repoStore.activeRepoPath) {
			changesStore.loadFiles(repoStore.activeRepoPath);
		}

		// Listen for file system changes and refresh
		const setupListener = async () => {
			const cb = await listen('repo-changed', () => {
				changesStore.refresh();
			});
			if (mounted) {
				unlisten = cb;
			} else {
				cb(); // Already unmounted — clean up immediately
			}
		};
		setupListener();

		return () => {
			mounted = false;
			if (unlisten) unlisten();
		};
	});

	/** Map a FileChangeKind to a short display label. */
	function kindLabel(kind: FileChangeKind): string {
		switch (kind) {
			case 'Added': return 'A';
			case 'Modified': return 'M';
			case 'Deleted': return 'D';
			case 'Renamed': return 'R';
			case 'Typechange': return 'T';
		}
	}

	/** Map a FileChangeKind to a CSS modifier class. */
	function kindClass(kind: FileChangeKind): string {
		switch (kind) {
			case 'Added': return 'added';
			case 'Modified': return 'modified';
			case 'Deleted': return 'deleted';
			case 'Renamed': return 'renamed';
			case 'Typechange': return 'typechange';
		}
	}

	/** Get the filename from a path. */
	function fileName(path: string): string {
		const parts = path.split('/');
		return parts[parts.length - 1];
	}

	/** Get the directory from a path, or empty string. */
	function fileDir(path: string): string {
		const lastSlash = path.lastIndexOf('/');
		return lastSlash > 0 ? path.substring(0, lastSlash + 1) : '';
	}
</script>

<div class="changes-workspace">
	{#if changesStore.loading && changesStore.files.length === 0}
		<!-- Loading state (only when no cached files) -->
		<div class="state-message">
			<div class="spinner"></div>
			<p>Loading changed files…</p>
		</div>

	{:else if changesStore.error}
		<!-- Error state -->
		<div class="state-message error">
			<p class="state-label">Error loading changes</p>
			<p class="state-detail">{changesStore.error}</p>
			<button class="retry-btn" onclick={() => changesStore.refresh()}>Retry</button>
		</div>

	{:else if changesStore.files.length === 0}
		<!-- Empty state: clean repo -->
		<div class="state-message">
			<p class="state-label">No changes</p>
			<p class="state-detail">Working tree is clean</p>
		</div>

	{:else}
		<!-- File list -->
		<div class="file-list" role="listbox" aria-label="Changed files">
			{#each changesStore.files as file (file.path)}
				<button
					class="file-row"
					class:selected={changesStore.selectedPath === file.path}
					role="option"
					aria-selected={changesStore.selectedPath === file.path}
					onclick={() => changesStore.selectFile(file.path)}
				>
					<span class="file-path">
						{#if fileDir(file.path)}
							<span class="file-dir">{fileDir(file.path)}</span>
						{/if}
						<span class="file-name">{fileName(file.path)}</span>
					</span>

					<span class="file-badges">
						{#if file.staged}
							<span class="status-badge staged {kindClass(file.staged)}" title="Staged: {file.staged}">
								{kindLabel(file.staged)}
							</span>
						{/if}
						{#if file.unstaged}
							<span class="status-badge unstaged {kindClass(file.unstaged)}" title="Unstaged: {file.unstaged}">
								{kindLabel(file.unstaged)}
							</span>
						{/if}
					</span>
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.changes-workspace {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
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

	.retry-btn {
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

	.retry-btn:hover {
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

	/* ── File list ──────────────────────────────────────────────────── */

	.file-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-2) 0;
	}

	.file-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		height: var(--size-row-default);
		padding: 0 var(--space-5);
		background: none;
		border: none;
		color: var(--color-text-primary);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		text-align: left;
		transition: background var(--transition-fast);
	}

	.file-row:hover {
		background: var(--color-bg-hover);
	}

	.file-row.selected {
		background: var(--color-bg-active);
	}

	.file-row:focus-visible {
		outline: var(--focus-ring-width) solid var(--focus-ring-color);
		outline-offset: calc(-1 * var(--focus-ring-width));
	}

	/* ── File path ──────────────────────────────────────────────────── */

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

	/* ── Status badges ──────────────────────────────────────────────── */

	.file-badges {
		display: flex;
		gap: var(--space-1);
		flex-shrink: 0;
		margin-left: var(--space-3);
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		border-radius: var(--radius-xs, 3px);
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		line-height: 1;
	}

	/* Staged badges: filled background */
	.status-badge.staged.added     { background: var(--color-success); color: white; }
	.status-badge.staged.modified  { background: var(--color-info); color: white; }
	.status-badge.staged.deleted   { background: var(--color-danger); color: white; }
	.status-badge.staged.renamed   { background: var(--color-warning); color: white; }
	.status-badge.staged.typechange { background: var(--color-text-muted); color: white; }

	/* Unstaged badges: outline style */
	.status-badge.unstaged.added     { border: 1px solid var(--color-success); color: var(--color-success); }
	.status-badge.unstaged.modified  { border: 1px solid var(--color-info); color: var(--color-info); }
	.status-badge.unstaged.deleted   { border: 1px solid var(--color-danger); color: var(--color-danger); }
	.status-badge.unstaged.renamed   { border: 1px solid var(--color-warning); color: var(--color-warning); }
	.status-badge.unstaged.typechange { border: 1px solid var(--color-text-muted); color: var(--color-text-muted); }
</style>
