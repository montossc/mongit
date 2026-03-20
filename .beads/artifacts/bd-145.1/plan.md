# Graph-Detail Binding in Main App Shell — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use skill({ name: "executing-plans" }) to implement this plan task-by-task.

**Goal:** Bind the commit graph (from Spike B) to a detail panel inside the repo shell's new `/repo/history` route, with controlled selection, loading/error/empty states, and selection lifecycle management.

**Architecture:** A new SvelteKit route at `/repo/history` loads graph data via existing Tauri IPC, owns selection state, and passes it to a refactored `GraphCanvas` (now accepting controlled `selectedId` prop) and `CommitDetail`. The repo summary page gets a navigation link. Request-ID guards prevent stale data on repo changes.

**Tech Stack:** Svelte 5 (runes), SvelteKit (adapter-static, SSR disabled), Tauri 2.0 IPC, TypeScript

---

## Must-Haves

**Goal:** Users navigate from repo summary into a dedicated history workspace where graph selection and commit details stay synchronized.

### Observable Truths

1. User can navigate from `/repo` summary to `/repo/history` inside the repo shell
2. User sees commit graph loaded for the active repository with loading/empty/error states
3. Clicking/keyboard-selecting a commit in the graph updates the detail panel immediately
4. Clicking a parent link in the detail panel updates the graph highlight and detail panel
5. Selection clears safely on repo change or when selected commit no longer exists after refresh
6. `pnpm check`, `pnpm build`, and `cargo check` all pass

### Required Artifacts

| Artifact | Provides | Path |
|----------|----------|------|
| History route page | Graph + detail workspace inside repo shell | `src/routes/repo/history/+page.svelte` |
| Controlled GraphCanvas | External selection control via prop | `src/lib/graph/GraphCanvas.svelte` (modified) |
| Navigation link | Entry point from summary to history | `src/routes/repo/+page.svelte` (modified) |

### Key Links

| From | To | Via | Risk |
|------|-----|-----|------|
| History page | GraphCanvas | `selectedId` prop + `onSelectCommit` callback | Selection drift if prop isn't consumed correctly |
| History page | CommitDetail | `node` prop + `onNavigateToCommit` callback | Stale node if lookup misses in nodeMap |
| History page | Tauri backend | `invoke('get_commit_log')` + `invoke('get_refs')` | Race conditions on repo switch |
| Repo summary | History page | SvelteKit `goto('/repo/history')` | Route must exist in repo shell |

### Task Dependencies

```
Task 1 (GraphCanvas controlled selection): needs nothing, modifies src/lib/graph/GraphCanvas.svelte
Task 2 (History route skeleton): needs nothing, creates src/routes/repo/history/+page.svelte
Task 3 (Navigation link): needs Task 2, modifies src/routes/repo/+page.svelte
Task 4 (Graph data loading): needs Task 2, modifies src/routes/repo/history/+page.svelte
Task 5 (Selection binding): needs Tasks 1+4, modifies src/routes/repo/history/+page.svelte
Task 6 (Full verification): needs all

Wave 1: Task 1, Task 2 (parallel — different files)
Wave 2: Task 3, Task 4 (parallel — different files, both depend on Wave 1)
Wave 3: Task 5 (depends on Wave 2)
Wave 4: Task 6 (verification checkpoint)
```

---

## Wave 1 — Foundation (Parallel)

### Task 1: Refactor GraphCanvas for controlled selection

**Tier:** worker

**Files:**
- Modify: `src/lib/graph/GraphCanvas.svelte`

**Handoff Contract:**
- **Produces:** `GraphCanvas` component that accepts optional `selectedId` prop
- **Consumed By:** Task 5 (selection binding)

**Context:** Currently `selectedId` is internal state at line 38 (`let selectedId = $state<string | null>(null)`). The parent cannot read or set it, which means detail-panel parent navigation cannot update the graph highlight. We need to make it optionally controlled from outside while keeping backward compatibility for the spike-b page.

**Step 1: Add `selectedId` to Props interface**

In `src/lib/graph/GraphCanvas.svelte`, modify the Props interface (lines 13-21) to add the optional `selectedId` prop:

