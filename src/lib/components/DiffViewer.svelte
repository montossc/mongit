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

	const SAMPLE_FILE_NAME = 'src/lib/utils/build-graph-layout.ts';

	const SAMPLE_ORIGINAL = `type Node = {
	id: string;
	parents: string[];
	author: string;
	timestamp: number;
	branch: string;
};

type Point = {
	x: number;
	y: number;
};

export type GraphLayout = {
	lanes: Map<string, number>;
	positions: Map<string, Point>;
	maxLane: number;
};

export function buildGraphLayout(nodes: Node[]): GraphLayout {
	const lanes = new Map<string, number>();
	const positions = new Map<string, Point>();
	const activeLanes: string[] = [];
	let nextLane = 0;
	let maxLane = 0;

	for (let index = 0; index < nodes.length; index += 1) {
		const node = nodes[index];
		let lane = activeLanes.indexOf(node.id);

		if (lane === -1) {
			lane = nextLane;
			activeLanes[lane] = node.id;
			nextLane += 1;
		}

		lanes.set(node.id, lane);
		positions.set(node.id, {
			x: lane * 24 + 16,
			y: index * 28 + 20
		});

		for (const parentId of node.parents) {
			const parentLane = activeLanes.indexOf(parentId);
			if (parentLane === -1) {
				activeLanes[lane] = parentId;
			} else if (parentLane !== lane) {
				activeLanes[parentLane] = activeLanes[lane];
				activeLanes[lane] = parentId;
			}
		}

		maxLane = Math.max(maxLane, lane);
	}

	return { lanes, positions, maxLane };
}`;

	const SAMPLE_MODIFIED = `type Node = {
	id: string;
	parents: string[];
	author: string;
	timestamp: number;
	branch: string;
	isHead?: boolean;
};

type Point = {
	x: number;
	y: number;
};

export type GraphLayout = {
	lanes: Map<string, number>;
	positions: Map<string, Point>;
	maxLane: number;
	laneWidth: number;
};

const LANE_WIDTH = 26;
const ROW_HEIGHT = 30;
const OFFSET_X = 18;
const OFFSET_Y = 22;

export function buildGraphLayout(nodes: Node[]): GraphLayout {
	const lanes = new Map<string, number>();
	const positions = new Map<string, Point>();
	const activeLanes: string[] = [];
	let nextLane = 0;
	let maxLane = 0;

	for (let index = 0; index < nodes.length; index += 1) {
		const node = nodes[index];
		let lane = activeLanes.indexOf(node.id);

		if (lane < 0) {
			lane = nextLane;
			activeLanes[lane] = node.id;
			nextLane += 1;
		}

		lanes.set(node.id, lane);
		positions.set(node.id, {
			x: lane * LANE_WIDTH + OFFSET_X,
			y: index * ROW_HEIGHT + OFFSET_Y
		});

		for (const parentId of node.parents) {
			const parentLane = activeLanes.indexOf(parentId);
			if (parentLane < 0) {
				activeLanes[lane] = parentId;
				continue;
			}

			if (parentLane !== lane) {
				activeLanes[parentLane] = activeLanes[lane];
				activeLanes[lane] = parentId;
			}
		}

		maxLane = Math.max(maxLane, lane);
	}

	return { lanes, positions, maxLane, laneWidth: LANE_WIDTH };
}`;

	let { original = SAMPLE_ORIGINAL, modified = SAMPLE_MODIFIED, filename = SAMPLE_FILE_NAME }: DiffViewerProps =
		$props();

	let container = $state<HTMLDivElement | null>(null);
	let mergeView: MergeView | null = null;
	let mounted = false;

	const stats = $derived.by(() => {
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
		if (!container) return;

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
		createMergeView();
	});

	$effect(() => {
		if (!mounted) return;
		original;
		modified;
		filename;
		createMergeView();
	});

	onDestroy(() => {
		destroyMergeView();
	});
</script>

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
</style>
