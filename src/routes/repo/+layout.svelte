<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { repoStore } from '$lib/stores/repo.svelte';

	let { children } = $props();

	onMount(() => {
		if (!repoStore.activeRepoPath) {
			goto('/');
		}
	});

	const tabs = [
		{ label: 'Summary', href: '/repo' },
		{ label: 'Changes', href: '/repo/changes' },
	] as const;
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
		</header>

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
