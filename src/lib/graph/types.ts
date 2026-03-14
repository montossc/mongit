/** Raw commit data from Rust backend (matches CommitInfo in repository.rs) */
export interface CommitData {
  id: string;
  message: string;
  author_name: string;
  author_email: string;
  time: number; // Unix timestamp (seconds)
  parent_ids: string[];
}

/** Ref data from Rust backend (matches RefInfo in repository.rs) */
export interface RefData {
  name: string;
  ref_type: 'LocalBranch' | 'RemoteBranch' | 'Tag' | 'Head';
  commit_id: string;
}

/** A commit positioned in the graph with lane assignment */
export interface CommitNode {
  /** Original commit data */
  data: CommitData;
  /** Horizontal lane (column) index, 0-based from left */
  lane: number;
  /** Vertical row index, 0-based from top */
  row: number;
  /** Refs pointing to this commit */
  refs: RefData[];
  /** Color index for the branch lane */
  color: number;
}

/** A line segment connecting two commits in the graph */
export interface GraphSegment {
  /** Source commit ID */
  fromId: string;
  /** Target commit ID (parent) */
  toId: string;
  /** Source lane */
  fromLane: number;
  /** Target lane */
  toLane: number;
  /** Source row */
  fromRow: number;
  /** Target row */
  toRow: number;
  /** Color index matching the branch */
  color: number;
  /** Whether this is a merge edge (second+ parent) */
  isMerge: boolean;
}

/** Complete layout result ready for rendering */
export interface LayoutResult {
  /** All commits with positions */
  nodes: CommitNode[];
  /** All connecting segments */
  segments: GraphSegment[];
  /** Total number of lanes used */
  laneCount: number;
  /** Lookup map: commit ID → CommitNode */
  nodeMap: Map<string, CommitNode>;
  /** Layout computation time in ms */
  layoutTimeMs: number;
}

/** Configuration for the layout algorithm */
export interface LayoutConfig {
  /** Maximum number of commits to layout */
  maxCommits: number;
}