```typescript
interface Props {
  layout: LayoutResult | null;
  selectedId?: string | null;  // NEW — controlled selection from parent
  onSelectCommit?: (id: string) => void;
  onContextAction?: (action: string, node: CommitNode) => void;
  onScrollChange?: (scrollTop: number) => void;
  onHeightChange?: (height: number) => void;
  onHoverCommit?: (id: string | null) => void;
  onKeyInteraction?: (key: string) => void;
}
```

**Step 2: Destructure the new prop and rename internal state**

Update the props destructuring (lines 23-31) to capture `selectedId` as `controlledSelectedId`, and rename the internal `selectedId` state (line 38) to `internalSelectedId`:

```typescript
let {
  layout,
  selectedId: controlledSelectedId,
  onSelectCommit,
  onContextAction,
  onScrollChange,
  onHeightChange,
  onHoverCommit,
  onKeyInteraction
}: Props = $props();
```

Rename line 38 from:
```typescript
let selectedId = $state<string | null>(null);
```
to:
```typescript
let internalSelectedId = $state<string | null>(null);
```

**Step 3: Create derived effective selection**

Add a `$derived` expression right after the internal state declarations (after what was line 39):

```typescript
const effectiveSelectedId = $derived(
  controlledSelectedId !== undefined ? controlledSelectedId : internalSelectedId
);
```

This means:
- If parent passes `selectedId` prop → controlled mode (parent is source of truth)
- If no `selectedId` prop → uncontrolled mode (internal state, backward compatible with spike-b)

**Step 4: Update all reads of `selectedId` to use `effectiveSelectedId`**

Replace every **read** of the old `selectedId` with `effectiveSelectedId`. These are:

1. **Line 88** in `draw()` → `renderGraph(ctx, layout, { ... selectedId, ... })`:
   Change `selectedId` to `effectiveSelectedId`

2. **Line 207** in ArrowDown handler → `const selectedNode = nodeById(selectedId)`:
   Change to `const selectedNode = nodeById(effectiveSelectedId)`

3. **Line 216** in ArrowUp handler → `const selectedNode = nodeById(selectedId)`:
   Change to `const selectedNode = nodeById(effectiveSelectedId)`

4. **Line 247** in PageDown handler → `const selectedNode = nodeById(selectedId)`:
   Change to `const selectedNode = nodeById(effectiveSelectedId)`

5. **Line 257** in PageUp handler → `const selectedNode = nodeById(selectedId)`:
   Change to `const selectedNode = nodeById(effectiveSelectedId)`

6. **Line 266** in Enter handler → `if (selectedId)` and `onSelectCommit?.(selectedId)`:
   Change to `if (effectiveSelectedId)` and `onSelectCommit?.(effectiveSelectedId)`

**Step 5: Update all writes of `selectedId` to use `internalSelectedId`**

Replace every **write** to the old `selectedId` with `internalSelectedId`:

1. **Line 131** in `applySelection()` → `selectedId = node.data.id`:
   Change to `internalSelectedId = node.data.id`

2. **Line 199** in Escape handler → `selectedId = null`:
   Change to `internalSelectedId = null`

**Step 6: Run verification**

Run: `pnpm check`
Expected: 0 errors — the component API is backward compatible (new prop is optional)

**Step 7: Commit**

```bash
git add src/lib/graph/GraphCanvas.svelte
git commit -m "feat(bd-145.1): add controlled selectedId prop to GraphCanvas

- Add optional selectedId prop to Props interface
- Use effectiveSelectedId derived from controlled or internal state
- Backward compatible: spike-b page works unchanged (no prop = internal state)"
```

---

### Task 2: Create history route skeleton

**Tier:** worker

**Files:**
- Create: `src/routes/repo/history/+page.svelte`

**Handoff Contract:**
- **Produces:** Empty history page at `/repo/history` rendering inside repo shell
- **Consumed By:** Tasks 3, 4, 5

**Context:** The repo shell at `src/routes/repo/+layout.svelte` renders child routes via `{@render children()}` (line 40) inside a `.repo-content` div (line 39). Any page under `src/routes/repo/` automatically gets the repo shell toolbar, guard, and layout. The page must NOT replace the existing `/repo` summary (which stays at `src/routes/repo/+page.svelte`).

