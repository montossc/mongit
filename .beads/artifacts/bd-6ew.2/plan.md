# Home State Summary Widgets — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use skill({ name: "executing-plans" }) to implement this plan task-by-task.

**Goal:** Replace the placeholder `/repo` landing page with an orientation-focused summary surface that answers "Am I in the right repo?" within 1-2 seconds.

**Architecture:** Single-file Svelte component replacement. The existing `repoStore` already exposes `activeRepoName`, `activeRepoPath`, `repoStatus` (branch, changed_files, staged_files). No backend changes needed — purely frontend.

**Tech Stack:** Svelte 5 (runes), existing design tokens from `src/app.css`, existing `Panel` and `Badge` UI primitives.

---

## Must-Haves

**Goal:** User lands on `/repo` and instantly sees repo identity, branch context, and working-tree state.

### Observable Truths

1. User can identify which repo is open (name + path)
2. User can see current branch or explicit detached-HEAD state
3. User can see changed file count, staged file count, and a concise state message
4. Page handles null/missing status gracefully (no broken UI)
5. Page uses static-on-load data (no watcher subscriptions)

### Required Artifacts

| Artifact | Provides | Path |
|----------|----------|------|
| Summary page | Repo identity + branch + state widgets | `src/routes/repo/+page.svelte` |

### Key Links

| From | To | Via | Risk |
|------|-----|-----|------|
| +page.svelte | repoStore | `$lib/stores/repo.svelte` import | Low — already imported and working |
| repoStore | Backend | `get_repo_status` via `invoke()` | None — already hydrated on open |

### Task Dependencies

```
Task 1 (Summary page): needs nothing, modifies src/routes/repo/+page.svelte

Wave 1: Task 1 (single task, single wave)
```

---

## Task 1: Replace placeholder repo landing with orientation summary surface

**Files:**
- Modify: `src/routes/repo/+page.svelte` (full replacement of existing 99-line file)

**Step 1: Write the new summary page component**

Replace `src/routes/repo/+page.svelte` entirely. The new component should:

1. **Repo identity section**: Show `repoStore.activeRepoName` as a prominent heading, with `repoStore.activeRepoPath` below it in muted monospace text for disambiguation.

2. **Branch context**: Show `repoStore.repoStatus.branch` using the existing `Badge` component with `variant="branch"`. When branch is `null`, show an explicit "Detached HEAD" badge with a distinct visual treatment (warning color).

3. **Working-tree state cards**: Two stat cards side-by-side showing:
   - Changed files count (with label)
   - Staged files count (with label)

4. **State message**: A concise human-readable summary:
   - Both counts 0 → "Working tree clean" (success color)
   - Changed > 0, staged = 0 → "Unstaged changes" (warning color)
   - Staged > 0 → "Ready to commit" (accent color)

5. **Null status fallback**: When `repoStore.repoStatus` is null, show a calm "Loading status..." message instead of broken values.

Use these design tokens from `src/app.css`:
- Backgrounds: `--color-bg-surface`, `--color-bg-elevated`
- Text: `--color-text-primary`, `--color-text-secondary`, `--color-text-muted`
- Semantic: `--color-success`, `--color-warning`, `--color-accent`
- Spacing: `--space-*`
- Typography: `--text-heading-lg-*`, `--text-body-*`, `--text-mono-*`, `--text-caption-*`
- Radius: `--radius-md`

Layout approach:
- Vertically centered on the page (like current layout)
- Max-width ~480px for readability
- Generous spacing between sections
- Stat cards use the same `--color-bg-surface` + border pattern as existing status items

