# PRD: Spike D — Diff Viewer and File Watching Backbone

**Bead:** bd-htm
**Parent:** bd-134
**Type:** feature
**Priority:** P0

```yaml
requirements_score:
  total: 93
  breakdown:
    business_value: 27
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

mongit needs a trustworthy foundation for repository-change awareness before staging, conflict resolution, and ambient status can feel product-grade. The current codebase already has partial pieces — a Rust watcher, a watcher monitor UI, a CodeMirror diff shell, and a git2 diff model — but they are not yet wired into a real repo-backed flow. This spike should validate that real repository changes can be detected, translated into targeted UI updates, and rendered through a reusable diff surface without blunt full-app reload behavior.

## Scope

### In-Scope

- Expose real worktree diff data from Rust to the frontend
- Use a **real changed-file list** in the spike surface
- Allow selecting a changed file and rendering its diff
- Keep watcher lifecycle stable when starting, stopping, or switching repos
- Preserve and validate debounce behavior around filesystem activity
- Bridge backend watcher events to frontend state updates
- Refresh only affected diff/status surfaces, not the entire app
- Use `/spike-d` as the validation surface for this foundation
- Validate **500ms end-to-end** watcher-to-UI update target

### Out-of-Scope

- Line-level or hunk-level staging actions
- Conflict resolution workflow productization
- Full home/workspace shell integration
- Multi-repo watching
- Rich per-file event payload design beyond what this spike needs
- Full production changes workspace UX (bd-20d)
- Full conflict workflow UX (bd-3gr)

## Proposed Solution

Build on the existing Spike D scaffold instead of replacing it:

1. Add a Tauri command that exposes `Git2Repository::diff_workdir()` to the frontend.
2. Extend the spike UI to show the repo's changed files, select one, and render its diff with the existing `DiffViewer`.
3. Keep the Rust watcher as the source of coarse repo-change signals, using its current filtered/debounced behavior.
4. Connect `repo-changed` events to a targeted frontend refresh path that re-fetches diff/status data for the active repo without invalidating the whole application.
5. Keep the architecture reusable for later adoption by local changes and conflict workflows.

## Requirements

### R1: Real Repo Diff Retrieval

The frontend can request the current worktree diff for a repository using a Tauri IPC command backed by `Git2Repository::diff_workdir()`.

**Acceptance:**
- Returns changed files from a real repo, not sample data
- Includes file path, file status, hunks, and hunk lines
- Invalid repo paths return structured errors, not panics

**Affected files:** `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/git/repository.rs`

### R2: Real Changed File List in Spike UI

The `/spike-d` diff surface shows a real list of changed files for the selected repository.

**Acceptance:**
- Changed files are listed from live repo data
- User can select a file from the list
- Selection updates the rendered diff panel
- Empty state is shown when no changed files exist

**Affected files:** `src/routes/spike-d/+page.svelte`, `src/lib/components/DiffViewer.svelte`

### R3: Repo-Backed Diff Rendering

`DiffViewer` renders repo-backed diff content rather than only hardcoded sample text.

**Acceptance:**
- Selected file diff renders in CodeMirror MergeView
- Added/removed/modified files are handled predictably
- Loading and error states are visible
- Large unchanged regions remain collapsible

**Affected files:** `src/lib/components/DiffViewer.svelte`

### R4: Stable Watcher Lifecycle

Watcher start/stop/replacement behavior remains correct when the active repo changes.

**Acceptance:**
- Starting a watcher on a repo works for valid paths
- Starting a new watcher replaces the previous watcher
- Stopping the watcher halts new events
- Filter rules still suppress noisy paths and allow important `.git` changes

**Affected files:** `src-tauri/src/watcher.rs`, `src/lib/stores/watcher.svelte.ts`

### R5: Targeted UI Refresh Path

Watcher events trigger scoped refresh behavior instead of blunt app-wide reloads.

**Acceptance:**
- `repo-changed` causes diff/status refresh for the active repo surface only
- The app does not use full-page or full-app invalidation as the default response
- Refresh logic is understandable and localized in a store or route-level controller

**Affected files:** `src/lib/stores/watcher.svelte.ts`, `src/routes/spike-d/+page.svelte`, optional: `src/lib/stores/diff.svelte.ts`

### R6: Performance and Trustworthiness

File changes should surface in the UI within a practical threshold so the system feels trustworthy.

**Acceptance:**
- End-to-end file-change to UI-refresh target: **<= 500ms**
- Debounced edits do not spam redundant refreshes
- Rapid edits batch predictably rather than causing unstable UI churn

### R7: Reusable Foundation for Downstream Work

The spike should leave behind a shape that downstream work can adopt.

**Acceptance:**
- Diff retrieval path is reusable by later changes workspace work
- Watcher event flow is reusable by later status/changes/conflict surfaces
- No design choices in the spike block bd-20d or bd-3gr

## Success Criteria

1. User can open `/spike-d`, point it at a repo, and see a **real list of changed files**
2. Selecting a file renders a **real repo-backed diff**
3. Modifying files in the repo emits watcher updates that refresh the visible surface within **500ms end-to-end**
4. Watcher lifecycle remains stable when starting, stopping, and switching repos
5. Refresh behavior is **scoped**, not implemented as blunt full-app reloads
6. Verification passes:
   - `pnpm check` — 0 errors
   - `cargo check` — succeeds
   - relevant Rust tests for watcher/diff logic pass

## Verify

```bash
pnpm check    # 0 errors (svelte-check)
cargo check   # Rust typecheck succeeds (in src-tauri/)
cargo test watcher --lib      # Watcher filter tests
cargo test diff_workdir --lib # Diff retrieval tests
```

Manual verification:
- Open `/spike-d`
- Start watching a repo with modified files
- Confirm changed-file list loads from the repo
- Select a file and confirm real diff content renders
- Edit a tracked file externally
- Confirm update reaches the UI within ~500ms
- Stop watching and confirm no further UI refreshes occur

## Technical Context

### Relevant existing code

- `src-tauri/src/watcher.rs:17-112` — watcher filter, debounce, event emission
- `src/lib/stores/watcher.svelte.ts:9-69` — existing Svelte watcher store
- `src/lib/components/WatcherMonitor.svelte:18-22` — event listener pattern
- `src-tauri/src/git/repository.rs:16-51` — diff data structures
- `src-tauri/src/git/repository.rs:195-259` — `diff_workdir()` implementation
- `src/lib/components/DiffViewer.svelte:170-190` — current MergeView usage
- `src/routes/spike-d/+page.svelte:68-83` — current spike dashboard tabs
- `src-tauri/src/commands.rs:27-152` — IPC command patterns
- `src-tauri/src/lib.rs:13-26` — command registration

### Constraints

- SvelteKit runs with `ssr=false`; browser-only APIs are acceptable with guards
- `git2::Repository` is not `Send + Sync`; open-per-call pattern must remain
- Watcher debounce is already **300ms**
- `repo-changed` currently carries no payload
- Frontend currently lacks a formal test harness in `package.json`
- Scope must remain foundation-only, not full staging/conflict productization

## Affected Files

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/commands.rs` | Modify | Add diff IPC wrapper |
| `src-tauri/src/lib.rs` | Modify | Register new command |
| `src-tauri/src/watcher.rs` | Modify or preserve | Keep lifecycle/filter behavior; possibly minor refinement |
| `src/lib/stores/watcher.svelte.ts` | Modify | Coordinate active repo and targeted refresh |
| `src/lib/components/DiffViewer.svelte` | Modify | Replace sample-only flow with repo-backed rendering |
| `src/routes/spike-d/+page.svelte` | Modify | Add real changed-file list and selection workflow |
| `src/lib/stores/diff.svelte.ts` | Optional new | Centralize diff loading/selection state if needed |

