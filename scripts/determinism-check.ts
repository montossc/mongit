/**
 * Deterministic regression check for commit graph lane assignment.
 *
 * Verifies assignLanes() returns identical output across repeated runs
 * for an identical fixed-topology input fixture.
 *
 * Run: npx tsx scripts/determinism-check.ts
 */

import { hitTest } from "../src/lib/graph/hitTest";
import { assignLanes } from "../src/lib/graph/layout";
import {
	GRAPH_PADDING_LEFT,
	getVisibleRange,
	LANE_WIDTH,
	ROW_HEIGHT,
	segmentIntersectsVisibleRows,
	TEXT_PADDING_LEFT,
} from "../src/lib/graph/render";
import type {
	CommitData,
	GraphSegment,
	LayoutResult,
	RefData,
} from "../src/lib/graph/types";

function createDeterministicFixture(): {
	commits: CommitData[];
	refs: RefData[];
} {
	const commits: CommitData[] = [
		// HEAD commit (newest), fan-in merge from hotfix + mainline
		{
			id: "c11",
			message: "Merge hotfix and mainline",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 11,
			parent_ids: ["c10", "c9"],
		},
		// Continuation of mainline
		{
			id: "c10",
			message: "Mainline after merge",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 10,
			parent_ids: ["c8"],
		},
		// Hotfix lane
		{
			id: "c9",
			message: "Hotfix commit",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 9,
			parent_ids: ["c6"],
		},
		// Octopus merge (3 parents)
		{
			id: "c8",
			message: "Octopus merge alpha+beta+gamma",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 8,
			parent_ids: ["c7", "c5", "c4"],
		},
		// Linear chain continuation
		{
			id: "c7",
			message: "Mainline continuation",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 7,
			parent_ids: ["c6"],
		},
		// 3-way fan-out source
		{
			id: "c6",
			message: "Fan-out source",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 6,
			parent_ids: ["c3"],
		},
		// Branch alpha
		{
			id: "c5",
			message: "Alpha branch work",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 5,
			parent_ids: ["c6"],
		},
		// Branch beta
		{
			id: "c4",
			message: "Beta branch work",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 4,
			parent_ids: ["c6"],
		},
		// Linear chain (older)
		{
			id: "c3",
			message: "Linear 3",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 3,
			parent_ids: ["c2"],
		},
		{
			id: "c2",
			message: "Linear 2",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 2,
			parent_ids: ["c1"],
		},
		{
			id: "c1",
			message: "Linear root",
			author_name: "Fixture",
			author_email: "fixture@example.com",
			time: 1,
			parent_ids: [],
		},
	];

	const refs: RefData[] = [
		{ name: "HEAD", ref_type: "Head", commit_id: "c11" },
		{ name: "main", ref_type: "LocalBranch", commit_id: "c11" },
		{ name: "hotfix/urgent", ref_type: "LocalBranch", commit_id: "c9" },
		{ name: "feature/alpha", ref_type: "LocalBranch", commit_id: "c5" },
		{ name: "feature/beta", ref_type: "LocalBranch", commit_id: "c4" },
		{ name: "v0.1.0", ref_type: "Tag", commit_id: "c8" },
		{ name: "origin/main", ref_type: "RemoteBranch", commit_id: "c10" },
	];

	return { commits, refs };
}

function normalizeSegments(segments: GraphSegment[]): string[] {
	return segments
		.map(
			(s) =>
				`${s.fromId}|${s.toId}|${s.fromLane}|${s.toLane}|${s.fromRow}|${s.toRow}|${s.color}|${s.isMerge ? 1 : 0}`,
		)
		.sort();
}

function createComparableSnapshot(result: LayoutResult) {
	const nodes = result.nodes.map((n) => ({
		id: n.data.id,
		lane: n.lane,
		row: n.row,
		color: n.color,
		refs: n.refs.map((r) => `${r.ref_type}:${r.name}`),
	}));

	return {
		laneCount: result.laneCount,
		segmentCount: result.segments.length,
		nodes,
		segments: normalizeSegments(result.segments),
	};
}

