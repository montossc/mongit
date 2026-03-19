<script lang="ts">
	import { repoStore } from '$lib/stores/repo.svelte';
</script>

<div class="repo-landing">
	<div class="repo-summary">
		<h2>Repository opened</h2>

		{#if repoStore.repoStatus}
			<div class="status-grid">
				<div class="status-item">
					<span class="status-label">Branch</span>
					<span class="status-value mono">
						{repoStore.repoStatus.branch ?? 'detached'}
					</span>
				</div>
				<div class="status-item">
					<span class="status-label">Changed files</span>
					<span class="status-value">{repoStore.repoStatus.changed_files}</span>
				</div>
				<div class="status-item">
					<span class="status-label">Staged files</span>
					<span class="status-value">{repoStore.repoStatus.staged_files}</span>
				</div>
			</div>
		{/if}

		<p class="placeholder-note">
			Workspace features will be added in upcoming updates.
		</p>
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

	.repo-summary {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-6);
		max-width: 400px;
		text-align: center;
	}

	.repo-summary h2 {
		font-size: 18px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0;
	}

	.status-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-4);
		width: 100%;
	}

	.status-item {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		padding: var(--space-4);
		background: var(--color-bg-surface);
		border-radius: var(--radius-md);
		border: 1px solid var(--color-border);
	}

	.status-label {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.status-value {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.status-value.mono {
		font-family: var(--font-mono);
		font-size: var(--text-mono-size);
	}

	.placeholder-note {
		font-size: var(--text-body-sm-size);
		color: var(--color-text-muted);
		margin: 0;
	}
</style>
