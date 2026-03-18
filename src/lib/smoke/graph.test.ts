import { describe, expect, it } from "vitest";
import { assignLanes, generateSyntheticCommits } from "../graph/layout";

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