**Step 1: Create the history route directory and page**

Create `src/routes/repo/history/+page.svelte` with a minimal skeleton showing a placeholder message:

```svelte
<script lang="ts">
	import { repoStore } from '$lib/stores/repo.svelte';
</script>

<div class="history-workspace">
	{#if !repoStore.activeRepoPath}
		<div class="history-state">
			<p class="state-message">No repository loaded</p>
		</div>
	{:else}
		<div class="history-state">
			<p class="state-message">History workspace ready</p>
			<p class="state-hint">Graph loading will be connected next.</p>
		</div>
	{/if}
</div>

<style>
	.history-workspace {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.history-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-3);
		color: var(--color-text-muted);
	}

	.state-message {
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		margin: 0;
	}

	.state-hint {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		margin: 0;
	}
</style>
```

**Step 2: Run verification**

Run: `pnpm check`
Expected: 0 errors

**Step 3: Commit**

```bash
git add src/routes/repo/history/+page.svelte
git commit -m "feat(bd-145.1): create history route skeleton

- New route at /repo/history inside repo shell
- Renders placeholder with state-based messages
- Does not replace existing /repo summary page"
```

---

## Wave 2 — Loading & Navigation (Parallel)

### Task 3: Add navigation from repo summary to history

**Tier:** worker

**Files:**
- Modify: `src/routes/repo/+page.svelte`

**Handoff Contract:**
- **Produces:** Clickable "View History" button on repo summary that navigates to `/repo/history`
- **Consumed By:** End user (final UI)

**Context:** The repo summary at `src/routes/repo/+page.svelte` shows repo identity, branch, and working-tree stats. We need to add a navigation button below the state message (after line 64) that takes users to the history workspace. Use SvelteKit's `goto()` for navigation.

**Step 1: Add import and navigation button**

Add the `goto` import and a "View History" button. The button goes after the state message section, still inside `.summary`:

Add to the `<script>` block (after line 3):
```typescript
import { goto } from '$app/navigation';
```

After the closing `{/if}` on line 64 (end of the state message / fallback section), before the closing `</div>` of `.summary`, add:

```svelte
		<!-- History Entry -->
		<button class="history-btn" onclick={() => goto('/repo/history')}>
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="10" />
				<polyline points="12 6 12 12 16 14" />
			</svg>
			View History
		</button>
```

Add corresponding styles inside `<style>`:

```css
	/* ── History Entry ── */
	.history-btn {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-5);
		background: var(--color-accent);
		color: white;
		border: none;
		border-radius: var(--radius-sm);
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.history-btn:hover {
		filter: brightness(1.1);
	}
```

**Step 2: Run verification**

Run: `pnpm check`
Expected: 0 errors

**Step 3: Commit**

```bash
git add src/routes/repo/+page.svelte
git commit -m "feat(bd-145.1): add View History navigation to repo summary

- Add clock icon button linking to /repo/history
- Styled consistently with repo shell design tokens"
```

---

### Task 4: Add graph data loading with request-ID guards

**Tier:** worker

**Files:**
- Modify: `src/routes/repo/history/+page.svelte`

**Handoff Contract:**
- **Produces:** History page that loads commit log + refs for the active repo with loading/error/empty states
- **Consumed By:** Task 5 (selection binding)

**Context:** Follow the request-ID guard pattern from `src/lib/stores/repo.svelte.ts:39-93` and `src/lib/stores/diff.svelte.ts:44-90`. The pattern: increment a counter before each async operation, capture the counter value, check it hasn't changed before committing state. Use `invoke('get_commit_log', { path, max_count: 10000 })` and `invoke('get_refs', { path })` from `@tauri-apps/api/core`. Then call `assignLanes(commits, refs)` from `$lib/graph/layout` to produce a `LayoutResult`.

**Step 1: Replace the skeleton with full loading logic**

