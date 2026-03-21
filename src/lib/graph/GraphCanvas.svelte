<script lang="ts">
  import { untrack } from 'svelte';
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
    selectedId?: string | null;
    onSelectCommit?: (id: string | null) => void;
    onContextAction?: (action: string, node: CommitNode) => void;
    onScrollChange?: (scrollTop: number) => void;
    onHeightChange?: (height: number) => void;
    onHoverCommit?: (id: string | null) => void;
    onKeyInteraction?: (key: string) => void;
  }

  let {
    layout,
    selectedId: controlledSelectedId,
    onSelectCommit,
    onContextAction,
    onScrollChange,
    onHeightChange,
    onHoverCommit,
    onKeyInteraction
  }: Props = $props();

  let container = $state<HTMLDivElement | null>(null);
  let canvas = $state<HTMLCanvasElement | null>(null);
  let ctx = $state<CanvasRenderingContext2D | null>(null);

  let scrollTop = $state(0);
  let internalSelectedId = $state<string | null>(null);
  let hoveredId = $state<string | null>(null);
  const effectiveSelectedId = $derived(
    controlledSelectedId !== undefined ? controlledSelectedId : internalSelectedId
  );
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
      selectedId: effectiveSelectedId,
      hoveredId,
      laneCount: layout.laneCount
    });
  }

  function handleScroll(): void {
    if (!container) return;
    scrollTop = container.scrollTop;
    onScrollChange?.(scrollTop);
    queueRender();
  }

  function scrollToRow(row: number): void {
    if (!container) return;
    const targetY = row * ROW_HEIGHT;
    container.scrollTop = targetY;
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
    internalSelectedId = node.data.id;
    onSelectCommit?.(node.data.id);
    queueRender();
  }

  function setHoveredId(nextHoveredId: string | null): void {
    if (hoveredId === nextHoveredId) return;
    hoveredId = nextHoveredId;
    onHoverCommit?.(nextHoveredId);
    queueRender();
  }

  function handleMouseMove(event: MouseEvent): void {
    const target = hitFromMouse(event);
    if (target.type === 'none') {
      setHoveredId(null);
    } else {
      setHoveredId(target.node.data.id);
    }
  }

  function handleMouseLeave(): void {
    setHoveredId(null);
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
    onKeyInteraction?.(event.key);

    if (!layout || layout.nodes.length === 0) return;

    if (event.key === 'Escape') {
      if (contextMenu) {
        contextMenu = null;
        return;
      }
      internalSelectedId = null;
      onSelectCommit?.(null);
      hoveredId = null;
      queueRender();
      return;
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      const selectedNode = nodeById(effectiveSelectedId);
      const nextRow = Math.min(layout.nodes.length - 1, (selectedNode?.row ?? -1) + 1);
      const next = nodeByRow(nextRow);
      if (next) applySelection(next);
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      const selectedNode = nodeById(effectiveSelectedId);
      const prevRow = Math.max(0, (selectedNode?.row ?? 1) - 1);
      const prev = nodeByRow(prevRow);
      if (prev) applySelection(prev);
      return;
    }

    if (event.key === 'Home') {
      event.preventDefault();
      const first = nodeByRow(0);
      if (first) {
        applySelection(first);
        scrollToRow(0);
      }
      return;
    }

    if (event.key === 'End') {
      event.preventDefault();
      const lastRow = layout.nodes.length - 1;
      const last = nodeByRow(lastRow);
      if (last) {
        applySelection(last);
        scrollToRow(lastRow);
      }
      return;
    }

    if (event.key === 'PageDown') {
      event.preventDefault();
      const rowsPerPage = Math.max(1, Math.floor(height / ROW_HEIGHT));
      const selectedNode = nodeById(effectiveSelectedId);
      const targetRow = Math.min(layout.nodes.length - 1, (selectedNode?.row ?? 0) + rowsPerPage);
      const target = nodeByRow(targetRow);
      if (target) applySelection(target);
      return;
    }

    if (event.key === 'PageUp') {
      event.preventDefault();
      const rowsPerPage = Math.max(1, Math.floor(height / ROW_HEIGHT));
      const selectedNode = nodeById(effectiveSelectedId);
      const targetRow = Math.max(0, (selectedNode?.row ?? 0) - rowsPerPage);
      const target = nodeByRow(targetRow);
      if (target) applySelection(target);
      return;
    }

    if (event.key === 'Enter') {
      event.preventDefault();
      if (effectiveSelectedId) {
        onSelectCommit?.(effectiveSelectedId);
      }
    }
  }

  // Setup canvas context, resize observer, and theme media query listener
  $effect(() => {
    if (!container || !canvas) return;

    ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) return;

    // Resolve theme from computed styles — untrack to avoid read/write loop
    const currentTheme = untrack(() => theme);
    if (!currentTheme) {
      theme = resolveTheme(container);
    }

    resizeObserver?.disconnect();
    resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) return;

      const box = entry.contentRect;
      width = Math.floor(box.width);
      height = Math.floor(box.height);
      onHeightChange?.(height);
      setupCanvasSize();
      queueRender();
    });

    resizeObserver.observe(container);

    const onWindowResize = () => {
      setupCanvasSize();
      queueRender();
    };

    window.addEventListener('resize', onWindowResize);

    // Re-resolve theme when system color scheme changes (dark/light toggle)
    const colorSchemeQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const onThemeChange = () => {
      if (container) {
        theme = resolveTheme(container);
      }
    };
    colorSchemeQuery.addEventListener('change', onThemeChange);

    return () => {
      window.removeEventListener('resize', onWindowResize);
      colorSchemeQuery.removeEventListener('change', onThemeChange);
      resizeObserver?.disconnect();
      resizeObserver = null;

      if (rafId !== 0) {
        cancelAnimationFrame(rafId);
        rafId = 0;
      }
    };
  });

  // Re-render when layout data, theme, or controlled selection changes.
  // scrollTop/hoveredId are handled by their event handlers.
  $effect(() => {
    layout;
    theme;
    effectiveSelectedId;
    queueRender();
  });
</script>

<div
  class="graph-container"
  bind:this={container}
  role="application"
  aria-label="Commit graph — use arrow keys to navigate, Enter to select"
  tabindex="0"
  onscroll={handleScroll}
  onkeydown={handleKeydown}
>
  <canvas
    bind:this={canvas}
    class="graph-canvas"
    onmousemove={handleMouseMove}
    onmouseleave={handleMouseLeave}
    onclick={handleClick}
    oncontextmenu={handleContextMenu}
  ></canvas>
  <div class="graph-spacer" style:height="{totalHeight}px"></div>

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
