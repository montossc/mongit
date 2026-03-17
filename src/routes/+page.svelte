<main class="landing-shell">
	<section class="landing-card" aria-labelledby="landing-title">
		<h1 id="landing-title" class="app-title">mongit</h1>
		<p class="subtitle">Repository workspace (coming soon)</p>
		<a class="spike-link" href="/spike-b">Open Spike B — Graph Validation</a>
	</section>
</main>

<style>
	.landing-shell {
		display: grid;
		place-items: center;
		min-height: 100vh;
		padding: var(--space-6);
		background: var(--color-bg);
		color: var(--color-text-primary);
	}

	.landing-card {
		display: grid;
		gap: var(--space-4);
		padding: var(--space-6);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		background: var(--color-bg-surface);
		max-width: 420px;
		width: 100%;
	}

	.app-title {
		margin: 0;
		font-family: var(--font-display);
		font-size: 24px;
		font-weight: 700;
		color: var(--color-accent);
	}

	.subtitle {
		margin: 0;
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--color-text-secondary);
	}

	.spike-link {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--color-accent);
		text-decoration: none;
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		width: fit-content;
	}

	.spike-link:hover,
	.spike-link:focus-visible {
		background: var(--color-bg-hover);
		border-color: var(--color-accent);
		outline: none;
	}
</style>