Replace the entire content of `src/routes/repo/history/+page.svelte` with:

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { repoStore } from '$lib/stores/repo.svelte';
	import type { CommitData, RefData, LayoutResult } from '$lib/graph/types';
	import { assignLanes } from '$lib/graph/layout';

	// ── State ──
	let layout = $state<LayoutResult | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	// ── Request-ID guard (prevents stale async responses) ──
	let loadRequestId = 0;

	// ── Track which repo path we loaded for ──
	let loadedRepoPath = $state<string | null>(null);

	async function loadGraphData(repoPath: string): Promise<void> {
		loadRequestId += 1;
		const thisRequest = loadRequestId;

		loading = true;
		error = null;

		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const [commits, refs] = await Promise.all([
				invoke<CommitData[]>('get_commit_log', { path: repoPath, max_count: 10000 }),
				invoke<RefData[]>('get_refs', { path: repoPath })
			]);

			// Stale check: bail if a newer load started
			if (thisRequest !== loadRequestId) return;

			if (commits.length === 0) {
				layout = null;
				loadedRepoPath = repoPath;
				loading = false;
				return;
			}

			layout = assignLanes(commits, refs);
			loadedRepoPath = repoPath;
		} catch (e) {
			// Only set error if still the current request
			if (thisRequest !== loadRequestId) return;
			error = e instanceof Error ? e.message : String(e);
			layout = null;
		} finally {
			if (thisRequest === loadRequestId) {
				loading = false;
			}
		}
	}

	// ── Reactive: reload when active repo changes ──
	$effect(() => {
		const repoPath = repoStore.activeRepoPath;
		if (repoPath && repoPath !== loadedRepoPath) {
			loadGraphData(repoPath);
		} else if (!repoPath) {
			// Repo closed — clear state
			layout = null;
			loadedRepoPath = null;
			error = null;
		}
	});

	// ── Initial load on mount ──
	onMount(() => {
		if (repoStore.activeRepoPath) {
			loadGraphData(repoStore.activeRepoPath);
		}
	});
</script>

<div class="history-workspace">
	{#if loading}
		<div class="history-state">
			<p class="state-message">Loading history…</p>
		</div>
	{:else if error}
		<div class="history-state history-state--error">
			<p class="state-message">Failed to load history</p>
			<p class="state-detail">{error}</p>
			<button class="retry-btn" onclick={() => {
				if (repoStore.activeRepoPath) loadGraphData(repoStore.activeRepoPath);
			}}>
				Retry
			</button>
		</div>
	{:else if !layout}
		<div class="history-state">
			<p class="state-message">No commits found</p>
			<p class="state-hint">This repository has no commit history yet.</p>
		</div>
	{:else}
		<div class="history-state">
			<p class="state-message">Graph loaded — {layout.nodes.length} commits</p>
			<p class="state-hint">Selection binding will be connected next.</p>
		</div>
	{/if}
</div>

<style>
	.history-workspace {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.history-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-3);
		color: var(--color-text-muted);
	}

	.history-state--error {
		color: var(--color-danger);
	}

	.state-message {
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		margin: 0;
	}

	.state-detail {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		color: var(--color-text-muted);
		margin: 0;
		max-width: 400px;
		text-align: center;
		word-break: break-word;
	}

	.state-hint {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		margin: 0;
	}

	.retry-btn {
		padding: var(--space-2) var(--space-4);
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-primary);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.retry-btn:hover {
		background: var(--color-bg-hover);
	}
</style>
```

**Step 2: Run verification**

Run: `pnpm check`
Expected: 0 errors

**Step 3: Commit**

```bash
git add src/routes/repo/history/+page.svelte
git commit -m "feat(bd-145.1): add graph data loading with request-ID guards

