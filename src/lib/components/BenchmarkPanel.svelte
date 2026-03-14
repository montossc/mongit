<script lang="ts">
	import { EditorState } from '@codemirror/state';
	import { MergeView } from '@codemirror/merge';
	import { onDestroy } from 'svelte';
	import { baseExtensions, languageExtension } from '$lib/utils/codemirror-config';
	import { mongitReadOnlyTheme } from '$lib/utils/codemirror-theme';

	type BenchmarkStatus = 'PASS' | 'WARN' | 'FAIL';

	type BenchmarkResult = {
		test: string;
		lines: number;
		timeMs: number;
		status: BenchmarkStatus;
	};

	type BenchmarkCase = {
		test: string;
		lines: number;
		thresholdMs: number | null;
	};

	const BENCHMARKS: BenchmarkCase[] = [
		{ test: 'Diff 1k', lines: 1_000, thresholdMs: null },
		{ test: 'Diff 10k', lines: 10_000, thresholdMs: 200 },
		{ test: 'Diff 50k', lines: 50_000, thresholdMs: 1_000 }
	];

	let benchmarkContainer = $state<HTMLDivElement | null>(null);
	let isRunning = $state(false);
	let progress = $state(0);
	let progressLabel = $state('Idle');
	let results = $state<BenchmarkResult[]>([]);
	let runError = $state<string | null>(null);
	let activeMergeView: MergeView | null = null;

	function createSeededRandom(seed: number): () => number {
		let state = seed >>> 0;
		return () => {
			state = (1664525 * state + 1013904223) >>> 0;
			return state / 0x100000000;
		};
	}

	function generateCode(lines: number): string {
		const out: string[] = [];
		for (let i = 0; i < lines; i += 1) {
			if (i % 20 === 0) {
				const fnId = Math.floor(i / 20);
				out.push(`export function computeMetric${fnId}(input: number): string {`);
				continue;
			}

			if (i % 20 === 1) {
				out.push(`\tconst value = input + ${i};`);
				continue;
			}

			if (i % 20 === 2) {
				out.push(`\tconst label = \`metric-${i}-\${value}\`;`);
				continue;
			}

			if (i % 20 === 18) {
				out.push('\treturn label;');
				continue;
			}

			if (i % 20 === 19) {
				out.push('}');
				continue;
			}

			out.push(`\t// line ${i}: normalize and cache`);
		}

		return out.join('\n');
	}

	function generateModified(original: string, changePercent = 10): string {
		const lines = original.split('\n');
		const edits = Math.max(1, Math.floor((lines.length * changePercent) / 100));
		const rand = createSeededRandom(lines.length + edits);

		for (let i = 0; i < edits; i += 1) {
			if (lines.length === 0) break;
			const index = Math.floor(rand() * lines.length);
			const op = rand();

			if (op < 0.6) {
				lines[index] = `${lines[index]} // updated ${i}`;
			} else if (op < 0.8) {
				lines.splice(index, 0, `\tconst inserted${i} = ${index};`);
			} else {
				lines.splice(index, 1);
			}
		}

		return lines.join('\n');
	}

	function getStatus(timeMs: number, thresholdMs: number | null): BenchmarkStatus {
		if (thresholdMs === null) return 'PASS';
		if (timeMs <= thresholdMs) return 'PASS';
		if (timeMs <= thresholdMs * 1.25) return 'WARN';
		return 'FAIL';
	}

	async function nextFrame(): Promise<void> {
		await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
	}

	async function runDiffBenchmark(lineCount: number): Promise<number> {
		if (!benchmarkContainer) {
			throw new Error('Benchmark container is not mounted');
		}

		const original = generateCode(lineCount);
		const modified = generateModified(original);
		const container = benchmarkContainer;
		container.innerHTML = '';

		const extensions = [
			EditorState.readOnly.of(true),
			...baseExtensions(),
			...languageExtension('benchmark.ts'),
			mongitReadOnlyTheme
		];

		const start = performance.now();
		activeMergeView = new MergeView({
			parent: container,
			a: { doc: original, extensions },
			b: { doc: modified, extensions },
			gutter: true,
			highlightChanges: true,
			collapseUnchanged: { margin: 3, minSize: 8 }
		});

		await nextFrame();

		const elapsed = performance.now() - start;
		activeMergeView.destroy();
		activeMergeView = null;
		container.innerHTML = '';

		return elapsed;
	}

	async function runBenchmarks(): Promise<void> {
		if (isRunning) return;

		isRunning = true;
		progress = 0;
		progressLabel = 'Preparing benchmark data...';
		runError = null;
		results = [];

		try {
			for (let i = 0; i < BENCHMARKS.length; i += 1) {
				const testCase = BENCHMARKS[i];
				progressLabel = `Running ${testCase.test} (${testCase.lines.toLocaleString()} lines)`;

				const timeMs = await runDiffBenchmark(testCase.lines);
				results = [
					...results,
					{
						test: testCase.test,
						lines: testCase.lines,
						timeMs,
						status: getStatus(timeMs, testCase.thresholdMs)
					}
				];
				progress = (i + 1) / BENCHMARKS.length;

				await new Promise((resolve) => setTimeout(resolve, 0));
			}

			progressLabel = 'Benchmark run complete';
		} catch (error) {
			runError = error instanceof Error ? error.message : 'Benchmark failed';
			progressLabel = 'Benchmark failed';
		} finally {
			activeMergeView?.destroy();
			activeMergeView = null;
			benchmarkContainer?.replaceChildren();
			isRunning = false;
		}
	}

	onDestroy(() => {
		activeMergeView?.destroy();
		activeMergeView = null;
		benchmarkContainer?.replaceChildren();
	});
