# PRD: Graph-detail binding in main app shell

**Bead:** bd-145.1
**Parent:** bd-145 (Commit Graph Productization)
**Depends on:** bd-145 (parent-child)
**Type:** task
**Priority:** P0

```yaml
requirements_score:
  total: 92
  breakdown:
    business_value: 26
    functional_requirements: 24
    user_experience: 18
    technical_constraints: 14
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

mongit already has a validated Canvas commit-graph spike and a production repo shell, but they are still disconnected. The graph currently lives in `src/routes/spike-b/+page.svelte` as a validation surface, while the main repo shell at `src/routes/repo/+layout.svelte` and `src/routes/repo/+page.svelte` only shows orientation summary information. That means commit selection is not yet part of the real product workflow.

This child bead closes that gap by binding graph selection to a dependable commit-detail surface inside the repo shell. Users should be able to enter a dedicated history workspace, select a commit from the graph, inspect its details, and navigate between related commits without leaving the main app shell. Without this slice, the graph remains a technical spike instead of a usable history surface, and downstream graph work such as `bd-145.2` has no stable product route to extend.

### Why now?

- `bd-12d` validated the graph renderer and interaction baseline, so the remaining work is product integration rather than re-spiking.
- `bd-6ew` created the repo shell needed to host a durable history workspace.
- `bd-145.2` is blocked by this bead, so the graph needs a stable shell-level route and selection contract first.

### Who is affected?

- **Primary users:** solo power developers using mongit to inspect repository history from the main app workflow
- **Secondary users:** future implementation work under `bd-145`, `bd-1mk`, and `bd-1x9` that depends on a stable history route and selection model

## Scope

### In-Scope

- Add a dedicated nested history route inside the repo shell instead of reusing the spike route
- Load graph data for the active repository using the existing `get_commit_log` and `get_refs` Tauri commands
- Bind graph selection to commit-detail display so graph clicks, keyboard navigation, and detail-panel parent navigation stay in sync
- Keep commit selection predictable when data reloads or the active repository changes
- Preserve the existing `/repo` summary landing page and add a clear handoff into the history workspace
- Provide loading, empty, and error states that make the history workspace trustworthy in daily use

### Out-of-Scope

- Ref overlay readability improvements for dense real histories (`bd-145.2`)
- New graph rendering algorithms, lane layout redesign, or large-scale performance retuning already validated by Spike B
- Commit mutation actions such as checkout, cherry-pick, revert, reset, or branch creation from the detail surface
- File history, blame, compare-commit workflows, or broader history investigation tooling (`bd-1mk`)
- Replacing the repo shell header, redesigning the repo landing summary, or moving the feature back to `/spike-b`
- Automatic watcher-driven refresh orchestration beyond what is needed to keep selection state safe and non-stale

## Proposed Solution

Introduce a production history workspace under the repo shell using a dedicated nested route, recommended as `/repo/history`.

1. **Shell integration**
   - Keep `src/routes/repo/+page.svelte` as the orientation summary added by `bd-6ew.2`.
   - Add a route-level handoff from the summary into a history workspace under the same shell.
   - Reuse `src/routes/repo/+layout.svelte` so users stay inside repo context while moving between summary and history.

2. **History workspace state**
   - Build the history route state around the same request-guard patterns already used by `src/lib/stores/repo.svelte.ts` and `src/lib/stores/diff.svelte.ts`.
   - Fetch commit log and refs for the active repository using existing backend commands; no new Rust IPC contract is required for this slice.
   - Treat selected commit identity as route-owned product state rather than spike-local component state.

3. **Graph-detail binding**
   - Reuse `GraphCanvas.svelte` and `CommitDetail.svelte` instead of replacing them.
   - Extend the binding contract so detail-driven navigation can update the graph highlight and the graph can remain the source of row interaction.
   - Ensure the selected commit is cleared or remapped safely when the repo changes or a refresh no longer contains that commit.

### User Flow

1. User opens a repository and lands on `/repo` summary.
2. User enters the nested history workspace from the repo shell.
3. The workspace loads commit log and refs for the active repo and renders the graph with an adjacent detail panel.
4. User selects a commit from the graph and sees matching detail content immediately.
5. User navigates to a parent commit from the detail panel, and the graph highlight/detail panel remain synchronized.
6. If loading fails or no graph data is available, the route shows a clear state instead of an empty shell or stale details.

## Requirements

### Functional Requirements

#### R1. Dedicated history workspace route inside repo shell

The commit graph must move from spike-only usage into a production route inside the main repo shell.

**Scenarios:**
- **WHEN** a repository is active **THEN** users can enter a nested history workspace without leaving the repo shell context.
- **WHEN** the history workspace is introduced **THEN** the existing `/repo` summary landing page remains available and is not overwritten.
- **WHEN** downstream graph work lands **THEN** it extends the same history route instead of creating a competing shell surface.

#### R2. Dependable graph-to-detail selection binding

The history workspace must keep commit selection and detail presentation synchronized.

**Scenarios:**
- **WHEN** the user clicks or keyboard-selects a commit in the graph **THEN** the detail panel shows the matching commit.
- **WHEN** the user navigates to a parent commit from the detail panel **THEN** the active graph selection updates to the same commit instead of leaving the graph highlight stale.
- **WHEN** the user clears selection or no commit is selected **THEN** the detail panel shows a stable empty state instead of stale commit data.

#### R3. Active-repo graph loading using existing backend contracts

The history workspace must load real commit history for the currently active repository without inventing a new backend API.

**Scenarios:**
- **WHEN** the history route loads with an active repository **THEN** it fetches commit log and refs using the current Tauri commands.
- **WHEN** graph data loads successfully **THEN** the route renders a production graph/detail layout using the existing graph foundation.
- **WHEN** the active repository changes or becomes unavailable **THEN** the route invalidates stale graph state safely.

#### R4. Trustworthy loading, empty, and error states

The history workspace must make its state obvious during load failures or absent data.

**Scenarios:**
- **WHEN** commit history is loading **THEN** users see an explicit loading state rather than a blank panel.
- **WHEN** no graph data is available **THEN** users see a clear empty state describing what to do next.
- **WHEN** fetching history fails **THEN** the route shows an actionable error state and does not leave stale commit details visible.

#### R5. Stable selection lifecycle across refresh and repo changes

The selected commit must stay valid or fail safe as data changes.

**Scenarios:**
- **WHEN** graph data reloads and the selected commit still exists **THEN** the selection persists.
- **WHEN** graph data reloads and the selected commit no longer exists **THEN** the selection is cleared or predictably reassigned without pointing at missing data.
- **WHEN** the user opens a different repository **THEN** the history workspace does not leak the previous repository's selection or details.

### Non-Functional Requirements

- **Performance:** The history workspace should reuse the validated Spike B graph pipeline and avoid introducing extra full-layout work beyond normal graph loading for the active repo.
- **Security:** Only graph data for `repoStore.activeRepoPath` may be requested; invalid repo paths must surface structured command errors instead of crashing the route.
- **Accessibility:** Graph interaction must remain keyboard reachable, and the detail surface must expose meaningful empty/loading/error states.
- **Compatibility:** Must fit the existing Tauri 2.0 + SvelteKit repo shell, reuse current `get_commit_log` / `get_refs` commands, and preserve the open-per-call git2 pattern.
- **Reuse:** The route and selection contract must be extendable by `bd-145.2` and later history-investigation work instead of forcing a second shell integration.

## Success Criteria

- [ ] Users can navigate from `/repo` summary into a dedicated history route inside the repo shell.
  - Verify: manual app check in `pnpm tauri dev`
- [ ] Selecting a commit from the graph updates the detail panel, and parent navigation from the detail panel updates the active graph selection.
  - Verify: manual app check in `pnpm tauri dev`
- [ ] Loading, empty, and error states are visually distinct and never leave stale commit detail content onscreen.
  - Verify: manual app check with valid repo, invalid repo, and minimal-history repo
- [ ] Selection survives safe refreshes and clears safely on repo change or missing commit.
  - Verify: focused frontend state coverage where available plus manual repo-switch validation
- [ ] Project verification remains green after implementation.
  - Verify: `pnpm check`
  - Verify: `pnpm build`
  - Verify: `cargo check` (run in `src-tauri/`)

## Technical Context

### Existing Patterns

- `src/routes/spike-b/+page.svelte:72` - existing real-repo graph loading pattern using `get_commit_log` and `get_refs`
- `src/routes/spike-b/+page.svelte:146` - spike-level selection pattern where the page owns selected commit state
- `src/routes/spike-b/+page.svelte:158` - detail-panel parent navigation already routes through a parent handler
- `src/routes/repo/+layout.svelte:16` - repo shell guard and long-lived shell container already exist
- `src/routes/repo/+page.svelte:6` - current repo landing summary must remain intact and hand off to history rather than be replaced
- `src/lib/graph/GraphCanvas.svelte:38` - current graph selection is local state, which is the main binding limitation this bead must fix
- `src/lib/graph/CommitDetail.svelte:171` - detail panel already exposes parent-commit navigation callback that can feed a controlled selection model
- `src/lib/stores/repo.svelte.ts:39` - repo store uses request-id guards to prevent stale async responses
- `src/lib/stores/diff.svelte.ts:44` - diff store shows the preferred pattern for preserving or clearing selection during repo refresh

### Key Constraints

- `bd-145` is still draft-only, so this child PRD must anchor decisions in current code evidence rather than a finalized parent PRD.
- `bd-145.2` depends on this bead, so the history route and selection model must be stable enough for ref-overlay improvements to attach later.
- `src/routes/spike-b/+page.svelte` is a validation route, not the long-term product shell.
- `GraphCanvas.svelte` currently owns `selectedId` internally, which prevents detail-driven navigation from reliably controlling graph highlight state.
- Existing backend commands `get_commit_log` and `get_refs` already provide the graph data needed for this slice, so new Rust commands should not be the default answer.
- Memory search/read tooling was unavailable in this session (`FTS5 not available`, SQLite database unavailable), so this PRD relies on bead artifacts, git history, and direct code inspection.

### Affected Files

```yaml
files:
  - src/routes/repo/+page.svelte # Preserve summary landing while adding a clear entry into the history workspace
  - src/routes/repo/history/+page.svelte # New production history route inside the repo shell
  - src/lib/graph/GraphCanvas.svelte # Externalize or control selected commit state so detail navigation stays synchronized
  - src/lib/graph/CommitDetail.svelte # Reuse existing callback contract; minor adaptation only if route integration requires it
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| The history surface replaces the `/repo` landing summary and undoes `bd-6ew.2` product structure | Medium | High | Keep the graph on a dedicated nested route and preserve `/repo` as the orientation page |
| Graph highlight and detail panel drift out of sync because selection stays local to `GraphCanvas` | High | High | Make selection controlled at the route level and reuse callbacks in both directions |
| Repo switches or reloads leave stale commit details visible | Medium | High | Reuse request-id and selection-validity guards from existing stores |
| Scope creeps into ref-overlay redesign or advanced history tooling | Medium | Medium | Limit this bead to route integration plus dependable selection/detail binding; defer overlays to `bd-145.2` |
| Implementation adds unnecessary backend work despite existing commands | Low | Medium | Treat current commit-log and refs IPC as the default contract unless a verified blocker is found |

