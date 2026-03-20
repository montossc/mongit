<script lang="ts">
	import { repoStore } from '$lib/stores/repo.svelte';
</script>

<div class="history-workspace">
	{#if !repoStore.activeRepoPath}
		<div class="history-state">
			<p class="state-message">No repository loaded</p>
		</div>
	{:else}
		<div class="history-state">
			<p class="state-message">History workspace ready</p>
			<p class="state-hint">Graph loading will be connected next.</p>
		</div>
	{/if}
</div>

<style>
	.history-workspace {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.history-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-3);
		color: var(--color-text-muted);
	}

	.state-message {
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		margin: 0;
	}

	.state-hint {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		margin: 0;
	}
</style>
