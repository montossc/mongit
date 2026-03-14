<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { EditorState } from '@codemirror/state';
	import { EditorView } from '@codemirror/view';
	import { MergeView } from '@codemirror/merge';
	import { baseExtensions, languageExtension } from '$lib/utils/codemirror-config';
	import { mongitTheme, mongitReadOnlyTheme } from '$lib/utils/codemirror-theme';

	const SAMPLE_BASE = `type AppConfig = {
  mode: 'dev' | 'prod';
  retryCount: number;
  timeoutMs: number;
  enableCache: boolean;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
};

export function processConfig(config: AppConfig): string {
  const retries = Math.max(0, config.retryCount);
  const timeout = Math.max(100, config.timeoutMs);
  const cacheLabel = config.enableCache ? 'cache:on' : 'cache:off';

  const summary = [
    \`mode=\${config.mode}\`,
    \`retries=\${retries}\`,
    \`timeout=\${timeout}\`,
    \`log=\${config.logLevel}\`,
    cacheLabel
  ].join(';');

  if (config.mode === 'prod' && retries > 3) {
    return summary + ';strict=true';
  }

  return summary;
}
`;

	const SAMPLE_OURS = `type AppConfig = {
  mode: 'dev' | 'prod';
  retryCount: number;
  timeoutMs: number;
  enableCache: boolean;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
  envName?: string;
};

export function processConfig(config: AppConfig, requestId: string): string {
  const retries = Math.max(0, config.retryCount);
  const timeout = Math.max(100, config.timeoutMs);
  const cacheLabel = config.enableCache ? 'cache:on' : 'cache:off';
  const env = config.envName ?? 'local';

  const summary = [
    \`mode=\${config.mode}\`,
    \`retries=\${retries}\`,
    \`timeout=\${timeout}\`,
    \`log=\${config.logLevel}\`,
    \`env=\${env}\`,
    \`request=\${requestId}\`,
    cacheLabel
  ].join(';');

  if (config.mode === 'prod' && retries > 3) {
    return summary + ';strict=true';
  }

  return summary;
}
`;

	const SAMPLE_THEIRS = `type AppConfig = {
  mode: 'dev' | 'prod';
  retryCount: number;
  timeoutMs: number;
  enableCache: boolean;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
};

export function processConfig(config: AppConfig): string {
  const retries = Math.max(0, config.retryCount);
  const timeout = Math.max(100, config.timeoutMs);
  const cacheLabel = config.enableCache ? 'cache:on' : 'cache:off';

  const summary = {
    mode: config.mode,
    retries,
    timeout,
    log: config.logLevel,
    cache: cacheLabel
  };

  if (config.mode === 'prod' && retries > 3) {
    return JSON.stringify({ ...summary, strict: true });
  }

  return JSON.stringify(summary);
}
`;

	let {
		base = SAMPLE_BASE,
		ours = SAMPLE_OURS,
		theirs = SAMPLE_THEIRS
	}: { base?: string; ours?: string; theirs?: string } = $props();

	let leftPaneEl = $state<HTMLDivElement | null>(null);
	let centerPaneEl = $state<HTMLDivElement | null>(null);
	let rightPaneEl = $state<HTMLDivElement | null>(null);

	let leftMergeView: MergeView | null = null;
	let centerEditorView: EditorView | null = null;
	let rightMergeView: MergeView | null = null;

	let currentChunk = $state(0);

	const sharedReadOnlyExtensions = [
		...baseExtensions(),
		...languageExtension('process-config.ts'),
		mongitTheme,
		mongitReadOnlyTheme,
		EditorState.readOnly.of(true),
		EditorView.editable.of(false)
	];

	function applyToCenter(content: string): void {
		if (!centerEditorView) return;
		centerEditorView.dispatch({
			changes: {
				from: 0,
				to: centerEditorView.state.doc.length,
				insert: content
			}
		});
	}

	function acceptOurs(): void {
		currentChunk = 0;
		applyToCenter(ours);
	}

	function acceptTheirs(): void {
		currentChunk = 0;
		applyToCenter(theirs);
	}

	onMount(() => {
		if (!leftPaneEl || !centerPaneEl || !rightPaneEl) return;

		leftMergeView = new MergeView({
			parent: leftPaneEl,
			a: { doc: base, extensions: sharedReadOnlyExtensions },
			b: { doc: ours, extensions: sharedReadOnlyExtensions },
			gutter: true,
			highlightChanges: true,
			collapseUnchanged: { margin: 3, minSize: 8 }
		});

		centerEditorView = new EditorView({
			parent: centerPaneEl,
			state: EditorState.create({
				doc: base,
				extensions: [...baseExtensions(), ...languageExtension('process-config.ts'), mongitTheme]
			})
		});

		rightMergeView = new MergeView({
			parent: rightPaneEl,
			a: { doc: base, extensions: sharedReadOnlyExtensions },
			b: { doc: theirs, extensions: sharedReadOnlyExtensions },
			gutter: true,
			highlightChanges: true,
			collapseUnchanged: { margin: 3, minSize: 8 }
		});
	});

	onDestroy(() => {
		leftMergeView?.destroy();
		centerEditorView?.destroy();
		rightMergeView?.destroy();
	});