- Load commit log + refs via existing Tauri commands
- Request-ID guard prevents stale async responses
- Loading, error (with retry), and empty states
- Reactive reload on repo change"
```

---

## Wave 3 — Selection Binding

### Task 5: Wire graph-detail selection binding

**Tier:** worker

**Files:**
- Modify: `src/routes/repo/history/+page.svelte`

**Handoff Contract:**
- **Produces:** Complete history workspace with synchronized graph ↔ detail selection
- **Consumed By:** Task 6 (final verification), future work in bd-145.2

**Context:** The history page now loads graph data. This task adds `GraphCanvas` + `CommitDetail` rendering with:
1. Route-owned `selectedId` state (not internal to GraphCanvas)
2. `onSelectCommit` callback updates `selectedId` → CommitDetail re-renders
3. `onNavigateToCommit` callback updates `selectedId` → GraphCanvas re-renders via prop
4. Selection lifecycle: clears on repo change, persists on refresh if commit exists, clears if commit gone

**Step 1: Replace the full page with complete graph-detail workspace**

Replace the entire content of `src/routes/repo/history/+page.svelte` with the final implementation:

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { repoStore } from '$lib/stores/repo.svelte';
	import type { CommitData, CommitNode, RefData, LayoutResult } from '$lib/graph/types';
	import { assignLanes } from '$lib/graph/layout';
	import GraphCanvas from '$lib/graph/GraphCanvas.svelte';
	import CommitDetail from '$lib/graph/CommitDetail.svelte';

	// ── Graph state ──
	let layout = $state<LayoutResult | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);

	// ── Selection state (route-owned, passed to GraphCanvas as controlled prop) ──
	let selectedId = $state<string | null>(null);
	const selectedNode = $derived<CommitNode | null>(
		selectedId && layout ? layout.nodeMap.get(selectedId) ?? null : null
	);

	// ── Request-ID guard ──
	let loadRequestId = 0;
	let loadedRepoPath = $state<string | null>(null);

	async function loadGraphData(repoPath: string): Promise<void> {
		loadRequestId += 1;
		const thisRequest = loadRequestId;

		loading = true;
		error = null;

		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const [commits, refs] = await Promise.all([
				invoke<CommitData[]>('get_commit_log', { path: repoPath, max_count: 10000 }),
				invoke<RefData[]>('get_refs', { path: repoPath })
			]);

			// Stale check
			if (thisRequest !== loadRequestId) return;

			if (commits.length === 0) {
				layout = null;
				selectedId = null;
				loadedRepoPath = repoPath;
				loading = false;
				return;
			}

			const newLayout = assignLanes(commits, refs);

			// Selection lifecycle: preserve if commit still exists, clear if gone
			if (selectedId && !newLayout.nodeMap.has(selectedId)) {
				selectedId = null;
			}

			layout = newLayout;
			loadedRepoPath = repoPath;
		} catch (e) {
			if (thisRequest !== loadRequestId) return;
			error = e instanceof Error ? e.message : String(e);
			layout = null;
			selectedId = null;
		} finally {
			if (thisRequest === loadRequestId) {
				loading = false;
			}
		}
	}

	// ── Selection handlers ──
	function handleSelectCommit(id: string): void {
		selectedId = id;
	}

	function handleNavigateToCommit(commitId: string): void {
		if (!layout) return;
		// Only navigate if target commit exists in current layout
		if (layout.nodeMap.has(commitId)) {
			selectedId = commitId;
		}
	}

	// ── Reactive: reload on repo change, clear on repo close ──
	$effect(() => {
		const repoPath = repoStore.activeRepoPath;
		if (repoPath && repoPath !== loadedRepoPath) {
			// New repo — clear stale selection before loading
			selectedId = null;
			loadGraphData(repoPath);
		} else if (!repoPath) {
			layout = null;
			loadedRepoPath = null;
			selectedId = null;
			error = null;
		}
	});

	// ── Initial load ──
	onMount(() => {
		if (repoStore.activeRepoPath) {
			loadGraphData(repoStore.activeRepoPath);
		}
	});
</script>

<div class="history-workspace">
	{#if loading}
		<div class="history-state">
			<p class="state-message">Loading history…</p>
		</div>
	{:else if error}
		<div class="history-state history-state--error">
			<p class="state-message">Failed to load history</p>
			<p class="state-detail">{error}</p>
			<button class="retry-btn" onclick={() => {
				if (repoStore.activeRepoPath) loadGraphData(repoStore.activeRepoPath);
			}}>
				Retry
			</button>
		</div>
	{:else if !layout}
		<div class="history-state">
			<p class="state-message">No commits found</p>
			<p class="state-hint">This repository has no commit history yet.</p>
		</div>
	{:else}
		<div class="history-graph-area">
			<GraphCanvas
				{layout}
				{selectedId}
				onSelectCommit={handleSelectCommit}
			/>
		</div>
		<aside class="history-detail-panel">
			<CommitDetail
				node={selectedNode}
				onNavigateToCommit={handleNavigateToCommit}
			/>
		</aside>
	{/if}
</div>

<style>
	.history-workspace {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	/* ── Loading / Error / Empty states ── */
	.history-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--space-3);
		color: var(--color-text-muted);
	}

	.history-state--error {
		color: var(--color-danger);
	}

	.state-message {
		font-size: var(--text-body-sm-size);
		font-weight: 500;
		margin: 0;
	}

	.state-detail {
		font-family: var(--font-mono);
		font-size: var(--text-mono-sm-size);
		color: var(--color-text-muted);
		margin: 0;
		max-width: 400px;
		text-align: center;
		word-break: break-word;
	}

	.state-hint {
		font-size: var(--text-caption-size);
		color: var(--color-text-muted);
		margin: 0;
	}

	.retry-btn {
		padding: var(--space-2) var(--space-4);
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-primary);
		font-size: var(--text-body-sm-size);
		cursor: pointer;
		transition: background var(--transition-fast);
	}

	.retry-btn:hover {
		background: var(--color-bg-hover);
	}

	/* ── Graph + Detail layout ── */
	.history-graph-area {
		flex: 1;
		min-width: 0;
		overflow: hidden;
	}

	.history-detail-panel {
		width: 320px;
		flex-shrink: 0;
		border-left: 1px solid var(--color-border);
		overflow-y: auto;
		background: var(--color-bg-surface);
	}
</style>
```

