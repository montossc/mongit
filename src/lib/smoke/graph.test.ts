import { describe, expect, it } from "vitest";
import { assignLanes, generateSyntheticCommits } from "../graph/layout";
import {
	sortRefsByPriority,
	computeVisibleRefs,
	estimateRefBadgeWidth,
	REF_OVERFLOW_MAX_WIDTH,
} from "../graph/render";
import { hitTest } from "../graph/hitTest";
import type { RefData } from "../graph/types";

describe("Commit Graph Smoke Test", () => {
	it("should generate and layout 10,000 commits efficiently", () => {
		const count = 10_000;
		const start = performance.now();

		const commits = generateSyntheticCommits(count);
		const genTime = performance.now() - start;

		expect(commits).toHaveLength(count);

		const layoutStart = performance.now();
		const layout = assignLanes(commits, []);
		const layoutTime = performance.now() - layoutStart;

		console.log(
			`[Smoke Test] 10k Commits Gen: ${genTime.toFixed(1)}ms, Layout: ${layoutTime.toFixed(1)}ms`,
		);

		// Thresholds for regression (generous for CI)
		expect(genTime).toBeLessThan(150);
		expect(layoutTime).toBeLessThan(200);
		expect(layout.nodes.length).toBe(count);
		expect(layout.laneCount).toBeGreaterThan(0);
	});
});

// ── Ref priority sorting ──

describe("sortRefsByPriority", () => {
	it("returns empty array unchanged", () => {
		expect(sortRefsByPriority([])).toEqual([]);
	});

	it("returns single ref unchanged", () => {
		const refs: RefData[] = [
			{ name: "main", ref_type: "LocalBranch", commit_id: "a" },
		];
		expect(sortRefsByPriority(refs)).toEqual(refs);
	});

	it("sorts Head > LocalBranch > Tag > RemoteBranch", () => {
		const refs: RefData[] = [
			{ name: "origin/main", ref_type: "RemoteBranch", commit_id: "a" },
			{ name: "v1.0", ref_type: "Tag", commit_id: "a" },
			{ name: "HEAD", ref_type: "Head", commit_id: "a" },
			{ name: "main", ref_type: "LocalBranch", commit_id: "a" },
		];
		const sorted = sortRefsByPriority(refs);
		expect(sorted.map((r) => r.ref_type)).toEqual([
			"Head",
			"LocalBranch",
			"Tag",
			"RemoteBranch",
		]);
	});

	it("preserves order within same priority (stable sort)", () => {
		const refs: RefData[] = [
			{ name: "feat-a", ref_type: "LocalBranch", commit_id: "a" },
			{ name: "feat-b", ref_type: "LocalBranch", commit_id: "a" },
			{ name: "feat-c", ref_type: "LocalBranch", commit_id: "a" },
		];
		const sorted = sortRefsByPriority(refs);
		expect(sorted.map((r) => r.name)).toEqual(["feat-a", "feat-b", "feat-c"]);
	});
});

// ── Overflow computation ──

describe("computeVisibleRefs", () => {
	it("returns all refs when few fit easily", () => {
		const refs: RefData[] = [
			{ name: "main", ref_type: "LocalBranch", commit_id: "a" },
			{ name: "v1.0", ref_type: "Tag", commit_id: "a" },
		];
		const result = computeVisibleRefs(refs);
		expect(result.visible).toHaveLength(2);
		expect(result.overflowCount).toBe(0);
	});

	it("always shows at least one ref even if wide", () => {
		const refs: RefData[] = [
			{ name: "a-very-long-branch-name-that-exceeds-limits", ref_type: "LocalBranch", commit_id: "a" },
			{ name: "another-long-one", ref_type: "RemoteBranch", commit_id: "a" },
		];
		const result = computeVisibleRefs(refs);
		expect(result.visible).toHaveLength(1);
		expect(result.overflowCount).toBe(1);
	});

	it("overflows when many refs exceed width budget", () => {
		// Create 15 refs — more than can fit in REF_OVERFLOW_MAX_WIDTH
		const refs: RefData[] = Array.from({ length: 15 }, (_, i) => ({
			name: `branch-${i.toString().padStart(2, "0")}`,
			ref_type: "LocalBranch" as const,
			commit_id: "a",
		}));
		const result = computeVisibleRefs(refs);
		expect(result.visible.length).toBeLessThan(15);
		expect(result.overflowCount).toBeGreaterThan(0);
		expect(result.visible.length + result.overflowCount).toBe(15);
	});

	it("returns empty for empty input", () => {
		const result = computeVisibleRefs([]);
		expect(result.visible).toEqual([]);
		expect(result.overflowCount).toBe(0);
	});
});

