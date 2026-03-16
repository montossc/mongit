<script lang="ts">
	interface Props {
		value?: string;
		placeholder?: string;
		type?: 'text' | 'number' | 'search' | 'password' | 'url';
		disabled?: boolean;
		mono?: boolean;
		oninput?: (e: Event) => void;
		onkeydown?: (e: KeyboardEvent) => void;
	}

	let {
		value = $bindable(''),
		placeholder = '',
		type = 'text',
		disabled = false,
		mono = false,
		oninput,
		onkeydown,
	}: Props = $props();
</script>

<input
	class="input"
	class:mono
	{type}
	{placeholder}
	{disabled}
	bind:value
	{oninput}
	{onkeydown}
/>

<style>
	.input {
		display: block;
		width: 100%;
		height: var(--size-input);
		padding: var(--space-2) var(--space-4);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		background: var(--color-bg-surface);
		font-family: var(--font-sans);
		font-size: var(--text-body-size);
		color: var(--color-text-primary);
		outline: none;
		transition:
			border-color var(--transition-fast),
			outline-color var(--transition-fast);
	}

	.input.mono {
		font-family: var(--font-mono);
	}

	.input::placeholder {
		color: var(--color-text-muted);
	}

	.input:focus {
		border-color: var(--color-accent);
		outline: var(--focus-ring-width) solid var(--focus-ring-color);
		outline-offset: var(--focus-ring-offset);
	}

	.input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
