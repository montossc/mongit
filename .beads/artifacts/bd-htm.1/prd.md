# PRD: CodeMirror diff integration shell

**Bead:** bd-htm.1  
**Created:** 2026-03-17  
**Status:** Approved

## Bead Metadata

```yaml
depends_on: []
parallel: false
conflicts_with:
  - bd-htm.3
blocks:
  - bd-htm.3
estimated_hours: 3
requirements_score:
  total: 92
  breakdown:
    business_value: 27
    functional_requirements: 23
    user_experience: 18
    technical_constraints: 14
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

### What problem are we solving?

mongit already has a promising `DiffViewer` spike, but it is still sample-data driven and does not yet define a trustworthy reusable shell for later changes and conflict workflows. Without a stable shell that can render caller-provided diff content and recover gracefully from empty, loading, and render-failure conditions, downstream work will either duplicate UI state handling or couple rendering too tightly to repo-loading and watcher logic.

### Why now?

Spike D is already split into focused child beads. `bd-htm.1` needs to lock in the frontend diff shell boundary now so `bd-htm.3` can wire refresh behavior against a predictable component contract instead of inventing one during integration.

### Who is affected?

- **Primary users:** solo power developers using mongit’s future local-changes and conflict surfaces
- **Secondary users:** future implementation beads (`bd-20d`, `bd-3gr`, `bd-htm.3`) that need a reusable diff rendering primitive

---

## Scope

### In-Scope

- Define `DiffViewer` as a reusable **frontend rendering shell** for diff content
- Render caller-provided diff text and file metadata through CodeMirror MergeView
- Show explicit loading, empty, and error-recovery UI states
- Preserve stable lifecycle behavior when inputs change or the component unmounts
- Use `/spike-d` as the validation surface for the shell states and contract

### Out-of-Scope

- Fetching real diff data from Tauri commands
- Building the changed-file list or repo-backed selection workflow
- Listening to `repo-changed` or implementing targeted refresh behavior
- Watcher lifecycle, debounce tuning, or Rust event payload changes
- Line-level/hunk-level staging controls
- Conflict resolution actions beyond rendering-side shell reuse

---

## Proposed Solution

### Overview

Refactor the current sample-oriented `DiffViewer` into a caller-owned rendering component with explicit state inputs. The component should remain responsible for CodeMirror MergeView creation, destruction, and visual fallback states, while callers such as `/spike-d` own when data is loading, empty, ready, or failed. This keeps `bd-htm.1` strictly on the reusable shell boundary and avoids overlap with `bd-htm.3` refresh wiring.

### User Flow (if user-facing)

1. Developer opens `/spike-d` and views the Diff tab.
2. The page supplies one of four shell states: loading, empty, ready, or error.
3. `DiffViewer` renders the matching UI without crashing and, for ready state, mounts a read-only MergeView using the provided filename and diff text.

---

## Requirements

### Functional Requirements

#### R1: Explicit diff shell contract

`DiffViewer` must accept caller-owned state and content rather than owning repo fetching or watcher subscriptions.

**Scenarios:**

- **WHEN** a caller provides filename, original text, and modified text **THEN** the shell renders a read-only MergeView for that file
- **WHEN** a caller changes those props **THEN** the shell updates to reflect the new diff without leaving stale content behind
- **WHEN** no content is available yet **THEN** the caller can place the shell in loading or empty mode without requiring fake sample code

#### R2: Graceful fallback states

The shell must provide first-class loading, empty, and error states so downstream repo-loading work does not need to reinvent them.

**Scenarios:**

- **WHEN** the caller marks the shell as loading **THEN** the UI shows a loading state instead of mounting an incomplete MergeView
- **WHEN** the caller provides no diffable content **THEN** the UI shows an empty-state message explaining there is nothing to render
- **WHEN** render setup fails or the caller reports a parse/load failure **THEN** the shell shows an error panel and the surrounding page remains usable

#### R3: Stable CodeMirror lifecycle

The shell must manage MergeView instances safely so repeated prop changes do not leak editors or produce inconsistent UI.

**Scenarios:**

- **WHEN** diff props change after mount **THEN** any previous MergeView instance is destroyed before a new one is created
- **WHEN** the component unmounts **THEN** the MergeView instance is destroyed cleanly
- **WHEN** unchanged regions are large **THEN** collapse settings remain enabled so the shell stays practical for real diffs

#### R4: Spike validation surface

`/spike-d` must exercise the shell contract clearly enough that later integration work can rely on it.

**Scenarios:**

- **WHEN** `/spike-d` is opened on the Diff tab **THEN** the page can demonstrate loading, empty, error, and ready states for the shell
- **WHEN** the ready state is shown **THEN** the rendered content uses the same component contract intended for future repo-backed callers

### Non-Functional Requirements

- **Performance:** shell state transitions should feel immediate; MergeView recreation should avoid obvious flicker for typical file-sized diffs
- **Security:** no shell state may expose filesystem paths or backend errors beyond already user-visible repo context
- **Accessibility:** fallback states must remain readable and keyboard-accessible within the existing spike surface
- **Compatibility:** stay aligned with current CodeMirror setup in `src/lib/utils/codemirror-config.ts` and theme tokens in `src/lib/utils/codemirror-theme.ts`

---

## Success Criteria

- [ ] `DiffViewer` no longer depends on hardcoded sample content as its primary operating mode
  - Verify: inspect `src/lib/components/DiffViewer.svelte` and confirm the component is driven by explicit caller inputs/state, not embedded sample-only flow
- [ ] Loading, empty, ready, and error states are all visibly represented in the shell
  - Verify: open `/spike-d`, switch through the shell states, and confirm each state renders without console/runtime failure
- [ ] Ready-state rendering still uses CodeMirror MergeView with read-only behavior and collapsed unchanged sections
  - Verify: open `/spike-d`, view the ready state, and confirm side-by-side diff rendering with unchanged-region collapsing
- [ ] The shell boundary does not absorb watcher or repo-refresh responsibilities
  - Verify: inspect affected files and confirm there is no `repo-changed` listener or Tauri diff-fetch logic added under this bead’s scope
- [ ] Frontend verification passes
  - Verify: `pnpm check`

---

## Technical Context

### Existing Patterns

- `src/lib/components/DiffViewer.svelte:170-190` - existing MergeView creation pattern with read-only extensions and unchanged-region collapse
- `src/lib/actions/codemirror.ts:13-42` - Svelte lifecycle pattern returning `update()` and `destroy()` for CodeMirror-backed UI
- `src/lib/utils/codemirror-config.ts:9-30` - shared CodeMirror extension composition and language detection by filename
- `src/lib/utils/codemirror-theme.ts:9-47` - theme tokens for editor and diff coloration
- `src/routes/spike-d/+page.svelte:68-83` - current spike tab that can host validation for the shell contract

### Key Files

- `src/lib/components/DiffViewer.svelte` - core reusable shell to refactor
- `src/routes/spike-d/+page.svelte` - spike validation surface for shell states
- `src/lib/utils/codemirror-config.ts` - existing extension composition to preserve
- `src/lib/utils/codemirror-theme.ts` - existing theme tokens to preserve

### Affected Files

Files this bead will modify (for conflict detection):

```yaml
files:
  - src/lib/components/DiffViewer.svelte # Convert sample-driven spike into caller-owned diff shell with fallback states
  - src/routes/spike-d/+page.svelte # Exercise and validate shell states without adding refresh wiring
