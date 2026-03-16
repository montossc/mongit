<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'branch' | 'tag' | 'remote' | 'default';
    children: Snippet;
  }

  let { variant = 'default', children }: Props = $props();
</script>

<span class="badge {variant}">
  {@render children()}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    height: var(--size-badge);
    padding: var(--space-1) var(--space-3);
    font-size: var(--text-caption-size);
    font-weight: 500;
    border-radius: var(--radius-full);
    white-space: nowrap;
    line-height: 1;
    background: var(--color-bg-elevated);
    color: var(--color-text-secondary);
  }

  .badge.branch {
    background: var(--color-accent-muted);
    color: var(--color-accent);
  }

  .badge.tag {
    background: var(--color-warning-muted);
    color: var(--color-warning);
  }

  .badge.remote {
    background: color-mix(in srgb, var(--color-info) 15%, transparent);
    color: var(--color-info);
  }

  .badge.default {
    background: var(--color-bg-elevated);
    color: var(--color-text-secondary);
  }
</style>