function formatDiff(
	first: ReturnType<typeof createComparableSnapshot>,
	next: ReturnType<typeof createComparableSnapshot>,
): string {
	if (first.laneCount !== next.laneCount) {
		return `laneCount mismatch: ${first.laneCount} vs ${next.laneCount}`;
	}
	if (first.segmentCount !== next.segmentCount) {
		return `segmentCount mismatch: ${first.segmentCount} vs ${next.segmentCount}`;
	}

	for (let i = 0; i < first.nodes.length; i++) {
		const a = first.nodes[i];
		const b = next.nodes[i];
		if (
			a.id !== b.id ||
			a.lane !== b.lane ||
			a.row !== b.row ||
			a.color !== b.color
		) {
			return `node mismatch at index ${i}: ${JSON.stringify(a)} vs ${JSON.stringify(b)}`;
		}
		if (a.refs.join(",") !== b.refs.join(",")) {
			return `node refs mismatch for ${a.id}: [${a.refs.join(",")}] vs [${b.refs.join(",")}]`;
		}
	}

	for (let i = 0; i < first.segments.length; i++) {
		if (first.segments[i] !== next.segments[i]) {
			return `segment mismatch at sorted index ${i}: ${first.segments[i]} vs ${next.segments[i]}`;
		}
	}

	return "unknown mismatch";
}

function runDeterminismCheck(): void {
	const { commits, refs } = createDeterministicFixture();
	const fixtureLayout = assignLanes(commits, refs);
	const runs = [
		createComparableSnapshot(fixtureLayout),
		createComparableSnapshot(assignLanes(commits, refs)),
		createComparableSnapshot(assignLanes(commits, refs)),
	];

	for (let i = 1; i < runs.length; i++) {
		const baseline = runs[0];
		const candidate = runs[i];
		if (JSON.stringify(baseline) !== JSON.stringify(candidate)) {
			const diff = formatDiff(baseline, candidate);
			throw new Error(
				`Determinism regression: run #1 and run #${i + 1} differ. ${diff}`,
			);
		}
	}

	console.log(
		"PASS: assignLanes() is deterministic for the fixed topology fixture.",
	);
	console.log(`      commits=${commits.length}, refs=${refs.length}, runs=3`);
	console.log(
		`      laneCount=${runs[0].laneCount}, segments=${runs[0].segmentCount}`,
	);

	runVisibleRangeRegression();
	runEdgeCullingRegression();
	runHitTestRegression(fixtureLayout);
	console.log(
		"PASS: renderer viewport regressions hold for visible-range, edge-culling, and hit-test math.",
	);
}

function assert(condition: boolean, message: string): void {
	if (!condition) {
		throw new Error(`Regression failed: ${message}`);
	}
}

function runVisibleRangeRegression(): void {
	const totalRows = 200;
	const canvasHeight = 320;
	const top = getVisibleRange(totalRows, 0, canvasHeight);
	assert(top.first === 0, `top.first expected 0, got ${top.first}`);
	assert(top.last === 15, `top.last expected 15, got ${top.last}`);

	const middleScrollTop = ROW_HEIGHT * 5;
	const middle = getVisibleRange(totalRows, middleScrollTop, canvasHeight);
	assert(middle.first === 0, `middle.first expected 0, got ${middle.first}`);
	assert(middle.last === 20, `middle.last expected 20, got ${middle.last}`);

	const bottomScrollTop = Math.max(0, totalRows * ROW_HEIGHT - canvasHeight);
	const bottom = getVisibleRange(totalRows, bottomScrollTop, canvasHeight);
	const expectedBottomFirst = Math.max(
		0,
		Math.floor(bottomScrollTop / ROW_HEIGHT) - 5,
	);
	assert(
		bottom.first === expectedBottomFirst,
		`bottom.first expected ${expectedBottomFirst}, got ${bottom.first}`,
	);

	const empty = getVisibleRange(0, 0, canvasHeight);
	assert(
		empty.first === 0 && empty.last === -1,
		`empty range expected {0,-1}, got {${empty.first},${empty.last}}`,
	);
}

