<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { fade, fly } from 'svelte/transition';
	import { page } from '$app/state';
	import { repoStore } from '$lib/stores/repo.svelte';
	import { commandRegistry } from '$lib/commands/registry.svelte';
	import { CATEGORY_LABELS, type CommandContext, type Command } from '$lib/commands/types';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { setPaletteCallbacks } from '$lib/commands/shortcuts';

	let isOpen = $state(false);
	let query = $state('');
	let highlightedIndex = $state(0);
	let inputEl = $state<HTMLInputElement | null>(null);
	let listEl = $state<HTMLDivElement | null>(null);
	let previouslyFocused: HTMLElement | null = null;

	// ── ARIA IDs ─────────────────────────────────────────────────────────
	const listboxId = 'palette-listbox';
	const optionIdPrefix = 'palette-opt-';

	// ── Context ──────────────────────────────────────────────────────────

	const ctx = $derived<CommandContext>({
		repoPath: repoStore.activeRepoPath,
		currentRoute: page.url.pathname,
		hasRepo: !!repoStore.activeRepoPath,
		currentBranch: repoStore.repoStatus?.branch ?? null,
		hasChanges: (repoStore.repoStatus?.changed_files ?? 0) > 0,
	});

	// ── Recently used (shown when query is empty) ────────────────────────

	const recentCommands = $derived(commandRegistry.getRecent(ctx));
	const showRecent = $derived(!query.trim() && recentCommands.length > 0);

	// ── Filtered results ─────────────────────────────────────────────────

	const groups = $derived(commandRegistry.search(query, ctx));

	/** Flat list for keyboard navigation. Recents first, then category groups. */
	interface FlatItem {
		cmd: Command;
		matchIndices: number[];
	}

	const flatItems = $derived.by(() => {
		const items: FlatItem[] = [];
		if (showRecent) {
			for (const cmd of recentCommands) {
				items.push({ cmd, matchIndices: [] });
			}
		}
		for (const group of groups) {
			for (const mc of group.commands) {
				items.push({ cmd: mc.cmd, matchIndices: mc.matchIndices });
			}
		}
		return items;
	});

	const activeDescendant = $derived(
		flatItems.length > 0 ? `${optionIdPrefix}${highlightedIndex}` : undefined
	);

	// ── Open / Close ─────────────────────────────────────────────────────

	function open() {
		previouslyFocused = document.activeElement as HTMLElement;
		isOpen = true;
		query = '';
		highlightedIndex = 0;
		requestAnimationFrame(() => {
			inputEl?.focus();
		});
	}

	function close() {
		isOpen = false;
		query = '';
		highlightedIndex = 0;
		previouslyFocused?.focus();
		previouslyFocused = null;
	}

	function toggle() {
		if (isOpen) close(); else open();
	}

	// ── Execution (with error feedback) ──────────────────────────────────

	async function executeItem(cmd: Command) {
		close();
		try {
			await commandRegistry.execute(cmd.id, ctx);
		} catch (e) {
			toastStore.error(e instanceof Error ? e.message : String(e));
		}
	}

	async function executeHighlighted() {
		const item = flatItems[highlightedIndex];
		if (item) await executeItem(item.cmd);
	}

	// ── Keyboard handling ────────────────────────────────────────────────

	function handleGlobalKeydown(e: KeyboardEvent) {
		// Escape to close (CMD+K is handled by the shortcut manager)
		if (e.key === 'Escape' && isOpen) {
			e.preventDefault();
			close();
		}
	}

	function handleInputKeydown(e: KeyboardEvent) {
		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				highlightedIndex = Math.min(highlightedIndex + 1, flatItems.length - 1);
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
		void query;
		highlightedIndex = 0;
	});

	// ── Lifecycle ────────────────────────────────────────────────────────

	onMount(() => {
		window.addEventListener('keydown', handleGlobalKeydown);
		setPaletteCallbacks(() => isOpen, toggle);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleGlobalKeydown);
	});
</script>