## Tasks

### Task 1: Diff IPC and data contract [backend]

Expose `diff_workdir()` through Tauri.

**Verification:**
- `cargo check` passes
- Rust tests around diff retrieval pass
- Invalid path returns structured error, not panic

**Metadata:**
```yaml
depends_on: []
parallel: true
files:
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
```

### Task 2: Watcher lifecycle hardening [backend]

Preserve watcher correctness under repo switches and event bursts.

**Verification:**
- Watcher tests pass
- Replacement and stop behavior work
- Filtered paths still behave correctly

**Metadata:**
```yaml
depends_on: []
parallel: true
files:
  - src-tauri/src/watcher.rs
  - src/lib/stores/watcher.svelte.ts
```

### Task 3: Real changed-file list + diff selection [frontend]

Load real diff entries, render file list, and show selected diff.

**Verification:**
- `pnpm check` passes
- Empty/loading/error states present
- Selected file updates diff view

**Metadata:**
```yaml
depends_on: [backend]
parallel: false
files:
  - src/routes/spike-d/+page.svelte
  - src/lib/components/DiffViewer.svelte
```

### Task 4: Targeted refresh bridge [integration]

Connect `repo-changed` to scoped diff/status reload.

**Verification:**
- External edit refreshes visible diff without full-app reload
- End-to-end update stays within ~500ms

**Metadata:**
```yaml
depends_on: [backend, frontend]
parallel: false
files:
  - src/lib/stores/watcher.svelte.ts
  - src/routes/spike-d/+page.svelte
```

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `repo-changed` has no payload, causing over-refresh | Medium | Keep refresh scoped to active repo/diff surface |
| Diff structure may need translation for UI rendering | Medium | Add thin frontend mapping layer rather than changing git model first |
| Rapid file saves may create unstable UI churn | Medium | Respect debounce and add frontend refresh coalescing if needed |
| Scope creep into staging/conflicts | High | Keep spike limited to foundation/backbone validation |
| Large diffs may stress CodeMirror | Medium | Validate on spike surface before adopting broadly |

## Open Questions

None — all questions resolved during refinement.

---

## Metadata

**Parent:** bd-134
**Children:** bd-htm.1, bd-htm.2, bd-htm.3
