<script lang="ts">
  import type { CommitNode, LayoutResult } from './types';
  import {
    ROW_HEIGHT,
    renderGraph,
    resolveTheme,
    type GraphTheme
  } from './render';
  import { hitTest } from './hitTest';
  import ContextMenu from './ContextMenu.svelte';

  interface Props {
    layout: LayoutResult | null;
    onSelectCommit?: (id: string) => void;
    onContextAction?: (action: string, node: CommitNode) => void;
  }

  let { layout, onSelectCommit, onContextAction }: Props = $props();

  let container = $state<HTMLDivElement | null>(null);
  let canvas = $state<HTMLCanvasElement | null>(null);
  let ctx = $state<CanvasRenderingContext2D | null>(null);

  let scrollTop = $state(0);
  let selectedId = $state<string | null>(null);
  let hoveredId = $state<string | null>(null);
  let contextMenu = $state<{ node: CommitNode; x: number; y: number } | null>(null);

  let width = $state(0);
  let height = $state(0);
  let dpr = $state(1);

  let theme = $state<GraphTheme | null>(null);

  let rafId = 0;
  let resizeObserver: ResizeObserver | null = null;

  const totalHeight = $derived((layout?.nodes.length ?? 0) * ROW_HEIGHT);

  function setupCanvasSize(): void {
    if (!canvas || !ctx || width <= 0 || height <= 0) return;

    dpr = window.devicePixelRatio || 1;
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }

  function queueRender(): void {
    if (rafId !== 0) return;

    rafId = window.requestAnimationFrame(() => {
      rafId = 0;
      draw();
    });
  }

  function draw(): void {
    if (!ctx || !theme || !layout || width <= 0 || height <= 0) {
      if (ctx && theme && width > 0 && height > 0) {
        ctx.fillStyle = theme.bgColor;
        ctx.fillRect(0, 0, width, height);
      }
      return;
    }

    renderGraph(ctx, layout, {
      theme,
      dpr,
      canvasWidth: width,
      canvasHeight: height,
      scrollTop,
      selectedId,
      hoveredId,
      laneCount: layout.laneCount
    });
  }

  function handleScroll(): void {
    if (!container) return;
    scrollTop = container.scrollTop;
    queueRender();
  }

  function rowFromClientY(clientY: number): number {
    if (!canvas) return -1;
    const rect = canvas.getBoundingClientRect();
    const y = clientY - rect.top;
    const absoluteY = y + scrollTop;
    return Math.floor(absoluteY / ROW_HEIGHT);
  }

  function nodeAtClientY(clientY: number): string | null {
    const activeLayout = layout;
    if (!activeLayout) return null;

    const row = rowFromClientY(clientY);
    if (row < 0 || row >= activeLayout.nodes.length) return null;

    const node = activeLayout.nodes[row];
    if (!node) return null;

    return node.data.id;
  }

  function nodeByRow(row: number): CommitNode | null {
    const activeLayout = layout;
    if (!activeLayout) return null;
    if (row < 0 || row >= activeLayout.nodes.length) return null;
    return activeLayout.nodes[row] ?? null;
  }

  function nodeById(id: string | null): CommitNode | null {
    if (!id || !layout) return null;
    return layout.nodeMap.get(id) ?? null;
  }

  function hitFromMouse(event: MouseEvent) {
    if (!layout || !canvas) return { type: 'none' } as const;

    const rect = canvas.getBoundingClientRect();
    const canvasX = event.clientX - rect.left;
    const canvasY = event.clientY - rect.top;
    const absoluteY = canvasY + scrollTop;

    return hitTest(layout, canvasX, absoluteY, layout.laneCount);
  }

  function applySelection(node: CommitNode): void {
    selectedId = node.data.id;
    onSelectCommit?.(node.data.id);
    queueRender();
  }

  function handleMouseMove(event: MouseEvent): void {
    const target = hitFromMouse(event);
    if (target.type === 'none') {
      hoveredId = null;
    } else {
      hoveredId = target.node.data.id;
    }
    queueRender();
  }

  function handleMouseLeave(): void {
    hoveredId = null;
    queueRender();
  }

  function handleClick(event: MouseEvent): void {
    contextMenu = null;

    const target = hitFromMouse(event);
    if (target.type === 'none') return;
    applySelection(target.node);
  }

  function handleContextMenu(event: MouseEvent): void {
    event.preventDefault();

    const target = hitFromMouse(event);
    if (target.type === 'none') {
      contextMenu = null;
      return;
    }

    applySelection(target.node);
    contextMenu = {
      node: target.node,
      x: event.clientX,
      y: event.clientY
    };
  }

  function closeContextMenu(): void {
    contextMenu = null;
  }

  function handleContextAction(action: string, node: CommitNode): void {
    onContextAction?.(action, node);
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (!layout || layout.nodes.length === 0) return;

    if (event.key === 'Escape') {
      if (contextMenu) {
        contextMenu = null;
        return;
      }
      selectedId = null;
      hoveredId = null;
      queueRender();
      return;
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      const selectedNode = nodeById(selectedId);
      const nextRow = Math.min(layout.nodes.length - 1, (selectedNode?.row ?? -1) + 1);
      const next = nodeByRow(nextRow);
      if (next) applySelection(next);
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      const selectedNode = nodeById(selectedId);
      const prevRow = Math.max(0, (selectedNode?.row ?? 1) - 1);
      const prev = nodeByRow(prevRow);
      if (prev) applySelection(prev);
      return;
    }

    if (event.key === 'Enter') {
      event.preventDefault();
      if (selectedId) {
        onSelectCommit?.(selectedId);
      }
    }
  }

  $effect(() => {
    if (!container || !canvas) return;

    if (!theme) {
      theme = resolveTheme(container);
    }

    ctx = canvas.getContext('2d', { desynchronized: true });
    if (!ctx) return;

    resizeObserver?.disconnect();
    resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;

      const box = entry.contentRect;
      width = Math.floor(box.width);
      height = Math.floor(box.height);
      setupCanvasSize();
      queueRender();
    });

    resizeObserver.observe(container);

    const onWindowResize = () => {
      setupCanvasSize();
      queueRender();
    };

    window.addEventListener('resize', onWindowResize);

    return () => {
      window.removeEventListener('resize', onWindowResize);
      resizeObserver?.disconnect();
      resizeObserver = null;

      if (rafId !== 0) {
        cancelAnimationFrame(rafId);
        rafId = 0;
      }
    };
  });

  $effect(() => {
    layout;
    scrollTop;
    selectedId;
    hoveredId;
    theme;
    width;
    height;
    queueRender();
  });
</script>

<div
  class="graph-container"
  bind:this={container}
  role="listbox"
  aria-label="Commit graph"
  tabindex="0"
  onscroll={handleScroll}
  onkeydown={handleKeydown}
>
  <div class="graph-spacer" style:height="{totalHeight}px"></div>
  <canvas
    bind:this={canvas}
    class="graph-canvas"
    onmousemove={handleMouseMove}
    onmouseleave={handleMouseLeave}
    onclick={handleClick}
    oncontextmenu={handleContextMenu}
  ></canvas>

  {#if contextMenu}
    <ContextMenu
      node={contextMenu.node}
      x={contextMenu.x}
      y={contextMenu.y}
      onClose={closeContextMenu}
      onAction={handleContextAction}
    />
  {/if}
</div>

<style>
  .graph-container {
    position: relative;
    overflow-y: auto;
    height: 100%;
    background: var(--color-bg);
    outline: none;
  }

  .graph-spacer {
    width: 100%;
    pointer-events: none;
  }

  .graph-canvas {
    position: sticky;
    top: 0;
    left: 0;
    display: block;
    width: 100%;
    cursor: default;
  }
</style>
