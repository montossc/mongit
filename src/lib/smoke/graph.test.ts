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
		expect(result.visible.length).toBeGreaterThanOrEqual(1);
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
		const { visible: visibleRefs } = computeVisibleRefs(sorted);

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

		// Hit-test far past visible badges — should return row, not ref
		const farX = graphEndX + REF_OVERFLOW_MAX_WIDTH + 200;
		const farResult = hitTest(layout, farX, 16, laneCount);
		expect(farResult.type).toBe("row");
	});
});