## Open Questions

None. Route placement was resolved during refinement: the graph-detail surface should live on a dedicated nested route inside the repo shell rather than replacing `/repo`.

## Tasks

### Repo-shell history route handoff [ui]

Expose a dedicated history route from the existing `/repo` summary without replacing the summary landing page.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src/routes/repo/+page.svelte
  - src/routes/repo/history/+page.svelte
```

**Verification:**

- `pnpm check`
- Manual app check confirms `/repo` still shows the summary landing surface
- Manual app check confirms users can enter the history workspace from the repo shell

### History graph loading and state lifecycle [frontend]

Load graph data for the active repository with request-guarded route state that handles loading, error, empty, and repo-change invalidation predictably.

**Metadata:**

```yaml
depends_on: ["Repo-shell history route handoff"]
parallel: false
conflicts_with: []
files:
  - src/routes/repo/history/+page.svelte
```

**Verification:**

- `pnpm check`
- History route loads commit log and refs for the active repository using existing Tauri commands
- Invalid repo or load failure shows an error state without stale graph/detail content

### Controlled graph-detail selection binding [ui]

Make commit selection a dependable route-level contract so graph clicks, keyboard navigation, and detail-panel parent navigation all stay synchronized.

**Metadata:**

```yaml
depends_on: ["History graph loading and state lifecycle"]
parallel: false
conflicts_with: []
files:
  - src/routes/repo/history/+page.svelte
  - src/lib/graph/GraphCanvas.svelte
  - src/lib/graph/CommitDetail.svelte
```

**Verification:**

- `pnpm check`
- Manual app check confirms graph selection updates commit details
- Manual app check confirms parent navigation from the detail panel updates the active graph selection
- Selection persists when the same commit remains present after refresh and clears safely when it does not

### Product-grade history route verification [integration]

Leave the history workspace ready for `bd-145.2` by proving the route, selection contract, and shell placement are stable.

**Metadata:**

```yaml
depends_on: ["Controlled graph-detail selection binding"]
parallel: true
conflicts_with: []
files:
  - src/routes/repo/history/+page.svelte
  - src/lib/graph/GraphCanvas.svelte
```

**Verification:**

- `pnpm check`
- `pnpm build`
- `cargo check`
- Manual app check confirms the nested history route remains within repo shell context and is ready for overlay work in `bd-145.2`

---

## Notes

- This PRD intentionally builds on the validated Spike B graph modules rather than re-spiking rendering or layout.
- User decision captured during refinement: the graph-detail surface should use a dedicated nested route inside the repo shell.
- No relevant `TODO` or `FIXME` markers were found in the targeted graph, route, or store files during refinement.
