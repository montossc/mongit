<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { watcherStore } from '$lib/stores/watcher.svelte';

	let pathInput = $state('');
	let unlisten: (() => void) | undefined;

	const statusLabel = $derived(watcherStore.watching ? 'Watching' : 'Stopped');
	const eventCount = $derived(watcherStore.events.length);

	$effect(() => {
		if (watcherStore.repoPath && !pathInput) {
			pathInput = watcherStore.repoPath;
		}
	});

	onMount(async () => {
		unlisten = await listen<void>('repo-changed', () => {
			watcherStore.addEvent();
		});
	});

	onDestroy(() => {
		unlisten?.();
	});

	async function toggleWatching(): Promise<void> {
		if (watcherStore.watching) {
			await watcherStore.stopWatching();
			return;
		}

		const trimmedPath = pathInput.trim();
		if (!trimmedPath) {
			return;
		}

		await watcherStore.startWatching(trimmedPath);
	}

	function clearLog(): void {
		watcherStore.clearEvents();
	}

	function formatTimestamp(value: Date): string {
		const hours = String(value.getHours()).padStart(2, '0');
		const minutes = String(value.getMinutes()).padStart(2, '0');
		const seconds = String(value.getSeconds()).padStart(2, '0');
		const millis = String(value.getMilliseconds()).padStart(3, '0');
		return `${hours}:${minutes}:${seconds}.${millis}`;
	}
</script>

<section class="watcher-monitor" aria-label="Watcher monitor">
	<header class="header">
		<div class="title-wrap">
			<h2 class="title">Watcher Monitor</h2>
			<p class="subtitle">FSEvents stream for repo changes</p>
		</div>
		<div class="status" data-watching={watcherStore.watching}>
			<span class="dot" aria-hidden="true"></span>
			<span>{statusLabel}</span>
		</div>
	</header>

	<div class="controls">
		<input
			class="path-input"
			type="text"
			placeholder="/path/to/repository"
			bind:value={pathInput}
			disabled={watcherStore.watching}
		/>
		<button class="btn primary" type="button" onclick={toggleWatching}>
			{watcherStore.watching ? 'Stop Watching' : 'Start Watching'}
		</button>
	</div>

	{#if watcherStore.error}
		<div class="error" role="alert">{watcherStore.error}</div>
	{/if}

	<div class="log-header">
		<span>Events: {eventCount}</span>
		<button class="btn" type="button" onclick={clearLog} disabled={eventCount === 0}>Clear Log</button>
	</div>

	<div class="log" role="log" aria-live="polite">
		{#if watcherStore.events.length === 0}
			<div class="empty">No events yet</div>
		{:else}
			<ul>
				{#each watcherStore.events as event (event.id)}
					<li>
						<span class="event-id">#{event.id}</span>
						<span class="event-time">{formatTimestamp(event.timestamp)}</span>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</section>

<style>
	.watcher-monitor {
		display: grid;
		gap: var(--space-6);
		padding: var(--space-6);
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		color: var(--color-text-primary);
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-5);
	}

	.title-wrap {
		display: grid;
		gap: var(--space-4);
	}

	.title {
		margin: 0;
		font-size: 0.95rem;
		font-weight: 600;
	}

	.subtitle {
		margin: 0;
		font-size: 0.8rem;
		color: var(--color-text-secondary);
	}

	.status {
		display: inline-flex;
		align-items: center;
		gap: var(--space-4);
		padding: 4px var(--space-5);
		border-radius: 999px;
		border: 1px solid var(--color-border);
		background: var(--color-bg-elevated);
		font-size: 0.8rem;
		color: var(--color-text-secondary);
	}

	.status[data-watching='true'] {
		color: var(--color-success);
		border-color: color-mix(in srgb, var(--color-success) 45%, var(--color-border));
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 999px;
		background: var(--color-text-muted);
	}

	.status[data-watching='true'] .dot {
		background: var(--color-success);
	}

	.controls {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		gap: var(--space-5);
	}

	.path-input {
		min-width: 0;
		padding: 10px var(--space-5);
		border-radius: var(--radius-sm);
		border: 1px solid var(--color-border);
		background: var(--color-bg);
		color: var(--color-text-primary);
		font-size: 0.85rem;
	}

	.path-input::placeholder {
		color: var(--color-text-muted);
	}

	.path-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.btn {
		padding: 8px var(--space-5);
		border-radius: var(--radius-sm);
		border: 1px solid var(--color-border);
		background: var(--color-bg-elevated);
		color: var(--color-text-primary);
		font-size: 0.82rem;
		cursor: pointer;
		transition: background var(--transition-fast), border-color var(--transition-fast);
	}

	.btn:hover:enabled {
		background: var(--color-bg-hover);
	}

	.btn:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	.btn.primary {
		border-color: color-mix(in srgb, var(--color-accent) 45%, var(--color-border));
		background: color-mix(in srgb, var(--color-accent) 16%, var(--color-bg-elevated));
	}

	.error {
		padding: 8px var(--space-5);
		border-radius: var(--radius-sm);
		border: 1px solid color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
		background: color-mix(in srgb, var(--color-danger) 14%, var(--color-bg-surface));
		color: var(--color-danger);
		font-size: 0.8rem;
	}

	.log-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		color: var(--color-text-secondary);
		font-size: 0.8rem;
	}

	.log {
		height: 240px;
		overflow: auto;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		background: var(--color-bg);
		padding: var(--space-4);
		font-family: var(--font-mono);
		font-size: 0.78rem;
	}

	.log ul {
		margin: 0;
		padding: 0;
		list-style: none;
		display: grid;
		gap: 6px;
	}

	.log li {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: var(--space-5);
		align-items: center;
		padding: 4px var(--space-4);
		border-radius: var(--radius-sm);
		background: color-mix(in srgb, var(--color-bg-elevated) 55%, transparent);
	}

	.event-id {
		color: var(--color-text-secondary);
	}

	.event-time {
		color: var(--color-text-primary);
	}

	.empty {
		display: grid;
		place-items: center;
		height: 100%;
		color: var(--color-text-muted);
	}
</style>