{#snippet highlightedLabel(label: string, indices: number[])}
	{#if indices.length > 0}
		{#each label.split('') as char, i}{#if indices.includes(i)}<mark class="palette-match">{char}</mark>{:else}{char}{/if}{/each}
	{:else}
		{label}
	{/if}
{/snippet}

{#if isOpen}
	<div class="palette-overlay">
		<button
			class="palette-backdrop"
			tabindex="-1"
			aria-label="Close command palette"
			onclick={close}
			transition:fade={{ duration: 150 }}
		></button>

		<div
			class="palette-panel"
			role="dialog"
			aria-label="Command palette"
			aria-modal="true"
			transition:fly={{ y: -8, duration: 150 }}
		>
			<div class="palette-input-wrap">
				<svg class="palette-search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
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
					role="combobox"
					aria-expanded="true"
					aria-controls={listboxId}
					aria-activedescendant={activeDescendant}
					aria-autocomplete="list"
					aria-label="Search commands"
					onkeydown={handleInputKeydown}
				/>
				<kbd class="palette-esc" aria-hidden="true">ESC</kbd>
			</div>

			<div class="palette-results" bind:this={listEl} id={listboxId} role="listbox" aria-label="Commands">
				{#if flatItems.length === 0}
					<div class="palette-empty" role="status">
						{#if query}
							No commands matching "{query}"
						{:else}
							No commands available
						{/if}
					</div>
				{:else}
					{@const flatIdx = { value: 0 }}

					{#if showRecent}
						<div class="palette-group">
							<div class="palette-group-label">Recently Used</div>
							{#each recentCommands as cmd}
								{@const idx = flatIdx.value++}
								<button
									id="{optionIdPrefix}{idx}"
									role="option"
									aria-selected={idx === highlightedIndex}
									class="palette-item"
									class:highlighted={idx === highlightedIndex}
									data-highlighted={idx === highlightedIndex}
									onmouseenter={() => { highlightedIndex = idx; }}
									onclick={() => executeItem(cmd)}
								>
									<div class="palette-item-content">
										<span class="palette-item-label">{cmd.label}</span>
										{#if cmd.description}
											<span class="palette-item-description">{cmd.description}</span>
										{/if}
									</div>
									{#if cmd.shortcutHint}
										<kbd class="palette-item-shortcut">{cmd.shortcutHint}</kbd>
									{/if}
								</button>
							{/each}
						</div>
					{/if}

					{#each groups as group}
						<div class="palette-group">
							<div class="palette-group-label">{CATEGORY_LABELS[group.category]}</div>
							{#each group.commands as mc}
								{@const idx = flatIdx.value++}
								<button
									id="{optionIdPrefix}{idx}"
									role="option"
									aria-selected={idx === highlightedIndex}
									class="palette-item"
									class:highlighted={idx === highlightedIndex}
									data-highlighted={idx === highlightedIndex}
									onmouseenter={() => { highlightedIndex = idx; }}
									onclick={() => executeItem(mc.cmd)}
								>
									<div class="palette-item-content">
										<span class="palette-item-label">
											{@render highlightedLabel(mc.cmd.label, mc.matchIndices)}
										</span>
										{#if mc.cmd.description}
											<span class="palette-item-description">{mc.cmd.description}</span>
										{/if}
									</div>
									{#if mc.cmd.shortcutHint}
										<kbd class="palette-item-shortcut">{mc.cmd.shortcutHint}</kbd>
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
	.palette-overlay {
		position: fixed;
		inset: 0;
		z-index: var(--z-overlay);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 20vh;
	}

	.palette-backdrop {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		border: none;
		padding: 0;
		margin: 0;
		cursor: default;
		-webkit-appearance: none;
		appearance: none;
	}

	.palette-panel {
		position: relative;
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
		gap: var(--space-3);
		transition: background var(--transition-fast);
	}

	.palette-item:hover,
	.palette-item.highlighted {
		background: var(--color-bg-hover);
	}

	.palette-item.highlighted {
		background: var(--color-bg-active);
	}

	.palette-item-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.palette-item-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.palette-item-description {
		font-size: 11px;
		line-height: 1.2;
		color: var(--color-text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.palette-item-shortcut {
		flex-shrink: 0;
		padding: 2px 6px;
		background: var(--color-bg-hover);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-xs, 3px);
		font-family: var(--font-mono);
		font-size: var(--text-mono-xs-size, 10px);
		color: var(--color-text-muted);
	}

	/* ── Fuzzy match highlight ──────────────────────────────────── */

	.palette-match {
		background: none;
		color: var(--color-accent);
		font-weight: 600;
	}
</style>
