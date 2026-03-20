<script lang="ts">
	import { onMount } from 'svelte';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { Button, Input } from '$lib/components/ui';

	let manualPath = $state('');
	let dragOver = $state(false);

	onMount(() => {
		let mounted = true;
		repoStore.loadRecentRepos();

		let unlisten: (() => void) | undefined;

		async function setupDragDrop() {
			try {
				const { getCurrentWebviewWindow } = await import(
					'@tauri-apps/api/webviewWindow'
				);
				const webview = getCurrentWebviewWindow();
				const unlistenFn = await webview.onDragDropEvent((event) => {
					if (event.payload.type === 'over') {
						dragOver = true;
					} else if (event.payload.type === 'drop') {
						dragOver = false;
						const paths = event.payload.paths;
						if (paths.length > 0) {
							repoStore.openRepo(paths[0]);
						}
					} else if (event.payload.type === 'leave') {
						dragOver = false;
					}
				});

				// If component unmounted during async setup, clean up immediately
				if (!mounted) {
					unlistenFn();
					return;
				}

				unlisten = unlistenFn;
			} catch {
				// Drag-drop unavailable outside Tauri — silently skip
			}
		}

		setupDragDrop();

		return () => {
			mounted = false;
			unlisten?.();
		};
	});

	function handleManualOpen() {
		if (manualPath.trim()) {
			repoStore.openRepo(manualPath.trim());
			manualPath = '';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			handleManualOpen();
		}
	}

	function formatDate(timestamp: number): string {
		return new Date(timestamp * 1000).toLocaleDateString(undefined, {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
		});
	}
</script>

