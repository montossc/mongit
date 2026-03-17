<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import DiffViewer, { type DiffViewerState } from '$lib/components/DiffViewer.svelte';
	import MergeEditor from '$lib/components/MergeEditor.svelte';
	import WatcherMonitor from '$lib/components/WatcherMonitor.svelte';
	import BenchmarkPanel from '$lib/components/BenchmarkPanel.svelte';

	// ── Sample diff data (callers own state, not DiffViewer) ────────────

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

	// ── Shell state harness ────────────────────────────────────────────

	const shellStates = ['loading', 'empty', 'error', 'ready'] as const;
	type ShellStateKind = (typeof shellStates)[number];

	let activeShellState = $state<ShellStateKind>('ready');

	const diffView = $derived.by<DiffViewerState>(() => {
		switch (activeShellState) {
			case 'loading':
				return { kind: 'loading' };
			case 'empty':
				return { kind: 'empty', message: 'No working tree changes detected' };
			case 'error':
				return { kind: 'error', message: 'Failed to parse diff: invalid hunk header at line 42' };
			case 'ready':
				return {
					kind: 'ready',
					original: SAMPLE_ORIGINAL,
					modified: SAMPLE_MODIFIED,
					filename: SAMPLE_FILE_NAME
				};
		}
	});

	// ── Tabs ────────────────────────────────────────────────────────────

	const tabs = [
		{ id: 'diff', label: 'Diff Viewer' },
		{ id: 'merge', label: 'Merge Editor' },
		{ id: 'watcher', label: 'File Watcher' },
		{ id: 'benchmarks', label: 'Benchmarks' },
	] as const;

	type TabId = (typeof tabs)[number]['id'];

	let activeTab = $state<TabId>('diff');

	function selectTab(id: TabId) {
		activeTab = id;
	}

	function handleKeydown(e: KeyboardEvent) {
		// Cmd+1-4 to switch tabs
		if (e.metaKey && e.key >= '1' && e.key <= '4') {
			e.preventDefault();
			const idx = parseInt(e.key) - 1;
			if (idx < tabs.length) {
				selectTab(tabs[idx].id);
			}
		}
	}

	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
	});

	onDestroy(() => {
		window.removeEventListener('keydown', handleKeydown);
	});
</script>

<main class="spike-layout">
	<header class="spike-header">
		<div class="header-left">
			<a href="/" class="back-link">&larr; Back</a>
			<h1 class="spike-title">Spike D — CodeMirror 6 + File Watcher</h1>
			<span class="spike-badge">bd-2n2</span>
		</div>
	</header>

	<div class="tab-bar" role="tablist">
		{#each tabs as tab, idx}
			<button
				role="tab"
				class="tab"
				class:active={activeTab === tab.id}
				aria-selected={activeTab === tab.id}
				onclick={() => selectTab(tab.id)}
			>
				{tab.label}
				<span class="tab-shortcut">⌘{idx + 1}</span>
			</button>
		{/each}
	</div>

	<div class="tab-content">
		{#if activeTab === 'diff'}
			<div class="tab-panel visible" role="tabpanel">
				<div class="shell-harness">
					<div class="state-selector">
						<span class="state-selector-label">Shell state:</span>
						{#each shellStates as kind}
							<button
								class="state-btn"
								class:active={activeShellState === kind}
								onclick={() => (activeShellState = kind)}
							>
								{kind}
							</button>
						{/each}
					</div>
					<div class="diff-area">
						<DiffViewer view={diffView} />
					</div>
				</div>
			</div>
		{:else if activeTab === 'merge'}
			<div class="tab-panel visible" role="tabpanel">
				<MergeEditor />
			</div>
		{:else if activeTab === 'watcher'}
			<div class="tab-panel visible" role="tabpanel">
				<WatcherMonitor />
			</div>
		{:else if activeTab === 'benchmarks'}
			<div class="tab-panel visible" role="tabpanel">
				<BenchmarkPanel />
			</div>
		{/if}
	</div>
</main>

<style>
	.spike-layout {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--color-bg);
		color: var(--color-text-primary);
	}

	.spike-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-6);
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
		-webkit-app-region: drag;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		-webkit-app-region: no-drag;
	}

	.back-link {
		font-size: 12px;
		color: var(--color-accent);
		text-decoration: none;
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-sm);
		transition: background var(--transition-fast);
	}

	.back-link:hover {
		background: var(--color-bg-hover);
	}

	.spike-title {
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin: 0;
		white-space: nowrap;
	}

	.spike-badge {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--color-accent);
		background: var(--color-accent-muted);
		padding: 2px 6px;
		border-radius: var(--radius-full);
		white-space: nowrap;
	}

	.tab-bar {
		display: flex;
		gap: 0;
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
		padding: 0 var(--space-6);
		flex-shrink: 0;
	}

	.tab {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text-secondary);
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		padding: var(--space-4) var(--space-5);
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: var(--space-3);
		transition:
			color var(--transition-fast),
			border-color var(--transition-fast);
		white-space: nowrap;
	}

	.tab:hover {
		color: var(--color-text-primary);
	}

	.tab.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}

	.tab-shortcut {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--color-text-muted);
		background: var(--color-bg-elevated);
		padding: 1px 4px;
		border-radius: 3px;
	}

	.tab.active .tab-shortcut {
		color: var(--color-accent);
		background: var(--color-accent-muted);
	}

	.tab-content {
		flex: 1;
		min-height: 0;
		position: relative;
	}

	.tab-panel {
		position: absolute;
		inset: 0;
		overflow: auto;
		display: none;
	}

	.tab-panel.visible {
		display: block;
	}

	/* Shell state harness */
	.shell-harness {
		display: flex;
		flex-direction: column;
		height: 100%;
		gap: 0;
	}

	.state-selector {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background: var(--color-bg-elevated);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	.state-selector-label {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.state-btn {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--color-text-secondary);
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		padding: var(--space-1) var(--space-3);
		cursor: pointer;
		transition:
			color var(--transition-fast),
			background var(--transition-fast),
			border-color var(--transition-fast);
	}

	.state-btn:hover {
		color: var(--color-text-primary);
		background: var(--color-bg-hover);
	}

	.state-btn.active {
		color: var(--color-accent);
		background: var(--color-accent-muted);
		border-color: var(--color-accent);
	}

	.diff-area {
		flex: 1;
		min-height: 0;
		overflow: auto;
		padding: var(--space-4);
	}
</style>
