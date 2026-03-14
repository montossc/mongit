import type { CommitNode, GraphSegment, LayoutResult, RefData } from './types';

export const ROW_HEIGHT = 32;
export const LANE_WIDTH = 16;
export const NODE_RADIUS = 4;
export const GRAPH_PADDING_LEFT = 8;
export const TEXT_PADDING_LEFT = 12;
export const FONT_SIZE = 12;

const VISIBLE_BUFFER_ROWS = 5;

export interface GraphTheme {
  colors: string[];
  bgColor: string;
  textPrimary: string;
  textSecondary: string;
  accentColor: string;
  fontMono: string;
  selectedBg: string;
  hoverBg: string;
}

interface RenderOptions {
  theme: GraphTheme;
  dpr: number;
  canvasWidth: number;
  canvasHeight: number;
  scrollTop: number;
  selectedId: string | null;
  hoveredId: string | null;
  laneCount: number;
}

interface VisibleRange {
  first: number;
  last: number;
}

/** Read CSS custom properties and build theme object */
export function resolveTheme(element: HTMLElement): GraphTheme {
  const styles = getComputedStyle(element);
  const read = (key: string, fallback = ''): string => {
    const value = styles.getPropertyValue(key).trim();
    return value || fallback;
  };

  const colors = Array.from({ length: 10 }, (_, idx) =>
    read(`--graph-color-${idx}`, '#53C1DE')
  );

  return {
    colors,
    bgColor: read('--color-bg', '#0F1117'),
    textPrimary: read('--color-text-primary', '#E8EAED'),
    textSecondary: read('--color-text-secondary', '#8B8FA3'),
    accentColor: read('--color-accent', '#53C1DE'),
    fontMono: read('--font-mono', "'SF Mono', monospace"),
    selectedBg: read('--color-bg-active', '#323744'),
    hoverBg: read('--color-bg-hover', '#2A2E3A')
  };
}

/** Main render function — draws visible portion of graph */
export function renderGraph(ctx: CanvasRenderingContext2D, layout: LayoutResult, options: RenderOptions): void {
  const {
    theme,
    dpr,
    canvasWidth,
    canvasHeight,
    scrollTop,
    selectedId,
    hoveredId,
    laneCount
  } = options;

  const visible = getVisibleRange(layout.nodes.length, scrollTop, canvasHeight);
  const graphTextStartX = GRAPH_PADDING_LEFT + laneCount * LANE_WIDTH + TEXT_PADDING_LEFT;
  const rightPadding = 12;

  ctx.fillStyle = theme.bgColor;
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  drawRowHighlights(ctx, layout, visible, {
    selectedId,
    hoveredId,
    selectedBg: theme.selectedBg,
    hoverBg: theme.hoverBg,
    canvasWidth,
    scrollTop
  });

  drawEdges(ctx, layout.segments, visible, scrollTop, theme, dpr);
  drawNodes(ctx, layout.nodes, visible, scrollTop, theme, selectedId);
  drawRefLabels(ctx, layout.nodes, visible, scrollTop, theme, graphTextStartX);
  drawCommitText(ctx, layout.nodes, visible, scrollTop, theme, graphTextStartX, canvasWidth - rightPadding);
}

/** Convert lane index to x coordinate */
function laneToX(lane: number): number {
  return GRAPH_PADDING_LEFT + lane * LANE_WIDTH + LANE_WIDTH / 2;
}

/** Convert row index to y coordinate (center of row) */
function rowToY(row: number, scrollTop: number): number {
  return row * ROW_HEIGHT + ROW_HEIGHT / 2 - scrollTop;
}

/** Format relative time (e.g., "2h ago", "3d ago") */
function formatRelativeTime(unixSeconds: number): string {
  const delta = Math.max(0, Math.floor(Date.now() / 1000) - unixSeconds);
  if (delta < 60) return `${delta}s ago`;

  const minutes = Math.floor(delta / 60);
  if (minutes < 60) return `${minutes}m ago`;

  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;

  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}d ago`;

  const weeks = Math.floor(days / 7);
  if (weeks < 5) return `${weeks}w ago`;

  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo ago`;

  const years = Math.floor(days / 365);
  return `${years}y ago`;
}

