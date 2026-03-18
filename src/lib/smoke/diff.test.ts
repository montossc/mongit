import { MergeView } from "@codemirror/merge";
import { EditorState } from "@codemirror/state";
import { describe, expect, it } from "vitest";
import { baseExtensions, languageExtension } from "../utils/codemirror-config";

describe("Diff Viewer Smoke Test", () => {
	it("should initialize MergeView with 10,000 lines efficiently", () => {
		const lineCount = 10_000;
		const original = Array.from(
			{ length: lineCount },
			(_, i) => `line ${i}`,
		).join("\n");
		const modified = Array.from({ length: lineCount }, (_, i) =>
			i % 10 === 0 ? `modified ${i}` : `line ${i}`,
		).join("\n");

		const extensions = [
			EditorState.readOnly.of(true),
			...baseExtensions(),
			...languageExtension("test.ts"),
		];

		const container = document.createElement("div");
		const start = performance.now();

		const mv = new MergeView({
			parent: container,
			a: { doc: original, extensions },
			b: { doc: modified, extensions },
		});

		const elapsed = performance.now() - start;
		console.log(`[Smoke Test] 10k Lines Diff Init: ${elapsed.toFixed(1)}ms`);

		expect(elapsed).toBeLessThan(300); // 10k lines diff is substantial
		expect(mv.a.state.doc.lines).toBe(lineCount);

		mv.destroy();
	});
});
