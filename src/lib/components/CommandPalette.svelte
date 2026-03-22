<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/state';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { commandRegistry } from '$lib/commands/registry.svelte';
	import { CATEGORY_LABELS, type CommandContext, type Command } from '$lib/commands/types';

	let isOpen = $state(false);
	let query = $state('');
	let highlightedIndex = $state(0);
	let inputEl = $state<HTMLInputElement | null>(null);
	let listEl = $state<HTMLDivElement | null>(null);

	// ── Context ──────────────────────────────────────────────────────────

	const ctx = $derived<CommandContext>({
		repoPath: repoStore.activeRepoPath,
		currentRoute: page.url.pathname,
		hasRepo: !!repoStore.activeRepoPath,
	});

	// ── Filtered results ─────────────────────────────────────────────────

	const groups = $derived(commandRegistry.search(query, ctx));

	/** Flat list of all visible commands (for keyboard navigation). */
	const flatCommands = $derived(groups.flatMap((g) => g.commands));

	// ── Open / Close ─────────────────────────────────────────────────────

	function open() {
		isOpen = true;
		query = '';
		highlightedIndex = 0;
		// Focus input after mount
		requestAnimationFrame(() => {
			inputEl?.focus();
		});
	}

	function close() {
		isOpen = false;
		query = '';
		highlightedIndex = 0;
	}

	// ── Execution ────────────────────────────────────────────────────────

	async function executeCommand(cmd: Command) {
		close();
		await cmd.execute(ctx);
	}

	async function executeHighlighted() {
		const cmd = flatCommands[highlightedIndex];
		if (cmd) {
			await executeCommand(cmd);
		}
	}

	// ── Keyboard handling ────────────────────────────────────────────────

	function handleGlobalKeydown(e: KeyboardEvent) {
		// CMD+K to open palette (macOS)
		if (e.key === 'k' && e.metaKey && !e.shiftKey && !e.altKey) {
			// Don't open if inside a CodeMirror editor
			const target = e.target as HTMLElement;
			if (target.closest('.cm-editor')) return;

			e.preventDefault();
			if (isOpen) {
				close();
			} else {
				open();
			}
		}

		// Escape to close
		if (e.key === 'Escape' && isOpen) {
			e.preventDefault();
			close();
		}
	}

	function handlePaletteKeydown(e: KeyboardEvent) {
		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				highlightedIndex = Math.min(highlightedIndex + 1, flatCommands.length - 1);
				scrollHighlightedIntoView();
				break;

			case 'ArrowUp':
				e.preventDefault();
				highlightedIndex = Math.max(highlightedIndex - 1, 0);
				scrollHighlightedIntoView();
				break;

			case 'Enter':
				e.preventDefault();
				executeHighlighted();
				break;

			case 'Tab':
				// Trap focus inside the palette
				e.preventDefault();
				inputEl?.focus();
				break;
		}
	}

	function scrollHighlightedIntoView() {
		requestAnimationFrame(() => {
			const el = listEl?.querySelector('[data-highlighted="true"]');
			el?.scrollIntoView({ block: 'nearest' });
		});
	}

	// Reset highlight when query changes
	$effect(() => {
		// Access query to create dependency
		void query;
		highlightedIndex = 0;
	});

	// ── Lifecycle ────────────────────────────────────────────────────────

	onMount(() => {
		window.addEventListener('keydown', handleGlobalKeydown);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleGlobalKeydown);
	});
</script>

{#if isOpen}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="palette-backdrop" onclick={close} onkeydown={handlePaletteKeydown}>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="palette-panel" onclick={(e) => e.stopPropagation()}>
			<div class="palette-input-wrap">
				<svg class="palette-search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<circle cx="11" cy="11" r="8" />
					<path d="M21 21l-4.35-4.35" />
				</svg>
				<input
					bind:this={inputEl}
					bind:value={query}
					type="text"
					class="palette-input"
					placeholder="Type a command…"
					autocomplete="off"
					spellcheck="false"
				/>
				<kbd class="palette-esc">ESC</kbd>
			</div>

			<div class="palette-results" bind:this={listEl}>
				{#if flatCommands.length === 0}
					<div class="palette-empty">
						{#if query}
							No commands matching "{query}"
						{:else}
							No commands available
						{/if}
					</div>
				{:else}
					{@const flatIndex = { value: 0 }}
					{#each groups as group}
						<div class="palette-group">
							<div class="palette-group-label">{CATEGORY_LABELS[group.category]}</div>
							{#each group.commands as cmd}
								{@const idx = flatIndex.value++}
								<button
									class="palette-item"
									class:highlighted={idx === highlightedIndex}
									data-highlighted={idx === highlightedIndex}
									onmouseenter={() => { highlightedIndex = idx; }}
									onclick={() => executeCommand(cmd)}
								>
									<span class="palette-item-label">{cmd.label}</span>
									{#if cmd.shortcutHint}
										<kbd class="palette-item-shortcut">{cmd.shortcutHint}</kbd>
									{/if}
								</button>
							{/each}
						</div>
					{/each}
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.palette-backdrop {
		position: fixed;
		inset: 0;
		z-index: var(--z-overlay);
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 20vh;
	}

	.palette-panel {
		z-index: var(--z-modal);
		width: 520px;
		max-width: 90vw;
		max-height: 400px;
		background: var(--color-bg-elevated);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-lg, 12px);
		box-shadow: var(--elevation-3);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/* ── Input ──────────────────────────────────────────────────── */

	.palette-input-wrap {
		display: flex;
		align-items: center;
		padding: var(--space-4) var(--space-5);
		border-bottom: 1px solid var(--color-border);
		gap: var(--space-3);
	}

	.palette-search-icon {
		flex-shrink: 0;
		color: var(--color-text-muted);
	}

	.palette-input {
		flex: 1;
		background: none;
		border: none;
		outline: none;
		color: var(--color-text-primary);
		font-family: var(--font-sans);
		font-size: var(--text-body-size);
		line-height: 1.4;
	}

	.palette-input::placeholder {
		color: var(--color-text-muted);
	}

	.palette-esc {
		flex-shrink: 0;
		padding: 2px 6px;
		background: var(--color-bg-hover);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-xs, 3px);
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--color-text-muted);
	}

	/* ── Results ────────────────────────────────────────────────── */

	.palette-results {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-2) 0;
	}

	.palette-empty {
		padding: var(--space-6) var(--space-5);
		text-align: center;
		color: var(--color-text-muted);
		font-size: var(--text-body-sm-size);
	}

	.palette-group {
		padding: var(--space-1) 0;
	}

	.palette-group-label {
		padding: var(--space-2) var(--space-5);
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-muted);
	}

	.palette-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--space-2) var(--space-5);
		background: none;
		border: none;
		color: var(--color-text-primary);
		font-family: var(--font-sans);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		text-align: left;
		transition: background var(--transition-fast);
	}

	.palette-item:hover,
	.palette-item.highlighted {
		background: var(--color-bg-hover);
	}

	.palette-item.highlighted {
		background: var(--color-bg-active);
	}

	.palette-item-label {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.palette-item-shortcut {
		flex-shrink: 0;
		margin-left: var(--space-4);
		padding: 2px 6px;
		background: var(--color-bg-hover);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-xs, 3px);
		font-family: var(--font-mono);
		font-size: var(--text-mono-xs-size, 10px);
		color: var(--color-text-muted);
	}
</style>
