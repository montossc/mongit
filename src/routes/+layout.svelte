<script lang="ts">
	import '../app.css';
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/state';
	import CommandPalette from '$lib/components/CommandPalette.svelte';
	import Toast from '$lib/components/Toast.svelte';
	import { registerBuiltinCommands } from '$lib/commands/commands';
	import { registerShortcuts, destroyShortcuts, CORE_SHORTCUTS } from '$lib/commands/shortcuts';
	import { repoStore } from '$lib/stores/repo.svelte';
	import type { CommandContext } from '$lib/commands/types';

	let { children } = $props();

	function getContext(): CommandContext {
		return {
			repoPath: repoStore.activeRepoPath,
			currentRoute: page.url.pathname,
			hasRepo: !!repoStore.activeRepoPath,
			currentBranch: repoStore.repoStatus?.branch ?? null,
			hasChanges: (repoStore.repoStatus?.changed_files ?? 0) > 0,
		};
	}

	onMount(() => {
		registerBuiltinCommands();
		registerShortcuts(CORE_SHORTCUTS, getContext);
	});

	onDestroy(() => {
		destroyShortcuts();
	});
</script>

<CommandPalette />
<Toast />
{@render children()}
