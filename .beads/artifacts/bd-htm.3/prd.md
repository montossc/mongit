# PRD: Tauri event bridge and targeted UI refresh path

**Bead:** bd-htm.3  
**Created:** 2026-03-17  
**Status:** Approved

## Bead Metadata

```yaml
depends_on:
  - bd-htm.1
  - bd-htm.2
parallel: false
conflicts_with: []
blocks: []
estimated_hours: 3
requirements_score:
  total: 92
  breakdown:
    business_value: 27
    functional_requirements: 24
    user_experience: 17
    technical_constraints: 14
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

### What problem are we solving?

mongit now has a reusable diff shell (`bd-htm.1`) and a hardened backend watcher contract (`bd-htm.2`), but repository-change events still stop at diagnostics instead of refreshing a real product surface. Without a clear frontend bridge from `repo-changed` into route-local state, users see watcher activity without any visible graph/status update, and downstream work risks falling back to blunt app-wide reload patterns instead of scoped refresh behavior.

### Why now?

Spike D split responsibilities deliberately: `bd-htm.1` locked the diff shell boundary and `bd-htm.2` locked the backend watcher boundary. `bd-htm.3` now needs to prove that the existing coarse watcher events can drive a **targeted** refresh path on the main repo graph route before broader changes-workspace work builds on that assumption.

### Who is affected?

- **Primary users:** solo power developers using mongit’s main repo view who expect repository changes to surface without manual full reloads
- **Secondary users:** downstream beads such as `bd-20d` and `bd-3gr` that need a trustworthy scoped refresh pattern rather than app-wide invalidation

---

## Scope

### In-Scope

- Bridge backend `repo-changed` events into the **main commit graph route** (`src/routes/+page.svelte`)
- Refresh only the active repo graph/status surface when the route is showing a real repo
- Add route-level guards so refresh logic is understandable and does not run in synthetic-data mode or when Tauri is unavailable
- Keep watcher diagnostics usable for manual validation without turning `/spike-d` into the primary refresh target
- Validate that the route refreshes in response to watcher events without using full-page or full-app invalidation

### Out-of-Scope

- Refreshing `/spike-d` as the primary target for this bead
- Changing watcher lifecycle, debounce, filtering, or backend event payloads owned by `bd-htm.2`
- Fetching worktree diffs or building changed-file selection workflows
- Full-app invalidation, global reload buses, or cross-route refresh orchestration beyond the main graph route
- Staging actions, conflict actions, or broader workspace-shell integration

---

## Proposed Solution

### Overview

Use a **route-local controller** in `src/routes/+page.svelte` as the targeted refresh bridge. The route already owns repo loading (`loadRepo()`), Tauri availability checks, loading state, and current `repoPath`, so it is the smallest and clearest place to subscribe to `repo-changed`. The route should listen for watcher events only while mounted, ignore events when synthetic data or non-Tauri mode is active, and re-run its existing repo load flow for the currently open repo path. This keeps the refresh path localized, avoids introducing a global invalidation pattern, and preserves `/spike-d` as a diagnostic/manual validation surface rather than a core dependency.

### User Flow (if user-facing)

1. Developer opens the main repo graph route and loads a real repository.
2. Developer starts watching the same repo from the watcher monitor diagnostic surface.
3. When repository files change, the main graph route refreshes its repo-backed state without a full app reload.

---

## Requirements

### Functional Requirements

#### R1: Route-local event bridge

The main repo route must subscribe to backend watcher events and connect them to its existing repo-loading controller.

**Scenarios:**

- **WHEN** the main route is mounted in Tauri mode **THEN** it can subscribe to `repo-changed` and cleanly unsubscribe on teardown
- **WHEN** `repo-changed` arrives while a real repo is loaded on the main route **THEN** the route re-runs its repo-backed load path for the current repo
- **WHEN** the route is not showing a real repo path **THEN** watcher events do not trigger a repo refresh

#### R2: Targeted refresh only

Watcher-driven refresh behavior must stay scoped to the main graph/status surface rather than becoming a blunt global invalidation mechanism.

**Scenarios:**

- **WHEN** repository changes occur **THEN** only the active main repo route refresh logic runs, not a full-page reload or app-wide invalidation
- **WHEN** `/spike-d` is used only for watcher diagnostics **THEN** it does not become the primary refresh target for this bead
- **WHEN** `bd-htm.1`’s diff shell is not yet wired to real diff data **THEN** this bead does not force that integration as part of the refresh path

#### R3: Refresh guards and stability

The route-level bridge must avoid noisy or invalid refresh attempts.

**Scenarios:**

- **WHEN** the route is in synthetic-data mode **THEN** watcher events do not overwrite synthetic state with repo-backed fetches
- **WHEN** Tauri IPC is unavailable **THEN** the event bridge stays inactive and does not produce runtime errors
- **WHEN** repeated watcher events arrive during an existing load **THEN** refresh behavior remains understandable and does not degrade into uncontrolled churn

#### R4: Manual validation path

The spike should leave behind a clear manual verification flow proving the bridge works end-to-end.

**Scenarios:**

- **WHEN** a developer loads a repo on the main route, starts a watcher, and edits a tracked file **THEN** the graph route visibly refreshes without a manual Open click
- **WHEN** the watcher is stopped **THEN** subsequent edits no longer trigger graph refreshes

### Non-Functional Requirements

- **Performance:** watcher-triggered route refresh should remain compatible with the parent spike target of roughly **<= 500ms end-to-end** from file change to visible UI update
- **Security:** the event bridge must not expose new backend details beyond the existing coarse event contract and route error handling
- **Accessibility:** route refresh should preserve the existing graph route interaction model without introducing inaccessible auto-refresh controls
- **Compatibility:** use the existing `@tauri-apps/api/event` listener pattern already proven in `WatcherMonitor.svelte`; do not require backend payload changes from `bd-htm.2`

---

## Success Criteria

- [ ] The main repo route subscribes to `repo-changed` and refreshes repo-backed state for the current repo
  - Verify: inspect `src/routes/+page.svelte` and confirm the route listens for watcher events and reuses `loadRepo()` or an equivalent route-local refresh path
- [ ] Refresh behavior stays scoped to the active graph/status surface rather than using full-app invalidation
  - Verify: inspect affected files and confirm there is no `invalidateAll`, full-page reload, or global reload bus added for this bead
- [ ] Synthetic mode and non-Tauri mode remain guarded from watcher-driven refresh
  - Verify: inspect `src/routes/+page.svelte` and confirm event handling is gated by current route/Tauri/repo state
- [ ] Manual spike validation proves end-to-end refresh works
  - Verify: open the main route with a real repo, start watching it via the watcher monitor, edit a tracked file, and confirm the graph route refreshes without pressing Open again
- [ ] Frontend verification passes
  - Verify: `pnpm check`

---

## Technical Context

### Existing Patterns

- `src/lib/components/WatcherMonitor.svelte:18-26` - current `listen<void>('repo-changed', ...)` pattern with explicit cleanup
- `src/routes/+page.svelte:44-71` - existing `loadRepo()` controller already owns repo-backed refresh logic
- `src/routes/+page.svelte:27-42` - existing route lifecycle and Tauri detection guards
- `src/lib/stores/watcher.svelte.ts:16-36` - watcher start/stop store remains diagnostic-oriented and should not become a global refresh orchestrator by default
- `.beads/artifacts/bd-htm.1/prd.md:61-66` - `bd-htm.1` explicitly excludes `repo-changed` and targeted refresh behavior
- `.beads/artifacts/bd-htm.2/prd.md:61-65` - `bd-htm.2` explicitly excludes frontend listener/store refresh ownership

### Key Files

- `src/routes/+page.svelte` - primary target for route-local event bridge and scoped refresh behavior
- `src/lib/components/WatcherMonitor.svelte` - existing watcher diagnostic surface used for manual verification
- `src/lib/stores/watcher.svelte.ts` - existing watcher diagnostic store; should stay narrow unless a tiny helper is clearly justified
- `src/routes/spike-d/+page.svelte` - reference-only diagnostic surface, not the primary refresh target for this bead

### Affected Files

Files this bead will modify (for conflict detection):

```yaml
files:
  - src/routes/+page.svelte # Add route-local repo-changed listener and targeted refresh guards
  - src/lib/stores/watcher.svelte.ts # Optional small helper only if needed for watcher state reuse or cleanup coordination