function runEdgeCullingRegression(): void {
	const visible = { first: 20, last: 30 };

	const crossingSegment: GraphSegment = {
		fromId: "from",
		toId: "to",
		fromLane: 0,
		toLane: 2,
		fromRow: 0,
		toRow: 50,
		color: 0,
		isMerge: true,
	};

	const aboveSegment: GraphSegment = {
		...crossingSegment,
		fromRow: 0,
		toRow: 10,
	};

	const belowSegment: GraphSegment = {
		...crossingSegment,
		fromRow: 35,
		toRow: 40,
	};

	assert(
		segmentIntersectsVisibleRows(crossingSegment, visible),
		"crossing long-span segment should be considered visible",
	);
	assert(
		!segmentIntersectsVisibleRows(aboveSegment, visible),
		"segment entirely above viewport should be culled",
	);
	assert(
		!segmentIntersectsVisibleRows(belowSegment, visible),
		"segment entirely below viewport should be culled",
	);
}

function runHitTestRegression(layout: LayoutResult): void {
	const firstNode = layout.nodes[0];
	assert(Boolean(firstNode), "fixture must contain at least one node");
	if (!firstNode) return;

	const rowCenterY = firstNode.row * ROW_HEIGHT + ROW_HEIGHT / 2;
	const nodeCenterX =
		GRAPH_PADDING_LEFT + firstNode.lane * LANE_WIDTH + LANE_WIDTH / 2;

	const nodeHit = hitTest(layout, nodeCenterX, rowCenterY, layout.laneCount);
	assert(
		nodeHit.type === "node" && nodeHit.node.data.id === firstNode.data.id,
		"node center should resolve to node hit",
	);

	const rowHit = hitTest(
		layout,
		nodeCenterX + 24,
		rowCenterY,
		layout.laneCount,
	);
	assert(
		rowHit.type === "row" && rowHit.node.data.id === firstNode.data.id,
		"near-node row area should resolve to row hit",
	);

	const graphEndX = GRAPH_PADDING_LEFT + layout.laneCount * LANE_WIDTH;
	const textStartX = graphEndX + TEXT_PADDING_LEFT;
	const nodeWithoutRefs =
		layout.nodes.find((node) => node.refs.length === 0) ?? firstNode;
	const textRowY = nodeWithoutRefs.row * ROW_HEIGHT + ROW_HEIGHT / 2;
	const textHit = hitTest(layout, textStartX + 10, textRowY, layout.laneCount);
	assert(
		textHit.type === "row" && textHit.node.data.id === nodeWithoutRefs.data.id,
		"text area should still resolve to row hit",
	);

	const offscreenTop = hitTest(layout, nodeCenterX, -1, layout.laneCount);
	assert(offscreenTop.type === "none", "negative Y should return none");

	const offscreenBottom = hitTest(
		layout,
		nodeCenterX,
		layout.nodes.length * ROW_HEIGHT + 1,
		layout.laneCount,
	);
	assert(
		offscreenBottom.type === "none",
		"Y below last row should return none",
	);

	const nodeWithRefs = layout.nodes.find((node) => node.refs.length > 0);
	assert(
		Boolean(nodeWithRefs),
		"fixture should contain at least one ref-bearing node",
	);
	if (!nodeWithRefs) return;

	const refRowY = nodeWithRefs.row * ROW_HEIGHT + ROW_HEIGHT / 2;
	const refProbeX = graphEndX + 8;
	const refHit = hitTest(layout, refProbeX, refRowY, layout.laneCount);
	assert(
		refHit.type === "ref" && refHit.node.data.id === nodeWithRefs.data.id,
		"ref badge zone should resolve to ref hit",
	);
}

runDeterminismCheck();