/** Truncate text to fit width */
function truncateText(ctx: CanvasRenderingContext2D, text: string, maxWidth: number): string {
  if (maxWidth <= 0) return '';
  if (ctx.measureText(text).width <= maxWidth) return text;

  const ellipsis = '…';
  const ellipsisWidth = ctx.measureText(ellipsis).width;
  if (ellipsisWidth >= maxWidth) return ellipsis;

  let low = 0;
  let high = text.length;
  while (low < high) {
    const mid = Math.ceil((low + high) / 2);
    const candidate = text.slice(0, mid);
    const width = ctx.measureText(candidate).width + ellipsisWidth;
    if (width <= maxWidth) {
      low = mid;
    } else {
      high = mid - 1;
    }
  }

  return `${text.slice(0, low)}${ellipsis}`;
}

function getVisibleRange(totalRows: number, scrollTop: number, canvasHeight: number): VisibleRange {
  if (totalRows <= 0) {
    return { first: 0, last: -1 };
  }

  const firstVisible = Math.floor(scrollTop / ROW_HEIGHT);
  const lastVisible = Math.ceil((scrollTop + canvasHeight) / ROW_HEIGHT);

  return {
    first: Math.max(0, firstVisible - VISIBLE_BUFFER_ROWS),
    last: Math.min(totalRows - 1, lastVisible + VISIBLE_BUFFER_ROWS)
  };
}

function isRowVisible(row: number, visible: VisibleRange): boolean {
  return row >= visible.first && row <= visible.last;
}

function drawRowHighlights(
  ctx: CanvasRenderingContext2D,
  layout: LayoutResult,
  visible: VisibleRange,
  options: {
    selectedId: string | null;
    hoveredId: string | null;
    selectedBg: string;
    hoverBg: string;
    canvasWidth: number;
    scrollTop: number;
  }
): void {
  const { selectedId, hoveredId, selectedBg, hoverBg, canvasWidth, scrollTop } = options;

  const selectedNode = selectedId ? layout.nodeMap.get(selectedId) : undefined;
  if (selectedNode && isRowVisible(selectedNode.row, visible)) {
    const y = selectedNode.row * ROW_HEIGHT - scrollTop;
    ctx.fillStyle = selectedBg;
    ctx.fillRect(0, y, canvasWidth, ROW_HEIGHT);
  }

  const hoveredNode = hoveredId ? layout.nodeMap.get(hoveredId) : undefined;
  if (
    hoveredNode &&
    hoveredId !== selectedId &&
    isRowVisible(hoveredNode.row, visible)
  ) {
    const y = hoveredNode.row * ROW_HEIGHT - scrollTop;
    ctx.fillStyle = hoverBg;
    ctx.fillRect(0, y, canvasWidth, ROW_HEIGHT);
  }
}

function drawEdges(
  ctx: CanvasRenderingContext2D,
  segments: GraphSegment[],
  visible: VisibleRange,
  scrollTop: number,
  theme: GraphTheme,
  dpr: number
): void {
  const groups = new Map<string, { color: string; isMerge: boolean; segments: GraphSegment[] }>();

  for (const segment of segments) {
    if (!isRowVisible(segment.fromRow, visible) && !isRowVisible(segment.toRow, visible)) {
      continue;
    }

    const color = theme.colors[segment.color % theme.colors.length] ?? theme.accentColor;
    const key = `${color}|${segment.isMerge ? 'merge' : 'normal'}`;

    const group = groups.get(key);
    if (group) {
      group.segments.push(segment);
    } else {
      groups.set(key, { color, isMerge: segment.isMerge, segments: [segment] });
    }
  }

  const crispOffset = 0.5 / Math.max(1, dpr);

  for (const group of groups.values()) {
    ctx.beginPath();
    ctx.strokeStyle = group.color;
    ctx.lineWidth = group.isMerge ? 1.0 : 1.5;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.setLineDash(group.isMerge ? [3, 3] : []);

    for (const segment of group.segments) {
      const x1 = laneToX(segment.fromLane) + crispOffset;
      const y1 = rowToY(segment.fromRow, scrollTop);
      const x2 = laneToX(segment.toLane) + crispOffset;
      const y2 = rowToY(segment.toRow, scrollTop);

      if (segment.fromLane === segment.toLane) {
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
      } else {
        const dy = y2 - y1;
        const controlOffset = Math.max(10, Math.abs(dy) * 0.35);
        ctx.moveTo(x1, y1);
        ctx.bezierCurveTo(x1, y1 + controlOffset, x2, y2 - controlOffset, x2, y2);
      }
    }

    ctx.stroke();
    ctx.setLineDash([]);
  }
}

