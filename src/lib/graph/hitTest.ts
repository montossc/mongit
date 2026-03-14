import {
	GRAPH_PADDING_LEFT,
	LANE_WIDTH,
	NODE_RADIUS,
	ROW_HEIGHT,
	TEXT_PADDING_LEFT,
} from "./render";
import type { CommitNode, LayoutResult, RefData } from "./types";

export type HitTarget =
	| { type: "node"; node: CommitNode }
	| { type: "ref"; node: CommitNode; ref: RefData }
	| { type: "row"; node: CommitNode }
	| { type: "none" };

const REF_GAP = 6;

function estimateRefBadgeWidth(ref: RefData): number {
	const basePadding = ref.ref_type === "Head" ? 14 : 12;
	const textEstimate = Math.max(24, ref.name.length * 7);
	return basePadding + textEstimate;
}

function estimateRefZoneWidth(node: CommitNode): number {
	if (node.refs.length === 0) return 0;
	return (
		node.refs.reduce((total, ref) => total + estimateRefBadgeWidth(ref), 0) +
		REF_GAP * Math.max(0, node.refs.length - 1)
	);
}

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
		const refZoneEnd = Math.max(
			textStartX,
			textStartX + estimateRefZoneWidth(node),
		);

		if (canvasX >= refZoneStart && canvasX <= refZoneEnd) {
			const firstRef = node.refs[0];
			if (firstRef) {
				return { type: "ref", node, ref: firstRef };
			}
		}
	}

	return { type: "row", node };
}
