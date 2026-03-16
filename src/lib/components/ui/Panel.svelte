<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    elevation?: 0 | 1 | 2 | 3 | 4;
    padding?: boolean;
    header?: Snippet;
    children: Snippet;
  }

  let {
    elevation = 0,
    padding = true,
    header,
    children,
  }: Props = $props();
</script>

<div
  class="panel"
  class:elevation-1={elevation === 1}
  class:elevation-2={elevation === 2}
  class:elevation-3={elevation === 3}
  class:elevation-4={elevation === 4}
>
  {#if header}
    <div class="panel-header">
      {@render header()}
    </div>
  {/if}
  <div class="panel-body" class:padded={padding}>
    {@render children()}
  </div>
</div>

<style>
  .panel {
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .panel.elevation-1 {
    box-shadow: var(--elevation-1);
  }

  .panel.elevation-2 {
    box-shadow: var(--elevation-2);
  }

  .panel.elevation-3 {
    box-shadow: var(--elevation-3);
  }

  .panel.elevation-4 {
    box-shadow: var(--elevation-4);
  }

  .panel-header {
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--color-border-subtle);
    font-size: var(--text-heading-sm-size);
    font-weight: var(--text-heading-sm-weight);
    background: transparent;
  }

  .panel-body.padded {
    padding: var(--space-5);
  }
</style>