function drawNodes(
  ctx: CanvasRenderingContext2D,
  nodes: CommitNode[],
  visible: VisibleRange,
  scrollTop: number,
  theme: GraphTheme,
  selectedId: string | null
): void {
  const buckets = new Map<string, CommitNode[]>();

  for (let row = visible.first; row <= visible.last; row++) {
    const node = nodes[row];
    if (!node) continue;

    const color = theme.colors[node.color % theme.colors.length] ?? theme.accentColor;
    const bucket = buckets.get(color);
    if (bucket) {
      bucket.push(node);
    } else {
      buckets.set(color, [node]);
    }
  }

  for (const [color, bucket] of buckets) {
    ctx.beginPath();
    ctx.fillStyle = color;

    for (const node of bucket) {
      const x = laneToX(node.lane);
      const y = rowToY(node.row, scrollTop);
      ctx.moveTo(x + NODE_RADIUS, y);
      ctx.arc(x, y, NODE_RADIUS, 0, Math.PI * 2);
    }

    ctx.fill();
  }

  if (selectedId) {
    const selectedNode = nodes.find((node) => node.data.id === selectedId);
    if (selectedNode && isRowVisible(selectedNode.row, visible)) {
      const x = laneToX(selectedNode.lane);
      const y = rowToY(selectedNode.row, scrollTop);

      ctx.beginPath();
      ctx.fillStyle = theme.bgColor;
      ctx.arc(x, y, NODE_RADIUS + 2.5, 0, Math.PI * 2);
      ctx.fill();

      ctx.beginPath();
      ctx.strokeStyle = theme.accentColor;
      ctx.lineWidth = 2;
      ctx.arc(x, y, NODE_RADIUS + 2.5, 0, Math.PI * 2);
      ctx.stroke();

      ctx.beginPath();
      ctx.fillStyle = theme.colors[selectedNode.color % theme.colors.length] ?? theme.accentColor;
      ctx.arc(x, y, NODE_RADIUS + 1, 0, Math.PI * 2);
      ctx.fill();
    }
  }
}

function drawRefLabels(
  ctx: CanvasRenderingContext2D,
  nodes: CommitNode[],
  visible: VisibleRange,
  scrollTop: number,
  theme: GraphTheme,
  graphTextStartX: number
): void {
  const badgeFont = `11px ${theme.fontMono}`;
  const rowSpacing = 6;
  const xStart = graphTextStartX - 4;

  for (let row = visible.first; row <= visible.last; row++) {
    const node = nodes[row];
    if (!node || node.refs.length === 0) continue;

    const yCenter = rowToY(node.row, scrollTop);
    let cursorX = xStart;

    for (const ref of node.refs) {
      const isHead = ref.ref_type === 'Head';
      const paddingX = isHead ? 7 : 6;
      const height = 16;

      ctx.font = `${isHead ? '700' : '500'} ${badgeFont}`;
      const text = ref.name;
      const textWidth = ctx.measureText(text).width;
      const badgeWidth = Math.ceil(textWidth + paddingX * 2);
      const y = yCenter - height / 2;

      const style = getRefStyle(ref, theme);
      roundedRectPath(ctx, cursorX, y, badgeWidth, height, 4);
      ctx.fillStyle = style.bg;
      ctx.fill();

      ctx.strokeStyle = style.border;
      ctx.lineWidth = isHead ? 1.5 : 1;
      ctx.stroke();

      ctx.fillStyle = style.text;
      ctx.textAlign = 'left';
      ctx.textBaseline = 'middle';
      ctx.fillText(text, cursorX + paddingX, yCenter + 0.5);

      cursorX += badgeWidth + rowSpacing;
    }
  }
}

