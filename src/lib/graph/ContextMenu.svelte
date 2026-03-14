<script lang="ts">
  import type { CommitNode } from './types';

  interface Props {
    node: CommitNode;
    x: number;
    y: number;
    onClose: () => void;
    onAction: (action: string, node: CommitNode) => void;
  }

  let { node, x, y, onClose, onAction }: Props = $props();

  let menuEl = $state<HTMLDivElement | null>(null);

  function handleAction(action: string): void {
    onAction(action, node);
    onClose();
  }

  $effect(() => {
    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) return;
      if (menuEl?.contains(target)) return;
      onClose();
    };

    const handleKeydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
      }
    };

    window.addEventListener('pointerdown', handlePointerDown);
    window.addEventListener('keydown', handleKeydown);

    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<div
  bind:this={menuEl}
  class="context-menu"
  style:left="{x}px"
  style:top="{y}px"
  role="menu"
  tabindex="-1"
>
  <button type="button" class="menu-item" onclick={() => handleAction('copy-hash')}>Copy Commit Hash</button>
  <button type="button" class="menu-item" onclick={() => handleAction('copy-message')}>Copy Commit Message</button>
  <button type="button" class="menu-item" onclick={() => handleAction('browse')}>Browse Files at Commit</button>
  <button type="button" class="menu-item" onclick={() => handleAction('create-branch')}>Create Branch Here...</button>
  <button type="button" class="menu-item" onclick={() => handleAction('cherry-pick')}>Cherry-pick</button>
  <div class="menu-separator"></div>
  <button type="button" class="menu-item danger" onclick={() => handleAction('reset')}>
    Reset Current Branch to Here...
  </button>
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: var(--z-dropdown);
    min-width: 220px;
    padding: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-elevated);
    box-shadow: var(--elevation-2);
  }

  .menu-item {
    display: block;
    width: 100%;
    padding: var(--space-3) var(--space-4);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-primary);
    font: inherit;
    text-align: left;
    cursor: default;
  }

  .menu-item:hover {
    background: var(--color-bg-hover);
  }

  .menu-item.danger {
    color: var(--color-danger);
  }

  .menu-item.danger:hover {
    background: var(--color-danger-muted);
  }

  .menu-separator {
    height: 1px;
    margin: var(--space-2) var(--space-1);
    background: var(--color-border-subtle);
  }
</style>
