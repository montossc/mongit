<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { conflictStore } from '$lib/stores/conflict.svelte';
	import { syncStore } from '$lib/stores/sync.svelte';

	let { children } = $props();

	onMount(() => {
		if (!repoStore.activeRepoPath) {
			goto('/');
			return;
		}
		// Load tracking info on mount
		syncStore.refreshAheadBehind(repoStore.activeRepoPath);
		// Load merge state so Resolve tab appears when merging
		conflictStore.loadMergeState(repoStore.activeRepoPath);
	});

	const tabs = $derived([
		{ label: 'Summary', href: '/repo' },
		{ label: 'Changes', href: '/repo/changes' },
		...(conflictStore.isMerging
			? [{ label: `Resolve (${conflictStore.conflictCount})`, href: '/repo/resolve' }]
			: []),
	] as const);

	// ── Sync handlers with auto-refresh ─────────────────────────────────

	async function handleFetch() {
		if (!repoStore.activeRepoPath) return;
		await syncStore.fetchOrigin(repoStore.activeRepoPath);
	}

	async function handlePull() {
		if (!repoStore.activeRepoPath) return;
		const success = await syncStore.pullOrigin(repoStore.activeRepoPath);
		if (success) {
			// Pull may change working tree — refresh repo status
			await repoStore.openRepo(repoStore.activeRepoPath);
		}
	}

	async function handlePush() {
		if (!repoStore.activeRepoPath) return;
		await syncStore.pushOrigin(repoStore.activeRepoPath);
	}
</script>

