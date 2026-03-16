<script lang="ts">
	import { EditorState } from '@codemirror/state';
	import { MergeView } from '@codemirror/merge';
	import { onDestroy, onMount } from 'svelte';
	import { baseExtensions, languageExtension } from '$lib/utils/codemirror-config';
	import { mongitTheme } from '$lib/utils/codemirror-theme';

	type DiffViewerProps = {
		original?: string;
		modified?: string;
		filename?: string;
	};

	let {
		original = '',
		modified = '',
		filename = ''
	}: DiffViewerProps = $props();

	let container = $state<HTMLDivElement | null>(null);
	let mergeView: MergeView | null = null;
	let mounted = false;

	const hasContent = $derived(original !== '' || modified !== '');

	const stats = $derived.by(() => {
		if (!hasContent) return { original: 0, modified: 0, added: 0, removed: 0 };

		const originalLines = original.split('\n');
		const modifiedLines = modified.split('\n');
		const originalSet = new Set(originalLines);
		const modifiedSet = new Set(modifiedLines);
		const removed = originalLines.filter((line: string) => !modifiedSet.has(line)).length;
		const added = modifiedLines.filter((line: string) => !originalSet.has(line)).length;

		return {
			original: originalLines.length,
			modified: modifiedLines.length,
			added,
			removed
		};
	});

	function destroyMergeView() {
		mergeView?.destroy();
		mergeView = null;
	}

	function createMergeView() {
		if (!container || !hasContent) return;

		destroyMergeView();

		const extensions = [
			...baseExtensions(),
			...languageExtension(filename),
			mongitTheme,
			EditorState.readOnly.of(true)
		];

		mergeView = new MergeView({
			parent: container,
			a: { doc: original, extensions },
			b: { doc: modified, extensions },
			gutter: true,
			highlightChanges: true,
			revertControls: 'a-to-b',
			collapseUnchanged: { margin: 3, minSize: 8 }
		});
	}

	onMount(() => {
		mounted = true;
		if (hasContent) {
			createMergeView();
		}
	});

	$effect(() => {
		if (!mounted) return;
		original;
		modified;
		filename;
		if (hasContent) {
			createMergeView();
		} else {
			destroyMergeView();
		}
	});

	onDestroy(() => {
		destroyMergeView();
	});
</script>

{#if !hasContent}
	<div class="diff-empty">
		<p>Select a changed file to view its diff</p>
	</div>
{:else}
	<div class="diff-viewer">
		<header class="diff-header">
			<div class="diff-file">{filename}</div>
			<div class="diff-stats">
				<span>{stats.original} → {stats.modified} lines</span>
				<span class="added">+{stats.added}</span>
				<span class="removed">-{stats.removed}</span>
			</div>
		</header>
		<div class="diff-container" bind:this={container}></div>
	</div>
{/if}

<style>
	.diff-empty {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--color-text-muted);
		font-size: 13px;
	}

	.diff-viewer {
		display: flex;
		flex-direction: column;
		height: 100%;
		gap: var(--space-4);
	}

	.diff-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-3) var(--space-4);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		background: var(--color-bg-surface);
		font-size: 12px;
		flex-shrink: 0;
	}

	.diff-file {
		font-family: var(--font-mono);
		color: var(--color-text-primary);
	}

	.diff-stats {
		display: flex;
		gap: var(--space-4);
		color: var(--color-text-secondary);
	}

	.diff-stats .added {
		color: var(--color-diff-added-text);
	}

	.diff-stats .removed {
		color: var(--color-diff-removed-text);
	}

	.diff-container {
		flex: 1;
		min-height: 0;
		overflow: auto;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
	}

	/* Override CM6 merge view sizing */
	:global(.cm-mergeView) {
		height: 100%;
		overflow: auto;
	}

	:global(.cm-editor) {
		height: 100%;
	}
</style>
