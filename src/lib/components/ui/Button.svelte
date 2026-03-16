<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'compact' | 'default' | 'prominent';
    disabled?: boolean;
    type?: 'button' | 'submit' | 'reset';
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
  }

  let {
    variant = 'secondary',
    size = 'default',
    disabled = false,
    type = 'button',
    onclick,
    children,
  }: Props = $props();
</script>

<button
  class="btn {variant} {size}"
  {type}
  {disabled}
  {onclick}
>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);

    height: var(--size-button-default);
    padding: var(--space-4) var(--space-5);
    border: none;
    border-radius: var(--radius-sm);

    font-family: var(--font-sans);
    font-size: var(--text-body-sm-size);
    font-weight: 500;
    white-space: nowrap;

    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast), border-color var(--transition-fast);
  }

  .btn:focus-visible {
    outline: var(--focus-ring-width) solid var(--focus-ring-color);
    outline-offset: var(--focus-ring-offset);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.primary {
    background: var(--color-accent);
    color: white;
  }

  .btn.primary:hover:not(:disabled) {
    background: var(--color-accent-hover);
  }

  .btn.secondary {
    background: var(--color-bg-elevated);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }

  .btn.secondary:hover:not(:disabled) {
    background: var(--color-bg-active);
  }

  .btn.ghost {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .btn.ghost:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .btn.danger {
    background: var(--color-danger);
    color: white;
  }

  .btn.danger:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn.compact {
    height: var(--size-button-compact);
    padding: var(--space-3) var(--space-4);
  }

  .btn.default {
    height: var(--size-button-default);
    padding: var(--space-4) var(--space-5);
  }

  .btn.prominent {
    height: var(--size-button-prominent);
    padding: 10px var(--space-6);
  }
</style>
