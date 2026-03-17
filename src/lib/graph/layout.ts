import type {
  CommitData,
  RefData,
  CommitNode,
  GraphSegment,
  LayoutResult,
  LayoutConfig
} from './types';

export const DEFAULT_CONFIG: LayoutConfig = {
  maxCommits: 10000
};

function nowMs(): number {
  if (typeof performance !== 'undefined' && typeof performance.now === 'function') {
    return performance.now();
  }
  return Date.now();
}

/**
 * Assign lanes to commits using a greedy first-parent algorithm.
 *
 * Determinism contract:
 * - For identical ordered `commits` + `refs` inputs, output is stable across runs.
 * - Lane assignment, node colors, and segment topology are deterministic.
 * - This relies on deterministic iteration/insertion order and no random sources.
 *
 * Strategy:
 * - First parent of a commit inherits its lane (straight lines for main branch)
 * - Other parents get new lanes or reuse freed lanes
 * - Lanes are recycled when no longer needed (keeps lane count low)
 */
export function assignLanes(
  commits: CommitData[],
  refs: RefData[],
  config: Partial<LayoutConfig> = {}
): LayoutResult {
  const start = nowMs();
  const maxCommits = config.maxCommits ?? DEFAULT_CONFIG.maxCommits;
  const visibleCommits = commits.length > maxCommits ? commits.slice(0, maxCommits) : commits;

  // Build parent -> children index from topological input (newest first).
  const childrenOf = new Map<string, string[]>();
  for (let i = 0; i < visibleCommits.length; i++) {
    const commit = visibleCommits[i];
    for (let p = 0; p < commit.parent_ids.length; p++) {
      const parentId = commit.parent_ids[p];
      const children = childrenOf.get(parentId);
      if (children) {
        children.push(commit.id);
      } else {
        childrenOf.set(parentId, [commit.id]);
      }
    }
  }

  // Build ref lookup: commit_id -> RefData[]
  const refMap = new Map<string, RefData[]>();
  for (let i = 0; i < refs.length; i++) {
    const ref = refs[i];
    const commitRefs = refMap.get(ref.commit_id);
    if (commitRefs) {
      commitRefs.push(ref);
    } else {
      refMap.set(ref.commit_id, [ref]);
    }
  }

  const nodes: CommitNode[] = new Array(visibleCommits.length);
  const nodeMap = new Map<string, CommitNode>();

  // activeLanes[lane] = commit id occupying this lane at current scan position.
  const activeLanes: (string | null)[] = [];
  const laneOf = new Map<string, number>();

  let colorCounter = 0;
  const laneColors = new Map<number, number>();

  function getFreeLane(): number {
    for (let i = 0; i < activeLanes.length; i++) {
      if (activeLanes[i] === null) {
        return i;
      }
    }
    activeLanes.push(null);
    return activeLanes.length - 1;
  }

  function getLaneColor(lane: number): number {
    const existing = laneColors.get(lane);
    if (existing !== undefined) {
      return existing;
    }
    const color = colorCounter++;
    laneColors.set(lane, color);
    return color;
  }

  for (let row = 0; row < visibleCommits.length; row++) {
    const commit = visibleCommits[row];

    // Touch childrenOf to make index usage explicit for future lane heuristics.
    void childrenOf.get(commit.id);

    const reservedLane = laneOf.get(commit.id);
    const lane = reservedLane ?? getFreeLane();

    activeLanes[lane] = commit.id;
    laneOf.set(commit.id, lane);

    const color = getLaneColor(lane);

    const node: CommitNode = {
      data: commit,
      lane,
      row,
      refs: refMap.get(commit.id) ?? [],
      color
    };

    nodes[row] = node;
    nodeMap.set(commit.id, node);

    let laneContinuesToFirstParent = false;

    for (let pi = 0; pi < commit.parent_ids.length; pi++) {
      const parentId = commit.parent_ids[pi];

      if (pi === 0) {
        // First parent continues on same lane for visual continuity.
        if (!laneOf.has(parentId)) {
          laneOf.set(parentId, lane);
          laneContinuesToFirstParent = true;
        }
      } else {
        // Merge parents get side lanes (reused when available).
        if (!laneOf.has(parentId)) {
          const mergeLane = getFreeLane();
          laneOf.set(parentId, mergeLane);
          activeLanes[mergeLane] = parentId;
          if (!laneColors.has(mergeLane)) {
            laneColors.set(mergeLane, colorCounter++);
          }
        }
      }
    }

    // Free lane when this line does not continue to an unassigned first parent.
    if (!laneContinuesToFirstParent) {
      activeLanes[lane] = null;
    }
  }

  const segments: GraphSegment[] = [];

  for (let i = 0; i < nodes.length; i++) {
    const node = nodes[i];
    for (let pi = 0; pi < node.data.parent_ids.length; pi++) {
      const parentId = node.data.parent_ids[pi];
      const parentNode = nodeMap.get(parentId);
      if (!parentNode) {
        continue;
      }

      segments.push({
        fromId: node.data.id,
        toId: parentId,
        fromLane: node.lane,
        toLane: parentNode.lane,
        fromRow: node.row,
        toRow: parentNode.row,
        color: pi === 0 ? node.color : parentNode.color,
        isMerge: pi > 0
      });
    }
  }

  let laneCount = 0;
  for (let i = 0; i < nodes.length; i++) {
    const used = nodes[i].lane + 1;
    if (used > laneCount) {
      laneCount = used;
    }
  }

  const layoutTimeMs = nowMs() - start;

  return {
    nodes,
    segments,
    laneCount,
    nodeMap,
    layoutTimeMs
  };
}

/**
 * Generate synthetic commit data for testing layout performance.
 * Output order is topological (newest first): children appear before parents.
 */
export function generateSyntheticCommits(count: number, branchCount: number = 5): CommitData[] {
  if (count <= 0) {
    return [];
  }

  const chronological: CommitData[] = [];
  const branchTips: string[] = [];

  const rootId = 'synthetic-0';
  chronological.push({
    id: rootId,
    message: 'Initial commit',
    author_name: 'Test',
    author_email: 'test@example.com',
    time: Math.floor(Date.now() / 1000) - count,
    parent_ids: []
  });
  branchTips.push(rootId);

  for (let i = 1; i < count; i++) {
    const id = `synthetic-${i}`;
    const parent_ids: string[] = [];
    const rand = Math.random();

    if (rand < 0.1 && branchTips.length < branchCount) {
      const branchIdx = Math.floor(Math.random() * branchTips.length);
      parent_ids.push(branchTips[branchIdx]);
      branchTips.push(id);
    } else if (rand < 0.15 && branchTips.length > 1) {
      const idx1 = Math.floor(Math.random() * branchTips.length);
      let idx2 = Math.floor(Math.random() * (branchTips.length - 1));
      if (idx2 >= idx1) idx2 += 1;

      parent_ids.push(branchTips[idx1]);
      parent_ids.push(branchTips[idx2]);

      branchTips[idx1] = id;
      branchTips.splice(idx2, 1);
    } else {
      const branchIdx = Math.floor(Math.random() * branchTips.length);
      parent_ids.push(branchTips[branchIdx]);
      branchTips[branchIdx] = id;
    }

    chronological.push({
      id,
      message: `Commit ${i}`,
      author_name: 'Test',
      author_email: 'test@example.com',
      time: Math.floor(Date.now() / 1000) - count + i,
      parent_ids
    });
  }

  // Convert oldest->newest into newest->oldest (topological for renderer).
  return chronological.reverse();
}