describe("computeVisibleRefs edge cases", () => {
	it("respects budget boundary — refs that just fit are shown", () => {
		// Create refs with known estimated widths to test budget boundary.
		// Each 'ab' ref: name.length=2, textWidth=max(24, 2*7)=24, paddingX=6*2=12, badge=36
		// With gap=6 between badges, cumulative: 36, 78, 120, 162, 204, 246, 288...
		// REF_OVERFLOW_MAX_WIDTH=280, so 7th ref (288) would exceed budget
		const refs: RefData[] = Array.from({ length: 8 }, (_, i) => ({
			name: `r${i}`,
			ref_type: "LocalBranch" as const,
			commit_id: "a",
		}));
		const result = computeVisibleRefs(refs);
		// Verify total visible width is within budget
		let totalWidth = 0;
		for (let i = 0; i < result.visible.length; i++) {
			totalWidth += estimateRefBadgeWidth(result.visible[i]);
			if (i > 0) totalWidth += 6; // REF_LABEL_GAP
		}
		expect(totalWidth).toBeLessThanOrEqual(REF_OVERFLOW_MAX_WIDTH);
		expect(result.visible.length + result.overflowCount).toBe(8);
	});

	it("overflow badge reservation scales with ref count", () => {
		// With 100 refs, overflow badge shows +99 (3 chars) — wider than +3 (2 chars)
		const smallSet: RefData[] = Array.from({ length: 5 }, (_, i) => ({
			name: `br-${i}`,
			ref_type: "LocalBranch" as const,
			commit_id: "a",
		}));
		const largeSet: RefData[] = Array.from({ length: 100 }, (_, i) => ({
			name: `br-${i.toString().padStart(3, "0")}`,
			ref_type: "LocalBranch" as const,
			commit_id: "a",
		}));
		const smallResult = computeVisibleRefs(smallSet);
		const largeResult = computeVisibleRefs(largeSet);
		// Large set should reserve more for overflow badge, so may show fewer visible refs
		expect(largeResult.visible.length).toBeLessThanOrEqual(smallResult.visible.length);
	});
});

// ── Hit-test alignment with visible refs ──

describe("hitTest ref alignment", () => {
	it("only targets visible refs, not overflow ones", () => {
		// Create a commit with many refs
		const manyRefs: RefData[] = Array.from({ length: 20 }, (_, i) => ({
			name: `branch-${i.toString().padStart(2, "0")}`,
			ref_type: "LocalBranch" as const,
			commit_id: "commit-0",
		}));

		const commits = [{ id: "commit-0", message: "test", author_name: "test", author_email: "test@test.com", time: 0, parent_ids: [] }];
		const layout = assignLanes(commits, manyRefs);

		// Compute which refs should be visible
		const sorted = sortRefsByPriority(manyRefs);
		const { visible: visibleRefs, overflowCount } = computeVisibleRefs(sorted);
		expect(overflowCount).toBeGreaterThan(0);

		// Hit-test each visible badge — should return ref hit
		const laneCount = layout.laneCount;
		const graphEndX = 8 + laneCount * 16; // GRAPH_PADDING_LEFT + laneCount * LANE_WIDTH
		let cursorX = graphEndX;

		for (const ref of visibleRefs) {
			const badgeWidth = estimateRefBadgeWidth(ref);
			const hitX = cursorX + badgeWidth / 2;
			const hitY = 16; // ROW_HEIGHT / 2 (center of row 0)
			const result = hitTest(layout, hitX, hitY, laneCount);
			expect(result.type).toBe("ref");
			if (result.type === "ref") {
				expect(result.ref.name).toBe(ref.name);
			}
			cursorX += badgeWidth + 6; // REF_LABEL_GAP
		}

		// Hit-test immediately after last visible badge — should NOT be ref
		const nearResult = hitTest(layout, cursorX + 5, 16, laneCount);
		expect(nearResult.type).toBe("row");

		// Hit-test far past visible badges — should return row, not ref
		const farX = graphEndX + REF_OVERFLOW_MAX_WIDTH + 200;
		const farResult = hitTest(layout, farX, 16, laneCount);
		expect(farResult.type).toBe("row");
	});

	it("works with mixed ref types and priority ordering", () => {
		const mixedRefs: RefData[] = [
			{ name: "origin/main", ref_type: "RemoteBranch", commit_id: "commit-0" },
			{ name: "HEAD", ref_type: "Head", commit_id: "commit-0" },
			{ name: "main", ref_type: "LocalBranch", commit_id: "commit-0" },
			{ name: "v1.0", ref_type: "Tag", commit_id: "commit-0" },
		];

		const commits = [{ id: "commit-0", message: "test", author_name: "test", author_email: "test@test.com", time: 0, parent_ids: [] }];
		const layout = assignLanes(commits, mixedRefs);

		const laneCount = layout.laneCount;
		const graphEndX = 8 + laneCount * 16;

		// First visible ref should be Head (highest priority)
		const sorted = sortRefsByPriority(mixedRefs);
		const firstBadgeWidth = estimateRefBadgeWidth(sorted[0]);
		const hitX = graphEndX + firstBadgeWidth / 2;
		const result = hitTest(layout, hitX, 16, laneCount);
		expect(result.type).toBe("ref");
		if (result.type === "ref") {
			expect(result.ref.ref_type).toBe("Head");
		}
	});
});
