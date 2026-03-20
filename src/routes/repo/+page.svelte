<script lang="ts">
	import { goto } from '$app/navigation';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { Badge } from '$lib/components/ui';
</script>

<div class="repo-landing">
	<div class="summary">
		<!-- Repo Identity -->
		<div class="identity">
			<h2 class="repo-title">{repoStore.activeRepoName ?? 'Unknown'}</h2>
			{#if repoStore.activeRepoPath}
				<span class="repo-path">{repoStore.activeRepoPath}</span>
			{/if}
		</div>

		<!-- Branch Context -->
		<div class="branch-context">
			{#if repoStore.repoStatus}
				{#if repoStore.repoStatus.branch}
					<Badge variant="branch">{repoStore.repoStatus.branch}</Badge>
				{:else}
					<span class="detached-badge">Detached HEAD</span>
				{/if}
			{:else}
				<span class="status-placeholder" aria-label="Loading branch status">…</span>
			{/if}
		</div>

		<!-- Working-Tree State -->
		{#if repoStore.repoStatus}
			<div class="state-cards">
				<div class="stat-card">
					<span class="stat-value">{repoStore.repoStatus.changed_files}</span>
					<span class="stat-label">Changed</span>
				</div>
				<div class="stat-card">
					<span class="stat-value">{repoStore.repoStatus.staged_files}</span>
					<span class="stat-label">Staged</span>
				</div>
			</div>

			<!-- State Message -->
			{@const changed = repoStore.repoStatus.changed_files}
			{@const staged = repoStore.repoStatus.staged_files}
			<p
				class="state-message"
				class:clean={changed === 0 && staged === 0}
				class:unstaged={changed > 0 && staged === 0}
				class:partial={changed > 0 && staged > 0}
				class:staged={changed === 0 && staged > 0}
			>
				{#if changed === 0 && staged === 0}
					Working tree clean
				{:else if changed > 0 && staged === 0}
					Unstaged changes
				{:else if changed > 0 && staged > 0}
					Partially staged
				{:else if staged > 0}
					Ready to commit
				{/if}
			</p>

			{#if (repoStore.repoStatus?.changed_files ?? 0) > 0 || (repoStore.repoStatus?.staged_files ?? 0) > 0}
				<button class="view-changes-link" onclick={() => goto('/repo/changes')}>
					View changes →
				</button>
			{/if}
		{:else}
			<p class="fallback-message">Loading status…</p>
		{/if}
	</div>
</div>

<style>
	.repo-landing {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		padding: var(--space-8);
		overflow-y: auto;
	}

	.summary {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-6);
		max-width: 480px;
		width: 100%;
		text-align: center;
	}

	/* ── Identity ── */
	.identity {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
	}

	.repo-title {
		font-family: var(--font-display);
		font-size: var(--text-heading-lg-size);
		font-weight: var(--text-heading-lg-weight);
		line-height: var(--text-heading-lg-leading);
		color: var(--color-text-primary);
		margin: 0;
	}

	.repo-path {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		color: var(--color-text-muted);
		word-break: break-all;
	}

	/* ── Branch ── */
	.branch-context {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.detached-badge {
		display: inline-flex;
		align-items: center;
		height: var(--size-badge);
		padding: var(--space-1) var(--space-3);
		font-size: var(--text-caption-size);
		font-weight: 500;
		border-radius: var(--radius-full);
		white-space: nowrap;
		line-height: 1;
		background: var(--color-warning-muted);
		color: var(--color-warning);
	}

	.status-placeholder {
		color: var(--color-text-muted);
		font-size: var(--text-body-sm-size);
	}

	/* ── State Cards ── */
	.state-cards {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-4);
		width: 100%;
		max-width: 280px;
	}

	.stat-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-5);
		background: var(--color-bg-surface);
		border-radius: var(--radius-md);
		border: 1px solid var(--color-border);
	}

	.stat-value {
		font-size: var(--text-heading-lg-size);
		font-weight: var(--text-heading-lg-weight);
		color: var(--color-text-primary);
	}

	.stat-label {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	/* ── State Message ── */
	.state-message {
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		margin: 0;
	}

	.state-message.clean {
		color: var(--color-success);
	}

	.state-message.unstaged {
		color: var(--color-warning);
	}

	.state-message.partial {
		color: var(--color-info);
	}

	.state-message.staged {
		color: var(--color-accent);
	}

	.fallback-message {
		font-size: var(--text-body-sm-size);
		color: var(--color-text-muted);
		margin: 0;
	}

	.view-changes-link {
		margin-top: var(--space-4);
		padding: var(--space-3) var(--space-5);
		background: none;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-accent);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.view-changes-link:hover {
		background: var(--color-bg-hover);
	}
</style>
