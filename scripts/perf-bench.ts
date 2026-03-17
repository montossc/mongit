/**
 * Performance benchmark for the commit graph layout engine.
 * Tests layout time, memory consumption, and scaling behavior.
 *
 * Run: npx tsx scripts/perf-bench.ts
 */

import { assignLanes, generateSyntheticCommits } from '../src/lib/graph/layout';
import type { CommitNode, RefData } from '../src/lib/graph/types';

// ── Hit test simulation ──

function simulateHitTest(
	nodes: CommitNode[],
	laneCount: number,
	iterations: number,
): number {
	const ROW_HEIGHT = 32;
	const LANE_WIDTH = 16;
	const NODE_RADIUS = 4;
	const GRAPH_PADDING_LEFT = 8;

	const start = performance.now();
	for (let i = 0; i < iterations; i++) {
		const randomRow = Math.floor(Math.random() * nodes.length);
		const randomX =
			Math.random() * (GRAPH_PADDING_LEFT + laneCount * LANE_WIDTH + 400);
		const absoluteY = randomRow * ROW_HEIGHT + ROW_HEIGHT / 2;

		// Row lookup (O(1))
		const row = Math.floor(absoluteY / ROW_HEIGHT);
		if (row >= 0 && row < nodes.length) {
			const node = nodes[row];
			const nodeX =
				GRAPH_PADDING_LEFT + node.lane * LANE_WIDTH + LANE_WIDTH / 2;
			const nodeY = row * ROW_HEIGHT + ROW_HEIGHT / 2;
			const dx = randomX - nodeX;
			const dy = absoluteY - nodeY;
			const _isHit = dx * dx + dy * dy <= NODE_RADIUS * NODE_RADIUS;
		}
	}
	return (performance.now() - start) / iterations;
}

// ── Benchmark runner ──

interface BenchResult {
	commits: number;
	branches: number;
	genTimeMs: number;
	layoutTimeMs: number;
	laneCount: number;
	segmentCount: number;
	hitTestAvgMs: number;
	heapUsedMB: number;
	heapTotalMB: number;
}

function runBenchmark(commitCount: number, branchCount: number): BenchResult {
	// Force GC if available
	if (global.gc) global.gc();
	const heapBefore = process.memoryUsage();

	// Generate synthetic data
	const genStart = performance.now();
	const commits = generateSyntheticCommits(commitCount, branchCount);
	const genTimeMs = performance.now() - genStart;

	// Create refs
	const refs: RefData[] = [];
	if (commits.length > 0) {
		refs.push({ name: 'main', ref_type: 'Head', commit_id: commits[0].id });
		refs.push({
			name: 'main',
			ref_type: 'LocalBranch',
			commit_id: commits[0].id
		});
	}
	if (commits.length > 10) {
		refs.push({
			name: 'feature/graph',
			ref_type: 'LocalBranch',
			commit_id: commits[10].id
		});
	}
	if (commits.length > 50) {
		refs.push({ name: 'v0.1.0', ref_type: 'Tag', commit_id: commits[50].id });
	}
	if (commits.length > 100) {
		refs.push({
			name: 'origin/main',
			ref_type: 'RemoteBranch',
			commit_id: commits[100].id
		});
	}

	// Run layout
	const result = assignLanes(commits, refs, { maxCommits: commitCount });

	// Hit test benchmark
	const hitTestAvgMs = simulateHitTest(result.nodes, result.laneCount, 10000);

	// Memory measurement
	const heapAfter = process.memoryUsage();
	const heapUsedMB = (heapAfter.heapUsed - heapBefore.heapUsed) / 1024 / 1024;
	const heapTotalMB = heapAfter.heapUsed / 1024 / 1024;

	return {
		commits: commitCount,
		branches: branchCount,
		genTimeMs,
		layoutTimeMs: result.layoutTimeMs,
		laneCount: result.laneCount,
		segmentCount: result.segments.length,
		hitTestAvgMs,
		heapUsedMB,
		heapTotalMB
	};
}

// ── Main ──

console.log('='.repeat(72));
console.log('  mongit — Commit Graph Performance Benchmark');
console.log('='.repeat(72));
console.log();

const targets = [
	{ commits: 1_000, branches: 5 },
	{ commits: 5_000, branches: 8 },
	{ commits: 10_000, branches: 10 },
	{ commits: 25_000, branches: 15 },
	{ commits: 50_000, branches: 20 },
	{ commits: 100_000, branches: 25 }
];

// PRD targets
const LAYOUT_TARGET_MS = 100; // < 100ms for 10k commits
const HIT_TEST_TARGET_MS = 1; // < 1ms per hit test
const MEMORY_TARGET_MB = 50; // < 50MB JS heap for 10k

