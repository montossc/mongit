<script module lang="ts">
	/**
	 * Discriminated union for DiffViewer shell states.
	 * Callers own the state; the component renders accordingly.
	 */
	export type DiffViewerState =
		| { kind: 'loading' }
		| { kind: 'empty'; message?: string }
		| { kind: 'error'; message: string }
		| { kind: 'ready'; original: string; modified: string; filename: string };
</script>

<script lang="ts">
	import { EditorState } from '@codemirror/state';
	import { MergeView } from '@codemirror/merge';
	import { baseExtensions, languageExtension } from '$lib/utils/codemirror-config';
	import { mongitTheme } from '$lib/utils/codemirror-theme';

	type DiffViewerProps = {
		view: DiffViewerState;
	};

	let { view }: DiffViewerProps = $props();

	let container = $state<HTMLDivElement | null>(null);

	const stats = $derived.by(() => {
		if (view.kind !== 'ready') return null;
		const originalLines = view.original.split('\n');
		const modifiedLines = view.modified.split('\n');
		const originalSet = new Set(originalLines);
		const modifiedSet = new Set(modifiedLines);
		return {
			original: originalLines.length,
			modified: modifiedLines.length,
			added: modifiedLines.filter((line) => !originalSet.has(line)).length,
			removed: originalLines.filter((line) => !modifiedSet.has(line)).length
		};
	});

	/**
	 * Single $effect manages the full MergeView lifecycle:
	 * - Creates MergeView when view is 'ready' and container exists
	 * - Cleanup destroys the view on re-run, state change, or unmount
	 * - No separate onMount/onDestroy needed
	 */
	$effect(() => {
		if (view.kind !== 'ready' || !container) return;

		const extensions = [
			...baseExtensions(),
			...languageExtension(view.filename),
			mongitTheme,
			EditorState.readOnly.of(true)
		];

		const mv = new MergeView({
			parent: container,
			a: { doc: view.original, extensions },
			b: { doc: view.modified, extensions },
			gutter: true,
			highlightChanges: true,
			revertControls: 'a-to-b',
			collapseUnchanged: { margin: 3, minSize: 8 }
		});

		return () => mv.destroy();
	});
</script>

{#if view.kind === 'loading'}
	<div class="diff-viewer">
		<div class="diff-placeholder">
			<div class="diff-spinner"></div>
			<span class="diff-placeholder-text">Loading diff&hellip;</span>
		</div>
	</div>
{:else if view.kind === 'empty'}
	<div class="diff-viewer">
		<div class="diff-placeholder">
			<span class="diff-placeholder-text">{view.message ?? 'No changes to display'}</span>
		</div>
	</div>
{:else if view.kind === 'error'}
	<div class="diff-viewer">
		<div class="diff-placeholder diff-error">
			<span class="diff-error-label">Error</span>
			<span class="diff-placeholder-text">{view.message}</span>
		</div>
	</div>
{:else}
	<div class="diff-viewer">
		<header class="diff-header">
			<div class="diff-file">{view.filename}</div>
			{#if stats}
				<div class="diff-stats">
					<span>{stats.original} &rarr; {stats.modified} lines</span>
					<span class="added">+{stats.added}</span>
					<span class="removed">-{stats.removed}</span>
				</div>
			{/if}
		</header>
		<div class="diff-container" bind:this={container}></div>
	</div>
{/if}

<style>
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
		height: 100%;
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

	/* Fallback states */
	.diff-placeholder {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: var(--space-4);
		padding: var(--space-8);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		background: var(--color-bg-surface);
	}

	.diff-placeholder-text {
		font-size: 13px;
		color: var(--color-text-secondary);
	}

	.diff-error {
		border-color: var(--color-diff-removed-text);
	}

	.diff-error-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-diff-removed-text);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.diff-spinner {
		width: 20px;
		height: 20px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