<main class="home" class:drag-over={dragOver}>
	<div class="home-content">
		<!-- Branding -->
		<header class="home-header">
			<svg
				class="home-logo"
				width="40"
				height="40"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<circle cx="12" cy="12" r="3" />
				<path d="M12 3v6m0 6v6" />
				<circle cx="6" cy="18" r="2" />
				<circle cx="18" cy="6" r="2" />
				<path d="M6 16v-3a3 3 0 0 1 3-3h6a3 3 0 0 1 3 3v-3" />
			</svg>
			<h1 class="home-title">mongit</h1>
			<p class="home-subtitle">Git client for macOS</p>
		</header>

		<!-- Open Repository Section -->
		<section class="open-section">
			<div class="open-actions">
				<Button
					variant="primary"
					size="prominent"
					onclick={() => repoStore.openFolderPicker()}
					disabled={repoStore.loading}
				>
					Open Repository…
				</Button>

				<div class="separator">
					<span class="separator-text">or enter path</span>
				</div>

				<div class="path-row">
					<Input
						bind:value={manualPath}
						placeholder="/path/to/repository"
						mono
						onkeydown={handleKeydown}
						disabled={repoStore.loading}
					/>
					<Button
						variant="secondary"
						onclick={handleManualOpen}
						disabled={repoStore.loading || !manualPath.trim()}
					>
						Open
					</Button>
				</div>
			</div>

			<p class="drag-hint">
				{#if dragOver}
					Drop to open repository
				{:else}
					You can also drag and drop a folder here
				{/if}
			</p>
		</section>

		<!-- Error -->
		{#if repoStore.error}
			<div class="error-banner" role="alert">
				<span class="error-text">{repoStore.error}</span>
				<button class="error-dismiss" onclick={() => repoStore.clearError()} aria-label="Dismiss error">
					✕
				</button>
			</div>
		{/if}

		<!-- Recent Repositories -->
		{#if repoStore.recentRepos.length > 0}
			<section class="recents-section">
				<h2 class="section-title">Recent</h2>
				<ul class="recents-list">
					{#each repoStore.recentRepos as repo (repo.path)}
						<li class="recent-item" class:stale={!repo.valid}>
							<button
								class="recent-button"
								onclick={() =>
									repo.valid
										? repoStore.openRepo(repo.path)
										: repoStore.retryRecentRepo(repo.path)}
								disabled={repoStore.loading}
							>
								<div class="recent-info">
									<span class="recent-name">{repo.name}</span>
									<span class="recent-path">{repo.path}</span>
								</div>
								<div class="recent-meta">
									{#if !repo.valid}
										<span class="stale-badge">Not found</span>
									{/if}
									<span class="recent-date"
										>{formatDate(repo.last_accessed)}</span
									>
								</div>
							</button>
							{#if !repo.valid}
								<button
									class="remove-btn"
									onclick={(e) => {
										e.stopPropagation();
										repoStore.removeRecentRepo(repo.path);
									}}
									title="Remove from recents"
								>
									✕
								</button>
							{/if}
						</li>
					{/each}
				</ul>
			</section>
		{/if}
	</div>

	<!-- Loading overlay -->
	{#if repoStore.loading}
		<div class="loading-overlay">
			<div class="spinner"></div>
			<p>Opening repository…</p>
		</div>
	{/if}

	<!-- Drag-drop overlay -->
	{#if dragOver}
		<div class="drop-overlay">
			<div class="drop-icon">
				<svg
					width="48"
					height="48"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path
						d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"
					/>
				</svg>
			</div>
			<p>Drop folder to open</p>
		</div>
	{/if}
</main>

<style>
	.home {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100vh;
		background: var(--color-bg);
		color: var(--color-text-primary);
		position: relative;
		-webkit-app-region: drag;
	}

	.home-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-8);
		max-width: 480px;
		width: 100%;
		padding: var(--space-8);
		-webkit-app-region: no-drag;
	}

	/* ── Header ─────────────────────────── */

	.home-header {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-3);
	}

	.home-logo {
		color: var(--color-accent);
		opacity: 0.8;
	}

	.home-title {
		font-family: var(--font-display);
		font-size: 28px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
		letter-spacing: -0.5px;
	}

	.home-subtitle {
		font-size: var(--text-body-sm-size);
		color: var(--color-text-muted);
		margin: 0;
	}

	/* ── Open section ───────────────────── */

	.open-section {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-5);
		width: 100%;
	}

	.open-actions {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-4);
		width: 100%;
	}

	.separator {
		display: flex;
		align-items: center;
		width: 100%;
		gap: var(--space-4);
	}

	.separator::before,
	.separator::after {
		content: '';
		flex: 1;
		height: 1px;
		background: var(--color-border);
	}

	.separator-text {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.path-row {
		display: flex;
		gap: var(--space-3);
		width: 100%;
	}

	.path-row :global(input) {
		flex: 1;
	}

	.drag-hint {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		margin: 0;
	}

	/* ── Error ──────────────────────────── */

	.error-banner {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		width: 100%;
		padding: var(--space-4) var(--space-5);
		background: var(--color-danger-muted);
		border: 1px solid color-mix(in srgb, var(--color-danger) 30%, transparent);
		border-radius: var(--radius-md);
	}

	.error-text {
		flex: 1;
		font-size: var(--text-body-sm-size);
		color: var(--color-danger);
		word-break: break-word;
	}

	.error-dismiss {
		background: none;
		border: none;
		color: var(--color-danger);
		cursor: pointer;
		padding: var(--space-1);
		font-size: 14px;
		opacity: 0.7;
		flex-shrink: 0;
	}

	.error-dismiss:hover {
		opacity: 1;
	}

	/* ── Recents ────────────────────────── */

	.recents-section {
		width: 100%;
	}

	.section-title {
		font-size: var(--text-body-sm-size);
		font-weight: 600;
		color: var(--color-text-secondary);
		margin: 0 0 var(--space-3);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.recents-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
		background: var(--color-border);
		border-radius: var(--radius-md);
		overflow: hidden;
	}

	.recent-item {
		display: flex;
		align-items: center;
		background: var(--color-bg-surface);
	}

	.recent-button {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-4) var(--space-5);
		background: none;
		border: none;
		cursor: pointer;
		text-align: left;
		color: var(--color-text-primary);
		transition: background var(--transition-fast);
		min-width: 0;
	}

	.recent-button:hover:not(:disabled) {
		background: var(--color-bg-hover);
	}

	.recent-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.recent-info {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		min-width: 0;
	}

	.recent-name {
		font-size: var(--text-body-size);
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.recent-path {
		font-family: var(--font-mono);
		font-size: var(--text-mono-xs-size);
		color: var(--color-text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.recent-meta {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-shrink: 0;
	}

	.recent-date {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.stale-badge {
		font-size: var(--text-caption-size);
		color: var(--color-warning);
		padding: var(--space-1) var(--space-3);
		background: var(--color-warning-muted);
		border-radius: var(--radius-sm);
		white-space: nowrap;
	}

	.recent-item.stale .recent-name {
		color: var(--color-text-secondary);
	}

	.remove-btn {
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		padding: var(--space-3);
		font-size: 14px;
		flex-shrink: 0;
		opacity: 0.5;
		transition: opacity var(--transition-fast);
	}

	.remove-btn:hover {
		opacity: 1;
		color: var(--color-danger);
	}

	/* ── Loading overlay ────────────────── */

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
		z-index: 10;
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

	/* ── Drop overlay ───────────────────── */

	.drop-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-4);
		background: rgba(15, 17, 23, 0.9);
		border: 2px dashed var(--color-accent);
		border-radius: var(--radius-lg);
		margin: var(--space-4);
		color: var(--color-accent);
		font-size: 16px;
		font-weight: 500;
		z-index: 20;
	}

	.drop-icon {
		opacity: 0.8;
	}
</style>
