<script lang="ts">
	import { onMount } from 'svelte';

	let greeting = $state('');

	onMount(async () => {
		// Test Tauri IPC when running inside Tauri
		if ('__TAURI_INTERNALS__' in window) {
			const { invoke } = await import('@tauri-apps/api/core');
			greeting = await invoke('greet', { name: 'mongit' });
		} else {
			greeting = 'Running in browser (no Tauri shell)';
		}
	});
</script>

<main>
	<div class="container">
		<h1>mongit</h1>
		<p class="subtitle">A standalone Git client for macOS</p>
		<p class="status">{greeting}</p>
	</div>
</main>

<style>
	main {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100vh;
	}

	.container {
		text-align: center;
	}

	h1 {
		font-size: 2.5rem;
		font-weight: 700;
		color: var(--color-accent);
		margin-bottom: var(--space-sm);
	}

	.subtitle {
		color: var(--color-text-secondary);
		font-size: 1.1rem;
		margin-bottom: var(--space-lg);
	}

	.status {
		color: var(--color-text-muted);
		font-family: var(--font-mono);
		font-size: 0.85rem;
	}
</style>