console.log('PRD Targets:');
console.log(`  Layout (10k):  < ${LAYOUT_TARGET_MS}ms`);
console.log(`  Hit test:      < ${HIT_TEST_TARGET_MS}ms`);
console.log(`  Memory (10k):  < ${MEMORY_TARGET_MB}MB JS heap`);
console.log();

// Warmup
console.log('Warming up...');
runBenchmark(1000, 3);
runBenchmark(1000, 3);
console.log();

// Run benchmarks
const results: BenchResult[] = [];

for (const target of targets) {
	process.stdout.write(
		`Benchmarking ${target.commits.toLocaleString()} commits...`
	);

	// Run 3 times, take median layout time
	const runs: BenchResult[] = [];
	for (let i = 0; i < 3; i++) {
		runs.push(runBenchmark(target.commits, target.branches));
	}
	runs.sort((a, b) => a.layoutTimeMs - b.layoutTimeMs);
	const median = runs[1]; // median of 3
	results.push(median);

	console.log(` done (${median.layoutTimeMs.toFixed(1)}ms)`);
}

console.log();

// Results table
console.log('─'.repeat(100));
console.log(
	'Commits'.padStart(10),
	'Branches'.padStart(10),
	'Gen (ms)'.padStart(10),
	'Layout (ms)'.padStart(12),
	'Lanes'.padStart(8),
	'Segments'.padStart(10),
	'HitTest (ms)'.padStart(14),
	'Heap (MB)'.padStart(11)
);
console.log('─'.repeat(100));

for (const r of results) {
	const layoutPass =
		r.commits <= 10000 ? (r.layoutTimeMs < LAYOUT_TARGET_MS ? '✓' : '✗') : ' ';
	const hitPass = r.hitTestAvgMs < HIT_TEST_TARGET_MS ? '✓' : '✗';
	const memPass =
		r.commits <= 10000 ? (r.heapTotalMB < MEMORY_TARGET_MB ? '✓' : '✗') : ' ';

	console.log(
		r.commits.toLocaleString().padStart(10),
		r.branches.toString().padStart(10),
		r.genTimeMs.toFixed(1).padStart(10),
		`${r.layoutTimeMs.toFixed(1)} ${layoutPass}`.padStart(12),
		r.laneCount.toString().padStart(8),
		r.segmentCount.toLocaleString().padStart(10),
		`${r.hitTestAvgMs.toFixed(4)} ${hitPass}`.padStart(14),
		`${r.heapTotalMB.toFixed(1)} ${memPass}`.padStart(11)
	);
}

console.log('─'.repeat(100));
console.log();

// Pass/fail summary for 10k target
const tenK = results.find((r) => r.commits === 10000);
if (tenK) {
	console.log('═'.repeat(50));
	console.log('  10k Commit Validation (PRD R7 Targets)');
	console.log('═'.repeat(50));

	const checks = [
		{
			name: 'Layout time < 100ms',
			value: `${tenK.layoutTimeMs.toFixed(1)}ms`,
			pass: tenK.layoutTimeMs < LAYOUT_TARGET_MS
		},
		{
			name: 'Hit test < 1ms',
			value: `${tenK.hitTestAvgMs.toFixed(4)}ms`,
			pass: tenK.hitTestAvgMs < HIT_TEST_TARGET_MS
		},
		{
			name: 'JS heap < 50MB',
			value: `${tenK.heapTotalMB.toFixed(1)}MB`,
			pass: tenK.heapTotalMB < MEMORY_TARGET_MB
		},
		{
			name: 'Lane count < 50',
			value: `${tenK.laneCount}`,
			pass: tenK.laneCount < 50
		}
	];

	for (const check of checks) {
		const icon = check.pass ? '✓ PASS' : '✗ FAIL';
		console.log(`  ${icon}  ${check.name.padEnd(25)} ${check.value}`);
	}

	const allPass = checks.every((c) => c.pass);
	console.log();
	console.log(
		`  Overall: ${allPass ? '✓ ALL TARGETS MET' : '✗ SOME TARGETS MISSED'}`
	);
	console.log();

	// Remaining targets that need manual verification
	console.log('  Manual verification needed (in-app):');
	console.log(
		'  [ ] Sustained FPS >= 55 during continuous scroll (toggle FPS: Cmd+Shift+P)'
	);
	console.log('  [ ] Render frame < 8ms (check FPS overlay frame time)');
	console.log('  [ ] First paint < 500ms (time from click to first frame)');
	console.log('  [ ] No visual glitches during rapid scroll');
	console.log('  [ ] Retina rendering is crisp (sharp lines/text at 2x)');
	console.log('  [ ] Correct topology (branches/merges render properly)');
}

console.log();
console.log('Done.');