{#if repoStore.activeRepoPath}
	<div class="repo-shell">
		<header class="repo-toolbar">
			<div class="repo-toolbar-left">
				<button class="back-btn" onclick={() => goto('/')} title="Back to home">
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M19 12H5M12 19l-7-7 7-7" />
					</svg>
				</button>
				<h1 class="repo-name">{repoStore.activeRepoName}</h1>
				{#if repoStore.repoStatus?.branch}
					<span class="branch-label">{repoStore.repoStatus.branch}</span>
				{/if}
			</div>

			<div class="repo-toolbar-right">
				{#if syncStore.aheadBehind?.upstream}
					<span class="tracking-badge" title="{syncStore.aheadBehind.upstream}">
						{#if syncStore.aheadBehind.behind > 0}
							<span class="tracking-behind">&darr;{syncStore.aheadBehind.behind}</span>
						{/if}
						{#if syncStore.aheadBehind.ahead > 0}
							<span class="tracking-ahead">&uarr;{syncStore.aheadBehind.ahead}</span>
						{/if}
						{#if syncStore.aheadBehind.ahead === 0 && syncStore.aheadBehind.behind === 0}
							<span class="tracking-synced">&check;</span>
						{/if}
					</span>
				{/if}

				{#if syncStore.lastResult}
					<span class="sync-success">
						{syncStore.lastResult.op === 'fetch' ? 'Fetched' : syncStore.lastResult.op === 'pull' ? 'Pulled' : 'Pushed'}
					</span>
				{/if}

				<button
					class="sync-btn"
					onclick={handleFetch}
					disabled={syncStore.busy}
					title="Fetch from origin"
				>
					{#if syncStore.fetching}
						<span class="spinner"></span>
					{:else}
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M21 12a9 9 0 0 0-9-9m9 9a9 9 0 0 1-9 9m9-9H3m9-9a9 9 0 0 0-9 9m9-9c1.66 0 3 4.03 3 9s-1.34 9-3 9m0-18c-1.66 0-3 4.03-3 9s1.34 9 3 9" />
						</svg>
					{/if}
					Fetch
				</button>

				<button
					class="sync-btn"
					onclick={handlePull}
					disabled={syncStore.busy}
					title="Pull from origin"
				>
					{#if syncStore.pulling}
						<span class="spinner"></span>
					{:else}
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M12 5v14M5 12l7 7 7-7" />
						</svg>
					{/if}
					Pull
				</button>

				<button
					class="sync-btn"
					onclick={handlePush}
					disabled={syncStore.busy}
					title="Push to origin"
				>
					{#if syncStore.pushing}
						<span class="spinner"></span>
					{:else}
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M12 19V5M5 12l7-7 7 7" />
						</svg>
					{/if}
					Push
				</button>
			</div>
		</header>

		{#if syncStore.error}
			<div class="sync-error-banner">
				<span class="sync-error-text">{syncStore.error}</span>
				<button class="sync-error-dismiss" onclick={() => syncStore.clearError()} title="Dismiss">
					&times;
				</button>
			</div>
		{/if}

		<nav class="repo-tabs">
			{#each tabs as tab}
				<a
					href={tab.href}
					class="repo-tab"
					class:active={page.url.pathname === tab.href}
					onclick={(e) => { e.preventDefault(); goto(tab.href); }}
				>
					{tab.label}
				</a>
			{/each}
		</nav>

		<div class="repo-content">
			{@render children()}
		</div>
	</div>
{:else}
	<div class="repo-guard">
		<p>No repository loaded.</p>
		<button onclick={() => goto('/')}>Go to Home</button>
	</div>
{/if}

<style>
	.repo-shell {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--color-bg);
		color: var(--color-text-primary);
	}

	.repo-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-6);
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
		-webkit-app-region: drag;
	}

	.repo-toolbar-left {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		-webkit-app-region: no-drag;
	}

	.repo-toolbar-right {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		-webkit-app-region: no-drag;
	}

	.back-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
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

	.repo-name {
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 700;
		color: var(--color-accent);
		margin: 0;
	}

	.branch-label {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		color: var(--color-text-secondary);
		padding: var(--space-1) var(--space-3);
		background: var(--color-bg-elevated);
		border-radius: var(--radius-sm);
	}

	/* ── Tracking badge ────────────────────────────────────────── */

	.tracking-badge {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		padding: var(--space-1) var(--space-3);
		background: var(--color-bg-elevated);
		border-radius: var(--radius-sm);
		color: var(--color-text-secondary);
	}

	.tracking-behind {
		color: var(--color-warning, #e5a820);
	}

	.tracking-ahead {
		color: var(--color-accent);
	}

	.tracking-synced {
		color: var(--color-success, #4caf50);
	}

	/* ── Sync buttons ──────────────────────────────────────────── */

	.sync-btn {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		height: 28px;
		padding: 0 var(--space-3);
		background: var(--color-bg-elevated);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-primary);
		font-family: var(--font-sans);
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		cursor: pointer;
		transition: background var(--transition-fast), color var(--transition-fast);
	}

	.sync-btn:hover:not(:disabled) {
		background: var(--color-bg-active);
	}

	.sync-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* ── Spinner ───────────────────────────────────────────────── */

	.spinner {
		width: 14px;
		height: 14px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	/* ── Success status ─────────────────────────────────────────── */

	.sync-success {
		font-size: var(--text-body-sm-size);
		color: var(--color-success, #4caf50);
		font-weight: 500;
	}

	/* ── Error banner ──────────────────────────────────────────── */

	.sync-error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-2) var(--space-6);
		background: var(--color-danger-bg, rgba(220, 38, 38, 0.1));
		border-bottom: 1px solid var(--color-danger, #dc2626);
		flex-shrink: 0;
	}

	.sync-error-text {
		font-size: var(--text-body-sm-size);
		color: var(--color-danger, #dc2626);
	}

	.sync-error-dismiss {
		background: none;
		border: none;
		color: var(--color-danger, #dc2626);
		font-size: 18px;
		cursor: pointer;
		padding: 0 var(--space-2);
		line-height: 1;
	}

	.sync-error-dismiss:hover {
		opacity: 0.7;
	}

	/* ── Tabs ──────────────────────────────────────────────────── */

	.repo-tabs {
		display: flex;
		gap: 0;
		padding: 0 var(--space-6);
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	.repo-tab {
		padding: var(--space-3) var(--space-5);
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		color: var(--color-text-secondary);
		text-decoration: none;
		border-bottom: 2px solid transparent;
		transition: color var(--transition-fast), border-color var(--transition-fast);
		cursor: pointer;
	}

	.repo-tab:hover {
		color: var(--color-text-primary);
	}

	.repo-tab.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}

	.repo-content {
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.repo-guard {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100vh;
		gap: var(--space-4);
		color: var(--color-text-muted);
	}

	.repo-guard button {
		background: var(--color-accent);
		color: white;
		border: none;
		padding: var(--space-3) var(--space-5);
		border-radius: var(--radius-sm);
		cursor: pointer;
		font-size: var(--text-body-sm-size);
	}
</style>