</script>

<section class="benchmark-panel">
	<header class="header">
		<div>
			<h2>Diff Rendering Benchmarks</h2>
			<p>Measures time-to-render for CM6 MergeView at 1k, 10k, and 50k line diffs.</p>
		</div>
		<button class="run-btn" type="button" onclick={runBenchmarks} disabled={isRunning}>
			{isRunning ? 'Running…' : 'Run Benchmarks'}
		</button>
	</header>

	<div class="progress-wrap" aria-live="polite">
		<div class="progress-meta">
			<span>{progressLabel}</span>
			<span class="mono">{Math.round(progress * 100)}%</span>
		</div>
		<div class="progress-track" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(progress * 100)}>
			<div class="progress-fill" style={`width: ${Math.round(progress * 100)}%`}></div>
		</div>
	</div>

	{#if runError}
		<p class="error">{runError}</p>
	{/if}

	<table class="results">
		<thead>
			<tr>
				<th>Test</th>
				<th>Lines</th>
				<th>Time (ms)</th>
				<th>Status</th>
			</tr>
		</thead>
		<tbody>
			{#if results.length === 0}
				<tr>
					<td colspan="4" class="empty">No results yet. Click “Run Benchmarks”.</td>
				</tr>
			{:else}
				{#each results as result}
					<tr>
						<td>{result.test}</td>
						<td class="mono">{result.lines.toLocaleString()}</td>
						<td class="mono">{result.timeMs.toFixed(1)}</td>
						<td>
							<span class={`status ${result.status.toLowerCase()}`}>{result.status}</span>
						</td>
					</tr>
				{/each}
			{/if}
		</tbody>
	</table>

	<div class="benchmark-host" bind:this={benchmarkContainer} aria-hidden="true"></div>
</section>

<style>
	.benchmark-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		padding: var(--space-6);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		background: var(--color-bg-surface);
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: var(--space-6);
	}

	h2 {
		margin: 0;
		font-size: 16px;
		color: var(--color-text-primary);
	}

	p {
		margin: var(--space-4) 0 0;
		font-size: 13px;
		color: var(--color-text-secondary);
	}

	.run-btn {
		align-self: center;
		padding: var(--space-4) var(--space-6);
		border-radius: var(--radius-sm);
		border: 1px solid var(--color-accent);
		background: var(--color-bg-elevated);
		color: var(--color-text-primary);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.run-btn:hover:enabled {
		background: var(--color-bg-hover);
	}

	.run-btn:disabled {
		opacity: 0.65;
		cursor: not-allowed;
	}

	.progress-wrap {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.progress-meta {
		display: flex;
		justify-content: space-between;
		font-size: 12px;
		color: var(--color-text-secondary);
	}

	.progress-track {
		height: 8px;
		border-radius: 999px;
		border: 1px solid var(--color-border);
		background: var(--color-bg);
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background: var(--color-accent);
		transition: width var(--transition-fast);
	}

	.error {
		margin: 0;
		color: var(--color-danger);
	}

	.results {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	.results th,
	.results td {
		padding: var(--space-4) var(--space-5);
		border-bottom: 1px solid var(--color-border);
		text-align: left;
	}

	.results th {
		color: var(--color-text-secondary);
		font-weight: 600;
	}

	.results td {
		color: var(--color-text-primary);
	}

	.empty {
		color: var(--color-text-muted) !important;
	}

	.mono {
		font-family: var(--font-mono);
	}

	.status {
		display: inline-flex;
		align-items: center;
		padding: 2px 8px;
		border-radius: 999px;
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.03em;
		font-family: var(--font-mono);
	}

	.status.pass {
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 14%, transparent);
	}

	.status.warn {
		color: var(--color-warning);
		background: color-mix(in srgb, var(--color-warning) 16%, transparent);
	}

	.status.fail {
		color: var(--color-danger);
		background: color-mix(in srgb, var(--color-danger) 16%, transparent);
	}

	.benchmark-host {
		position: absolute;
		left: -99999px;
		top: 0;
		width: 1200px;
		height: 800px;
		overflow: hidden;
		opacity: 0;
		pointer-events: none;
	}
</style>
