<script lang="ts">
  import { ROW_HEIGHT } from './render';
  import type { LayoutResult } from './types';

  interface Props {
    layout: LayoutResult | null;
    scrollTop?: number;
    canvasHeight?: number;
    visible?: boolean;
  }

  let { layout, scrollTop = 0, canvasHeight = 0, visible = false }: Props = $props();

  const FRAME_WINDOW_SIZE = 60;

  let rafId = $state<number | null>(null);
  let lastTimestamp = $state<number | null>(null);
  let frameDurations = $state<number[]>([]);

  let fps = $state(0);
  let frameTimeMs = $state(0);
  let minFrameTimeMs = $state(0);
  let maxFrameTimeMs = $state(0);

  const visibleRows = $derived(Math.ceil(Math.max(0, canvasHeight) / ROW_HEIGHT));
  const totalCommits = $derived(layout?.nodes.length ?? 0);
  const laneCount = $derived(layout?.laneCount ?? 0);
  const layoutTimeMs = $derived(layout?.layoutTimeMs ?? 0);

  const fpsHealth = $derived(fps >= 55 ? 'good' : fps >= 30 ? 'warn' : 'bad');

  function resetMeasurements(): void {
    lastTimestamp = null;
    frameDurations = [];
    fps = 0;
    frameTimeMs = 0;
    minFrameTimeMs = 0;
    maxFrameTimeMs = 0;
  }

  function stopLoop(): void {
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
  }

  function updateMetrics(): void {
    if (frameDurations.length === 0) {
      fps = 0;
      frameTimeMs = 0;
      minFrameTimeMs = 0;
      maxFrameTimeMs = 0;
      return;
    }

    let sum = 0;
    let min = Number.POSITIVE_INFINITY;
    let max = 0;

    for (const value of frameDurations) {
      sum += value;
      if (value < min) min = value;
      if (value > max) max = value;
    }

    const avg = sum / frameDurations.length;
    frameTimeMs = avg;
    fps = avg > 0 ? 1000 / avg : 0;
    minFrameTimeMs = min;
    maxFrameTimeMs = max;
  }

  function tick(timestamp: number): void {
    if (!visible) {
      stopLoop();
      return;
    }

    if (lastTimestamp !== null) {
      const delta = timestamp - lastTimestamp;
      if (delta > 0) {
        const next = [...frameDurations, delta];
        frameDurations =
          next.length > FRAME_WINDOW_SIZE ? next.slice(next.length - FRAME_WINDOW_SIZE) : next;
        updateMetrics();
      }
    }

    lastTimestamp = timestamp;
    rafId = requestAnimationFrame(tick);
  }

  function formatInt(value: number): string {
    return Math.round(value).toLocaleString();
  }

  function formatMs(value: number): string {
    return `${value.toLocaleString(undefined, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 1
    })} ms`;
  }

  function formatFps(value: number): string {
    return `${Math.round(value).toLocaleString()} fps`;
  }

  $effect(() => {
    if (!visible) {
      stopLoop();
      resetMeasurements();
      return;
    }

    stopLoop();
    resetMeasurements();
    rafId = requestAnimationFrame(tick);

    return () => {
      stopLoop();
      resetMeasurements();
    };
  });
</script>

{#if visible}
  <aside class="fps-overlay" aria-live="polite" aria-label="Performance metrics overlay">
    <div class="metric metric--fps">
      <span class="label">FPS</span>
      <span class="value value--fps">
        <span class="dot dot--{fpsHealth}" aria-hidden="true"></span>
        {formatFps(fps)}
      </span>
    </div>

    <div class="metric">
      <span class="label">Frame Time</span>
      <span class="value">{formatMs(frameTimeMs)}</span>
    </div>

    <div class="metric metric--sub">
      <span class="label">Min/Max</span>
      <span class="value">{formatMs(minFrameTimeMs)} / {formatMs(maxFrameTimeMs)}</span>
    </div>

    <div class="metric">
      <span class="label">Visible Rows</span>
      <span class="value">{formatInt(visibleRows)} rows</span>
    </div>

    <div class="metric">
      <span class="label">Total Commits</span>
      <span class="value">{formatInt(totalCommits)}</span>
    </div>

    <div class="metric">
      <span class="label">Lane Count</span>
      <span class="value">{formatInt(laneCount)} lanes</span>
    </div>

    <div class="metric">
      <span class="label">Layout Time</span>
      <span class="value">{formatMs(layoutTimeMs)}</span>
    </div>

    <!-- Keep prop usage explicit for future expansion -->
    <span class="sr-only">Scroll offset row {formatInt(Math.floor(scrollTop / ROW_HEIGHT))}</span>
  </aside>
{/if}

<style>
  .fps-overlay {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 20;
    display: grid;
    grid-template-columns: minmax(72px, auto) 1fr;
    gap: 4px 12px;
    width: min(220px, calc(100vw - 16px));
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-bg-elevated) 92%, transparent);
    box-shadow: var(--elevation-2);
    font-family: var(--font-mono);
    font-size: 11px;
    pointer-events: none;
  }

  .metric {
    display: contents;
  }

  .label {
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .value {
    color: var(--color-text-primary);
    text-align: right;
    white-space: nowrap;
  }

  .value--fps {
    display: inline-flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    display: inline-block;
    flex-shrink: 0;
  }

  .dot--good {
    background: var(--color-success);
  }

  .dot--warn {
    background: var(--color-warning);
  }

  .dot--bad {
    background: var(--color-danger);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    border: 0;
    white-space: nowrap;
  }
</style>
