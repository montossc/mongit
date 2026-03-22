<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import CommandPalette from '$lib/components/CommandPalette.svelte';
	import { registerBuiltinCommands } from '$lib/commands/commands';

	let { children } = $props();

	onMount(() => {
		registerBuiltinCommands();
	});
</script>

<CommandPalette />
{@render children()}