**Step 2: Run verification**

Run: `pnpm check`
Expected: 0 errors

**Step 3: Commit**

```bash
git add src/routes/repo/history/+page.svelte
git commit -m "feat(bd-145.1): wire graph-detail selection binding

- Route owns selectedId state, passes to GraphCanvas as controlled prop
- Graph clicks and keyboard nav update selection via onSelectCommit
- Detail panel parent navigation updates selection via onNavigateToCommit
- Selection preserved on refresh if commit exists, cleared if missing
- Selection cleared on repo change
- Side-by-side layout: graph (flex) + detail panel (320px)"
```

---

## Wave 4 — Verification

### Task 6: Full verification pass

**Tier:** worker
**Type:** checkpoint:human-verify

**Files:**
- Read: `src/routes/repo/history/+page.svelte`
- Read: `src/lib/graph/GraphCanvas.svelte`
- Read: `src/routes/repo/+page.svelte`

**Step 1: Run frontend typecheck**

Run: `pnpm check`
Expected: 0 errors, 0 warnings that block build

**Step 2: Run frontend build**

Run: `pnpm build`
Expected: Build succeeds, no errors

**Step 3: Run Rust typecheck**

Run: `cd src-tauri && cargo check`
Expected: 0 errors (no Rust changes expected, but verify nothing broke)

**Step 4: Verify file structure**

Run: `find src/routes/repo -type f | sort`
Expected output should include:
```
src/routes/repo/+layout.svelte
src/routes/repo/+page.svelte
src/routes/repo/history/+page.svelte
```

**Step 5: Manual app verification checkpoint**

Describe to user what to verify in `pnpm tauri dev`:
1. Navigate to `/repo` — summary page still shows repo info, branch, stats
2. Click "View History" → navigates to `/repo/history`
3. History page loads commit graph for the active repo
4. Click a commit in graph → detail panel shows commit info
5. Click a parent hash in detail panel → graph highlight + detail panel update together
6. Press Escape → selection clears, detail panel shows empty state
7. Keyboard nav (ArrowUp/Down) → selection moves, detail panel follows

**Step 6: Final commit if any fixes needed**

```bash
git add -A  # Only if final fixes applied
git commit -m "fix(bd-145.1): address verification findings"
```

---

## Notes

- This plan intentionally skips modifying `CommitDetail.svelte` — its existing `onNavigateToCommit` callback API is sufficient.
- The spike-b page (`src/routes/spike-b/+page.svelte`) is NOT modified — it continues working in uncontrolled mode since no `selectedId` prop is passed.
- No new Rust commands are needed — existing `get_commit_log` and `get_refs` provide all required data.
- The 320px detail panel width matches the spike-b layout for visual consistency.
- The `$effect` for repo changes uses `loadedRepoPath` comparison to prevent re-fetching when the same repo is already loaded.
