import {
	GRAPH_PADDING_LEFT,
	LANE_WIDTH,
	NODE_RADIUS,
	ROW_HEIGHT,
	TEXT_PADDING_LEFT,
	REF_LABEL_GAP,
	estimateRefBadgeWidth,
	sortRefsByPriority,
	computeVisibleRefs,
} from "./render";
import type { CommitNode, LayoutResult, RefData } from "./types";

export type HitTarget =
	| { type: "node"; node: CommitNode }
	| { type: "ref"; node: CommitNode; ref: RefData }
	| { type: "row"; node: CommitNode }
	| { type: "none" };


export function hitTest(
	layout: LayoutResult,
	canvasX: number,
	absoluteY: number,
	laneCount: number,
): HitTarget {
	const row = Math.floor(absoluteY / ROW_HEIGHT);
	if (row < 0 || row >= layout.nodes.length) {
		return { type: "none" };
	}

	const node = layout.nodes[row];
	if (!node) {
		return { type: "none" };
	}

	const nodeCenterX =
		GRAPH_PADDING_LEFT + node.lane * LANE_WIDTH + LANE_WIDTH / 2;
	const rowCenterY = row * ROW_HEIGHT + ROW_HEIGHT / 2;
	const dx = canvasX - nodeCenterX;
	const dy = absoluteY - rowCenterY;
	const nodeHitRadius = NODE_RADIUS + 2;

	if (dx * dx + dy * dy <= nodeHitRadius * nodeHitRadius) {
		return { type: "node", node };
	}

	const graphEndX = GRAPH_PADDING_LEFT + laneCount * LANE_WIDTH;
	const textStartX = graphEndX + TEXT_PADDING_LEFT;

	if (node.refs.length > 0) {
		const refZoneStart = graphEndX;
		let cursorX = refZoneStart;

		const sorted = sortRefsByPriority(node.refs);
		const { visible: visibleRefs } = computeVisibleRefs(sorted);

		for (const ref of visibleRefs) {
			const badgeWidth = estimateRefBadgeWidth(ref);
			if (canvasX >= cursorX && canvasX <= cursorX + badgeWidth) {
				return { type: "ref", node, ref };
			}
			cursorX += badgeWidth + REF_LABEL_GAP;
		}
	}

	return { type: "row", node };
}