function drawCommitText(
  ctx: CanvasRenderingContext2D,
  nodes: CommitNode[],
  visible: VisibleRange,
  scrollTop: number,
  theme: GraphTheme,
  textStartX: number,
  rightEdge: number
): void {
  for (let row = visible.first; row <= visible.last; row++) {
    const node = nodes[row];
    if (!node) continue;

    const y = rowToY(node.row, scrollTop);
    const meta = `${node.data.author_name} · ${formatRelativeTime(node.data.time)}`;

    ctx.font = `400 ${FONT_SIZE}px var(--font-sans, -apple-system, system-ui, sans-serif)`;
    const metaWidth = ctx.measureText(meta).width;
    const hash = node.data.id.slice(0, 7);

    ctx.font = `500 ${FONT_SIZE}px ${theme.fontMono}`;
    const hashText = `${hash} `;
    const hashWidth = ctx.measureText(hashText).width;

    const availableForMessage = Math.max(0, rightEdge - textStartX - hashWidth - metaWidth - 20);

    ctx.font = `400 ${FONT_SIZE}px var(--font-sans, -apple-system, system-ui, sans-serif)`;
    const message = truncateText(ctx, node.data.message, availableForMessage);

    ctx.textBaseline = 'middle';
    ctx.textAlign = 'left';

    ctx.font = `500 ${FONT_SIZE}px ${theme.fontMono}`;
    ctx.fillStyle = theme.textSecondary;
    ctx.fillText(hashText, textStartX, y + 0.5);

    ctx.font = `400 ${FONT_SIZE}px var(--font-sans, -apple-system, system-ui, sans-serif)`;
    ctx.fillStyle = theme.textPrimary;
    ctx.fillText(message, textStartX + hashWidth, y + 0.5);

    ctx.font = `400 ${FONT_SIZE}px var(--font-sans, -apple-system, system-ui, sans-serif)`;
    ctx.fillStyle = theme.textSecondary;
    ctx.textAlign = 'right';
    ctx.fillText(meta, rightEdge, y + 0.5);
  }

  ctx.textAlign = 'left';
}

function getRefStyle(ref: RefData, theme: GraphTheme): { bg: string; border: string; text: string } {
  switch (ref.ref_type) {
    case 'LocalBranch':
      return {
        bg: withAlpha(theme.accentColor, 0.18),
        border: withAlpha(theme.accentColor, 0.7),
        text: theme.accentColor
      };
    case 'RemoteBranch':
      return {
        bg: withAlpha(theme.accentColor, 0.12),
        border: withAlpha(theme.accentColor, 0.4),
        text: withAlpha(theme.accentColor, 0.86)
      };
    case 'Tag': {
      const warning = theme.colors[3] ?? '#FBBF24';
      return {
        bg: withAlpha(warning, 0.18),
        border: withAlpha(warning, 0.65),
        text: warning
      };
    }
    case 'Head':
      return {
        bg: withAlpha(theme.textPrimary, 0.14),
        border: withAlpha(theme.textPrimary, 0.55),
        text: theme.textPrimary
      };
    default:
      return {
        bg: withAlpha(theme.textSecondary, 0.12),
        border: withAlpha(theme.textSecondary, 0.4),
        text: theme.textSecondary
      };
  }
}

function roundedRectPath(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number
): void {
  const r = Math.min(radius, width / 2, height / 2);
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.lineTo(x + width - r, y);
  ctx.quadraticCurveTo(x + width, y, x + width, y + r);
  ctx.lineTo(x + width, y + height - r);
  ctx.quadraticCurveTo(x + width, y + height, x + width - r, y + height);
  ctx.lineTo(x + r, y + height);
  ctx.quadraticCurveTo(x, y + height, x, y + height - r);
  ctx.lineTo(x, y + r);
  ctx.quadraticCurveTo(x, y, x + r, y);
  ctx.closePath();
}

function withAlpha(color: string, alpha: number): string {
  const normalizedAlpha = Math.max(0, Math.min(alpha, 1));

  if (color.startsWith('#')) {
    const hex = color.slice(1);
    if (hex.length === 3 || hex.length === 4) {
      const [r, g, b] = hex.slice(0, 3).split('').map((part) => parseInt(part + part, 16));
      return `rgba(${r}, ${g}, ${b}, ${normalizedAlpha})`;
    }

    if (hex.length === 6 || hex.length === 8) {
      const r = parseInt(hex.slice(0, 2), 16);
      const g = parseInt(hex.slice(2, 4), 16);
      const b = parseInt(hex.slice(4, 6), 16);
      return `rgba(${r}, ${g}, ${b}, ${normalizedAlpha})`;
    }
  }

  if (color.startsWith('rgb(')) {
    return color.replace('rgb(', 'rgba(').replace(')', `, ${normalizedAlpha})`);
  }

  if (color.startsWith('rgba(')) {
    return color.replace(/rgba\(([^)]+),\s*[^,]+\)$/u, `rgba($1, ${normalizedAlpha})`);
  }

  return color;
}