</script>

<div class="merge-toolbar">
	<button class="merge-btn ours" type="button" onclick={acceptOurs}>Accept Ours</button>
	<button class="merge-btn theirs" type="button" onclick={acceptTheirs}>Accept Theirs</button>
	<span class="merge-toolbar-label">Chunk: {currentChunk + 1}</span>
</div>

<div class="merge-container">
	<div class="merge-pane">
		<div class="merge-pane-header ours">Ours vs Base</div>
		<div bind:this={leftPaneEl} class="merge-editor-host"></div>
	</div>

	<div class="merge-pane">
		<div class="merge-pane-header result">Result (Editable)</div>
		<div bind:this={centerPaneEl} class="merge-editor-host"></div>
	</div>

	<div class="merge-pane">
		<div class="merge-pane-header theirs">Theirs vs Base</div>
		<div bind:this={rightPaneEl} class="merge-editor-host"></div>
	</div>
</div>

<style>
	.merge-container {
		display: grid;
		grid-template-columns: 1fr 1fr 1fr;
		gap: 2px;
		height: 100%;
		overflow: hidden;
	}

	.merge-pane {
		overflow: auto;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		position: relative;
		min-height: 0;
	}

	.merge-editor-host {
		height: calc(100% - 32px);
		min-height: 0;
	}

	.merge-pane-header {
		position: sticky;
		top: 0;
		z-index: 1;
		padding: 6px 12px;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		border-bottom: 1px solid var(--color-border);
	}

	.merge-pane-header.ours {
		background-color: var(--color-success-muted);
		color: var(--color-success);
	}

	.merge-pane-header.result {
		background-color: var(--color-bg-elevated);
		color: var(--color-text-primary);
	}

	.merge-pane-header.theirs {
		background-color: color-mix(in srgb, var(--color-info) 10%, transparent);
		color: var(--color-info);
	}

	.merge-toolbar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: var(--color-bg-surface);
		border-bottom: 1px solid var(--color-border);
	}

	.merge-toolbar-label {
		margin-left: 4px;
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--color-text-secondary);
	}

	.merge-btn {
		padding: 4px 12px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--color-border);
		background: var(--color-bg-elevated);
		color: var(--color-text-primary);
		font-family: var(--font-sans);
		font-size: 12px;
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.merge-btn:hover {
		background: var(--color-bg-hover);
	}

	.merge-btn.ours {
		border-color: var(--color-success);
	}

	.merge-btn.theirs {
		border-color: var(--color-info);
	}

	:global(.cm-mergeView) {
		height: 100%;
		overflow: auto;
	}

	:global(.cm-editor) {
		height: 100%;
	}
</style>