```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| ---- | ---------- | ------ | ---------- |
| Scope drifts into global invalidation or app-wide refresh behavior | Medium | High | Keep the bridge route-local and reuse existing `loadRepo()` controller rather than introducing a global bus |
| Refresh fires while synthetic mode is active and clobbers demo state | Medium | Medium | Require explicit guards for Tauri mode, real repo path presence, and current route state |
| Repeated watcher events trigger overlapping loads and confusing UI churn | Medium | High | Keep refresh behavior localized and add clear route-level guard/coalescing behavior |
| bd-htm.3 accidentally reabsorbs work owned by `bd-htm.1` or `bd-htm.2` | Medium | High | Preserve shell-only and backend-only boundaries in success criteria and affected-file scope |

---

## Open Questions

| Question | Owner | Due Date | Status |
| -------- | ----- | -------- | ------ |
| None | — | — | Resolved |

---

## Tasks

Write tasks in a machine-convertible format for `prd-task` skill.

### Add main-route event bridge [frontend]

Wire `repo-changed` into the main repo route so the route refreshes its repo-backed graph/status state for the currently open repository.

**Metadata:**

```yaml
depends_on: []
parallel: false
conflicts_with: []
files:
  - src/routes/+page.svelte
```

**Verification:**

- `pnpm check`
- Inspect `src/routes/+page.svelte` and confirm the route subscribes to `repo-changed` with cleanup and reuses its existing repo load controller
- Manually load a real repo, start watching it, edit a tracked file, and confirm the graph route refreshes without pressing Open again

### Add scoped refresh guards [frontend]

Harden the main-route refresh path so watcher-driven updates are ignored in synthetic or non-Tauri mode and do not fall back to blunt global invalidation.

**Metadata:**

```yaml
depends_on:
  - Add main-route event bridge
parallel: false
conflicts_with: []
files:
  - src/routes/+page.svelte
  - src/lib/stores/watcher.svelte.ts
```

**Verification:**

- `pnpm check`
- Inspect affected files and confirm there is no `invalidateAll`, full-page reload, or spike-d-first refresh path introduced
- Confirm synthetic-data mode remains usable without watcher events forcing repo-backed reloads

---

## Dependency Legend

| Field | Purpose | Example |
| ----- | ------- | ------- |
| `depends_on` | Must complete before this task starts | `["Setup database", "Create schema"]` |
| `parallel` | Can run concurrently with other parallel tasks | `true` / `false` |
| `conflicts_with` | Cannot run in parallel (same files) | `["Update config"]` |
| `files` | Files this task modifies (for conflict detection) | `["src/db/schema.ts", "src/db/client.ts"]` |

---

## Notes

- User clarification resolved the main ambiguity: `bd-htm.3` should target the **main commit graph route first**.
- `bd-htm.1` remains the diff shell boundary; `bd-htm.2` remains the backend watcher boundary.
- This bead should prove scoped refresh behavior, not broaden the spike into global invalidation architecture.
