<script lang="ts">
  import type { LayoutResult } from './types';
  import {
    ROW_HEIGHT,
    renderGraph,
    resolveTheme,
    type GraphTheme
  } from './render';

  interface Props {
    layout: LayoutResult | null;
    onSelectCommit?: (id: string) => void;
  }

  let { layout, onSelectCommit }: Props = $props();

  let container = $state<HTMLDivElement | null>(null);
  let canvas = $state<HTMLCanvasElement | null>(null);
  let ctx = $state<CanvasRenderingContext2D | null>(null);

  let scrollTop = $state(0);
  let selectedId = $state<string | null>(null);
  let hoveredId = $state<string | null>(null);

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

  function handleMouseMove(event: MouseEvent): void {
    hoveredId = nodeAtClientY(event.clientY);
    queueRender();
  }

  function handleMouseLeave(): void {
    hoveredId = null;
    queueRender();
  }

  function handleClick(event: MouseEvent): void {
    const id = nodeAtClientY(event.clientY);
    if (!id) return;

    selectedId = id;
    onSelectCommit?.(id);
    queueRender();
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
  onscroll={handleScroll}
>
  <div class="graph-spacer" style:height="{totalHeight}px"></div>
  <canvas
    bind:this={canvas}
    class="graph-canvas"
    onmousemove={handleMouseMove}
    onmouseleave={handleMouseLeave}
    onclick={handleClick}
  ></canvas>
</div>

<style>
  .graph-container {
    position: relative;
    overflow-y: auto;
    height: 100%;
    background: var(--color-bg);
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
