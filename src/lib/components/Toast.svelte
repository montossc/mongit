<script lang="ts">
	import { toastStore, type Toast } from '$lib/stores/toast.svelte';

	function getIcon(variant: Toast['variant']): string {
		switch (variant) {
			case 'success': return '✓';
			case 'error': return '✕';
			case 'info': return 'ℹ';
		}
	}
</script>

{#if toastStore.toasts.length > 0}
	<div class="toast-container" role="status" aria-live="polite" aria-atomic="false">
		{#each toastStore.toasts as toast (toast.id)}
			<div class="toast toast-{toast.variant}" role="alert">
				<span class="toast-icon">{getIcon(toast.variant)}</span>
				<span class="toast-message">{toast.message}</span>
				<button
					class="toast-dismiss"
					onclick={() => toastStore.dismiss(toast.id)}
					aria-label="Dismiss notification"
				>✕</button>
			</div>
		{/each}
	</div>
{/if}

<style>
	.toast-container {
		position: fixed;
		bottom: var(--space-5);
		right: var(--space-5);
		z-index: var(--z-modal, 40);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		pointer-events: none;
		max-width: 400px;
	}

	.toast {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background: var(--color-bg-elevated);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md, 8px);
		box-shadow: var(--elevation-2);
		font-family: var(--font-sans);
		font-size: var(--text-body-sm-size);
		color: var(--color-text-primary);
		pointer-events: auto;
		animation: toast-slide-in 150ms ease-out;
	}

	.toast-error {
		border-color: var(--color-error, #E55353);
	}

	.toast-success {
		border-color: var(--color-success, #4CAF50);
	}

	.toast-info {
		border-color: var(--color-accent);
	}

	.toast-icon {
		flex-shrink: 0;
		width: 16px;
		text-align: center;
		font-weight: 700;
	}

	.toast-error .toast-icon {
		color: var(--color-error, #E55353);
	}

	.toast-success .toast-icon {
		color: var(--color-success, #4CAF50);
	}

	.toast-info .toast-icon {
		color: var(--color-accent);
	}

	.toast-message {
		flex: 1;
		min-width: 0;
		word-break: break-word;
	}

	.toast-dismiss {
		flex-shrink: 0;
		padding: 0;
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		font-size: 12px;
		line-height: 1;
		opacity: 0.6;
		transition: opacity var(--transition-fast);
	}

	.toast-dismiss:hover {
		opacity: 1;
	}

	@keyframes toast-slide-in {
		from {
			opacity: 0;
			transform: translateY(8px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
