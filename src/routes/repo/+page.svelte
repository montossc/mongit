<script lang="ts">
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
				<span class="status-placeholder">…</span>
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
				class:staged={staged > 0}
			>
				{#if changed === 0 && staged === 0}
					Working tree clean
				{:else if changed > 0 && staged === 0}
					Unstaged changes
				{:else if staged > 0}
					Ready to commit
				{/if}
			</p>
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
		font-weight: 700;
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

	.state-message.staged {
		color: var(--color-accent);
	}

	.fallback-message {
		font-size: var(--text-body-sm-size);
		color: var(--color-text-muted);
		margin: 0;
	}
</style>