```svelte
<script lang="ts">
	import { repoStore } from '$lib/stores/repo.svelte';
	import { Badge } from '$lib/components/ui';
</script>

<div class="repo-landing">
	<div class="summary">
		<!-- Repo Identity -->
		<div class="identity">
			<h2 class="repo-name">{repoStore.activeRepoName ?? 'Unknown'}</h2>
			{#if repoStore.activeRepoPath}
				<span class="repo-path">{repoStore.activeRepoPath}</span>
			{/if}
		</div>

		<!-- Branch Context -->
		<div class="branch-context">
			{#if repoStore.repoStatus}
				{#if repoStore.repoStatus.branch}
					<Badge variant="branch">{repoStore.repoStatus.branch}</Badge>
				{:else}
					<span class="detached-badge">Detached HEAD</span>
				{/if}
			{:else}
				<span class="status-placeholder">...</span>
			{/if}
		</div>

		<!-- Working-Tree State -->
		{#if repoStore.repoStatus}
			<div class="state-cards">
				<div class="stat-card">
					<span class="stat-value">{repoStore.repoStatus.changed_files}</span>
					<span class="stat-label">Changed</span>
				</div>
				<div class="stat-card">
					<span class="stat-value">{repoStore.repoStatus.staged_files}</span>
					<span class="stat-label">Staged</span>
				</div>
			</div>

			<!-- State Message -->
			{@const changed = repoStore.repoStatus.changed_files}
			{@const staged = repoStore.repoStatus.staged_files}
			<p class="state-message"
				class:clean={changed === 0 && staged === 0}
				class:unstaged={changed > 0 && staged === 0}
				class:staged={staged > 0}
			>
				{#if changed === 0 && staged === 0}
					Working tree clean
				{:else if changed > 0 && staged === 0}
					Unstaged changes
				{:else if staged > 0}
					Ready to commit
				{/if}
			</p>
		{:else}
			<p class="fallback-message">Loading status…</p>
		{/if}
	</div>
</div>

<style>
	.repo-landing {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		padding: var(--space-8);
	}

	.summary {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-6);
		max-width: 480px;
		width: 100%;
		text-align: center;
	}

	/* ── Identity ── */
	.identity {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
	}

	.repo-name {
		font-family: var(--font-display);
		font-size: var(--text-heading-lg-size);
		font-weight: var(--text-heading-lg-weight);
		line-height: var(--text-heading-lg-leading);
		color: var(--color-text-primary);
		margin: 0;
	}

	.repo-path {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		color: var(--color-text-muted);
		word-break: break-all;
	}

	/* ── Branch ── */
	.branch-context {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.detached-badge {
		display: inline-flex;
		align-items: center;
		height: var(--size-badge);
		padding: var(--space-1) var(--space-3);
		font-size: var(--text-caption-size);
		font-weight: 500;
		border-radius: var(--radius-full);
		white-space: nowrap;
		line-height: 1;
		background: var(--color-warning-muted);
		color: var(--color-warning);
	}

	.status-placeholder {
		color: var(--color-text-muted);
		font-size: var(--text-body-sm-size);
	}

	/* ── State Cards ── */
	.state-cards {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-4);
		width: 100%;
		max-width: 280px;
	}

	.stat-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-5);
		background: var(--color-bg-surface);
		border-radius: var(--radius-md);
		border: 1px solid var(--color-border);
	}

	.stat-value {
		font-size: var(--text-heading-lg-size);
		font-weight: 700;
		color: var(--color-text-primary);
	}

	.stat-label {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	/* ── State Message ── */
	.state-message {
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		margin: 0;
	}

	.state-message.clean {
		color: var(--color-success);
	}

	.state-message.unstaged {
		color: var(--color-warning);
	}

	.state-message.staged {
		color: var(--color-accent);
	}

	.fallback-message {
		font-size: var(--text-body-sm-size);
		color: var(--color-text-muted);
		margin: 0;
	}
</style>
```

**Step 2: Run svelte-check**

Run: `pnpm check`
Expected: 0 errors, 0 warnings relevant to modified files.

**Step 3: Run build**

Run: `pnpm build`
Expected: Build succeeds with no errors.

**Step 4: Run cargo check (baseline)**

Run: `cd src-tauri && cargo check`
Expected: No errors (no Rust changes, baseline gate).

**Step 5: Commit**

```bash
git add src/routes/repo/+page.svelte
git commit -m "feat(bd-6ew.2): replace repo landing with orientation summary surface

- Show repo name + path for identity disambiguation
- Show branch badge or explicit Detached HEAD state
- Show changed/staged file counts in stat cards
- Add concise state message (clean/unstaged/ready to commit)
- Handle null repoStatus with graceful fallback
- Use existing design tokens and Badge component"
```