```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| ---- | ---------- | ------ | ---------- |
| Scope drifts into repo loading or watcher refresh work owned by `bd-htm.3` | Medium | High | Keep shell API caller-owned; reject `repo-changed` and Tauri-fetch logic in this bead |
| MergeView recreation leaks instances during repeated prop changes | Medium | High | Require explicit destroy-before-create lifecycle verification |
| Fallback states become spike-only hacks instead of reusable shell behavior | Medium | Medium | Define the shell contract in the component API, not only in route-local conditionals |
| Hardcoded sample content remains the default path and hides integration gaps | Medium | Medium | Success criteria require caller-driven state and no sample-only operating mode |

---

## Open Questions

| Question | Owner | Due Date | Status |
| -------- | ----- | -------- | ------ |
| None | — | — | Resolved |

---

## Tasks

Write tasks in a machine-convertible format for `prd-task` skill.

### Normalize DiffViewer shell API [frontend]

Refactor `DiffViewer` so callers provide its rendering state and diff content while the component owns only MergeView lifecycle and fallback presentation.

**Metadata:**

```yaml
depends_on: []
parallel: false
conflicts_with: []
files:
  - src/lib/components/DiffViewer.svelte
```

**Verification:**

- `pnpm check`
- Inspect `src/lib/components/DiffViewer.svelte` and confirm the component exposes explicit loading/empty/error/ready handling without repo-fetch logic
- Open `/spike-d` and confirm the ready state renders a read-only MergeView for caller-provided content

### Add spike shell-state harness [frontend]

Update `/spike-d` so the Diff tab demonstrates loading, empty, error, and ready shell states using the new `DiffViewer` contract.

**Metadata:**

```yaml
depends_on:
  - Normalize DiffViewer shell API
parallel: false
conflicts_with:
  - bd-htm.3
files:
  - src/routes/spike-d/+page.svelte
```

**Verification:**

- `pnpm check`
- Open `/spike-d` and verify each shell state is reachable and visually distinct
- Confirm no `repo-changed` listener or Tauri diff fetch was introduced in the route for this bead

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

- User clarification resolved the main ambiguity: `bd-htm.1` is **frontend shell only**.
- Repo-backed diff fetching belongs to the broader Spike D foundation and may be handled by downstream work, but `bd-htm.1` must not absorb watcher-event refresh logic from `bd-htm.3`.
- The component contract should align with existing Rust diff structures conceptually, but this bead does not own the Tauri transport layer.
