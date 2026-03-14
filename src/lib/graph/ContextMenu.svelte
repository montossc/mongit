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
  let focusedIndex = $state(0);

  const items = [
    { action: 'copy-hash', label: 'Copy Commit Hash' },
    { action: 'copy-message', label: 'Copy Commit Message' },
    { action: 'show-terminal', label: 'Show in Terminal' },
  ] as const;

  function handleAction(action: string): void {
    onAction(action, node);
    onClose();
  }

  function focusItem(index: number): void {
    if (!menuEl) return;
    const buttons = menuEl.querySelectorAll<HTMLButtonElement>('button[role="menuitem"]');
    const target = buttons[index];
    if (target) {
      target.focus();
      focusedIndex = index;
    }
  }

  $effect(() => {
    // Focus first item on mount
    if (menuEl) {
      requestAnimationFrame(() => focusItem(0));
    }

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) return;
      if (menuEl?.contains(target)) return;
      onClose();
    };

    window.addEventListener('pointerdown', handlePointerDown);

    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
    };
  });

  function handleMenuKeydown(event: KeyboardEvent): void {
    const buttonCount = items.length;

    switch (event.key) {
      case 'Escape':
        event.preventDefault();
        event.stopPropagation();
        onClose();
        break;
      case 'ArrowDown':
        event.preventDefault();
        focusItem((focusedIndex + 1) % buttonCount);
        break;
      case 'ArrowUp':
        event.preventDefault();
        focusItem((focusedIndex - 1 + buttonCount) % buttonCount);
        break;
      case 'Home':
        event.preventDefault();
        focusItem(0);
        break;
      case 'End':
        event.preventDefault();
        focusItem(buttonCount - 1);
        break;
    }
  }
</script>

<div
  bind:this={menuEl}
  class="context-menu"
  style:left="{x}px"
  style:top="{y}px"
  role="menu"
  onkeydown={handleMenuKeydown}
>
  {#each items as item, idx}
    <button
      type="button"
      class="menu-item"
      role="menuitem"
      tabindex={idx === focusedIndex ? 0 : -1}
      onclick={() => handleAction(item.action)}
    >{item.label}</button>
  {/each}
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
