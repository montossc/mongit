# PRD: Changed files model and workspace list

**Bead:** bd-20d.1
**Parent:** bd-20d (Changes Workspace and Partial Staging)
**Depends on:** bd-20d (parent-child)
**Type:** task
**Priority:** P0

```yaml
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

mongit now has a repo shell, orientation summary, and a proven spike for repo-backed diff retrieval, but it still lacks the production changes workspace that makes partial staging usable. Users need a dependable file list that shows the real state of their working tree, lets them identify what changed quickly, and establishes the canonical per-file model that downstream hunk and line staging will build on.

This child bead defines the first production slice of that workspace: the changed-files data contract and the file list UI. It should turn the earlier spike foundation into a product-grade list surface while stopping short of hunk mutation actions owned by `bd-20d.2` and line-level precision work owned by `bd-20d.3`.

## Scope

### In-Scope

- Establish the production changed-files data model used by the changes workspace
- Represent each file row with a **dual-state model** that can expose both staged and unstaged state together
- Render a production file list for changed files inside the repo workspace
- Show accurate per-file status for common Git states relevant to the working tree:
  - added
  - modified
  - deleted
  - renamed
  - staged-only
  - unstaged-only
  - partially staged
- Support selecting a file row so users can inspect the currently selected file in the workspace flow
- Provide empty, loading, and error states for the file list surface
- Reuse the existing diff/watcher foundation where it fits, without leaving the file list coupled to the spike route
- Create the route/component structure needed for downstream staging work to attach to the same workspace surface

### Out-of-Scope

- Hunk-level stage or unstage actions (`bd-20d.2`)
- Line-level staging or patch application (`bd-20d.3`)
- Rebuilding the repo shell or replacing the summary landing page from `bd-6ew.2`
- Full diff-viewer redesign beyond what is needed to support file selection handoff
- Conflict resolution workflows
- Commit authoring, branch actions, or command palette integration
- Multi-repo workspaces or multi-select batch operations

## Proposed Solution

Split this slice into a backend file-state contract and a frontend workspace list surface.

1. **Backend file-state contract**
   - Build on the existing `Git2Repository::diff_workdir()` and status primitives in `src-tauri/src/git/repository.rs`.
   - Extend or adapt the data model so each changed file can represent staged and unstaged state together in one row, rather than forcing the UI to infer partial staging from aggregate counts.
   - Keep the backend as the source of truth for file ordering and state classification.

2. **Frontend workspace state**
   - Build on the existing `diffStore` selection/loading pattern in `src/lib/stores/diff.svelte.ts`.
   - Separate “file list metadata” from “full diff content” so the file list stays responsive and downstream diff/staging work can lazy-load heavier payloads.
   - Preserve race guards and watcher-triggered refresh compatibility.

3. **Repo workspace UI**
   - Add a dedicated changes workspace route under the repo shell (recommended: `/repo/changes`) instead of replacing the repo landing summary.
   - Render a left-hand changed-files list using existing design-system primitives and status badges.
   - Keep selection behavior in place so downstream work can attach diff rendering and staging actions without changing the route model.

### User Flow

1. User opens a repo and navigates into the changes workspace from the repo shell.
2. The workspace loads changed-file rows from the active repository and shows each file’s combined staged/unstaged state.
3. User scans the list, selects a file, and the workspace records that selection for downstream diff and staging flows.
4. If the repo becomes clean or the refresh fails, the workspace shows a clear empty or error state instead of stale or misleading rows.

## Requirements

### Functional Requirements

#### R1. Production changed-files data model

The workspace must have a production data model for changed files that is suitable for later partial-staging work.

**Scenarios:**
- **WHEN** the active repo has changed files **THEN** the backend returns file rows with stable path identity and file-state metadata suitable for rendering a workspace list.
- **WHEN** a file has both staged and unstaged changes **THEN** the row exposes that combined state directly instead of forcing the frontend to infer it from repo-level counts.
- **WHEN** downstream staging work consumes this model **THEN** it can build on the same row contract without redefining file identity and state semantics.

#### R2. Accurate per-file workspace states

The list must represent repository state accurately enough for users to trust the workspace.

**Scenarios:**
- **WHEN** a file is modified, added, deleted, or renamed **THEN** the row shows the correct file status.
- **WHEN** a file is staged-only, unstaged-only, or partially staged **THEN** the row visually reflects that distinction.
- **WHEN** the repository becomes clean **THEN** the workspace shows a clean empty state and no stale rows remain selected.

#### R3. Changed-files workspace list UI

The repo shell must expose a production file list surface for local changes.

**Scenarios:**
- **WHEN** the changes workspace route loads **THEN** users see a list of changed files sourced from live repo data, not spike-only sample data.
- **WHEN** the list is loading **THEN** the UI shows a clear loading state rather than an empty list that looks broken.
- **WHEN** loading fails **THEN** the UI shows an actionable error state instead of silently clearing the list.

#### R4. File selection handoff

This bead must establish file selection behavior for the changes workspace.

**Scenarios:**
- **WHEN** the user selects a file row **THEN** the workspace records that file as the active selection.
- **WHEN** the selected file disappears after refresh **THEN** the workspace clears or reassigns selection predictably rather than pointing at missing data.
- **WHEN** downstream diff/staging work lands **THEN** it can consume the same selection state without changing the list contract.

#### R5. Route and shell integration

The changed-files workspace must integrate into the repo shell without overwriting the orientation summary route.

**Scenarios:**
- **WHEN** the feature is implemented **THEN** the repo shell supports a dedicated changes workspace route in addition to the existing repo landing page.
- **WHEN** users enter the changes workspace **THEN** they stay inside the repo shell context established by `bd-6ew`.
- **WHEN** future staging tasks are implemented **THEN** they attach to this route instead of creating a parallel competing workspace surface.

### Non-Functional Requirements

- **Performance:** Changed-file metadata should load fast enough to feel immediate on typical local repos; avoid eagerly coupling the file list to full diff payloads when not necessary.
- **Security:** Only paths inside the active local repository are surfaced; invalid repo state must fail safely with structured errors.
- **Accessibility:** File rows must remain keyboard reachable and expose selected state and status semantics clearly.
- **Compatibility:** Must fit the existing Tauri 2.0 + SvelteKit repo shell architecture and reuse the open-per-call git2 pattern.
- **Reuse:** The data model must be stable enough for `bd-20d.2` and `bd-20d.3` to extend rather than replace.

## Success Criteria

- [ ] Users can open a dedicated changes workspace route and see a production list of changed files from the active repo.
  - Verify: manual app check in Tauri dev/build session
- [ ] Each row exposes accurate dual-state file information sufficient to distinguish staged-only, unstaged-only, and partially staged files.
  - Verify: targeted backend tests for mixed-status repos
- [ ] Selecting a file updates workspace selection predictably and survives refreshes when the file still exists.
  - Verify: manual app check + focused frontend/state test coverage where available
- [ ] Empty, loading, and error states are visible and trustworthy.
  - Verify: manual app check with clean repo, slow/failing load, and changed repo
- [ ] Project verification remains green after implementation.
  - Verify: `pnpm check`
  - Verify: `pnpm build`
  - Verify: `cargo check` (run in `src-tauri/`)

## Technical Context

### Existing Patterns

- `src-tauri/src/git/repository.rs:10-58` — existing repo status and diff file types; good starting point for extending file-state semantics.
- `src-tauri/src/git/repository.rs:205-276` — `diff_workdir()` already materializes changed files, hunks, and lines from git2.
- `src/lib/stores/diff.svelte.ts:36-168` — runes-based selection/loading store with request-id race guards and refresh logic.
- `src/routes/repo/+page.svelte:1-199` — current repo landing surface that should remain the orientation page, not be replaced by the changes workspace.
- `.beads/artifacts/bd-htm/prd.md:54-60` — prior spike established reusable diff retrieval and targeted refresh architecture for downstream work.
- `src/lib/components/DiffViewer.svelte:1-220` — caller-owned state pattern for empty/loading/error/ready diff rendering.

### Key Constraints

- Parent `bd-20d` is still draft-only, so this child PRD must stay tightly aligned to the bead description and existing code evidence instead of inheriting a finalized parent PRD.
- `bd-20d.2` is blocked by this bead, so the changed-files model should avoid forcing a second backend redesign just to support hunk staging.
- Current `RepoStatusInfo` in `src-tauri/src/git/repository.rs:10-14` only exposes aggregate changed/staged counts; it is insufficient for per-row accuracy by itself.
- The Spike D foundation (`bd-htm`) already proved real changed-file retrieval and watcher refresh, but its `/spike-d` route is not the product workspace.
- `git2::Repository` remains non-`Send + Sync`; open-per-call patterns in commands and repository helpers must remain.
- Memory search was attempted during refinement but unavailable in this session (`FTS5 not available`), so this PRD relies on bead artifacts, codebase evidence, and git history.
- Recent git history shows `bd-6ew.1` and `bd-6ew.2` landed repo-shell routes, while `bd-htm` landed real diff/list foundation; this bead should build on those changes rather than duplicating them.

### Affected Files

```yaml
files:
  - src-tauri/src/git/repository.rs # Extend/adapt file-state model for dual staged/unstaged row semantics
  - src-tauri/src/commands.rs # Expose workspace-friendly changed-file command or reshape existing diff command output
  - src-tauri/src/lib.rs # Register any new workspace file-list command if introduced
  - src/lib/stores/diff.svelte.ts # Separate file-list metadata, selection state, and refresh behavior for production workspace use
  - src/routes/repo/+layout.svelte # Optional shell navigation entry for the changes workspace route
  - src/routes/repo/+page.svelte # Preserve landing summary and link/handoff into changes workspace
  - src/routes/repo/changes/+page.svelte # New production changes workspace route with file list surface
  - src/lib/components/ui/Badge.svelte # Reuse existing badge patterns for file-state chips (no new wrapper component unless necessary)
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Backend model stays worktree-only and forces a second redesign in `bd-20d.2` | Medium | High | Lock the dual-state row contract in this bead so later staging work extends instead of replaces it. |
| Workspace route collides with the repo landing summary added in `bd-6ew.2` | Medium | Medium | Keep the landing page intact and add a dedicated nested changes route under the existing repo shell. |
| File list becomes coupled to full diff payloads and feels slow on larger repos | Medium | Medium | Keep list metadata lightweight and allow heavier diff content to remain lazy-loaded by selection. |
| Watcher refreshes clear selection or show stale rows unpredictably | Medium | Medium | Reuse the request-id and selection-validity guards already present in `diffStore`. |
| Scope creeps into hunk actions or line-level interactions | Medium | High | Keep this bead focused on file-state contract + list UI only; defer mutations to `bd-20d.2` and `bd-20d.3`. |

## Open Questions

None. The primary modeling choice was resolved during refinement: each changed-file row should use a dual-state representation that can expose staged and unstaged state together.

## Tasks

### Backend dual-state file model [backend]

Extend the repo diff/status backend so the changes workspace can render one stable row per changed file with staged and unstaged state together.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src-tauri/src/git/repository.rs
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
```

**Verification:**

- `cargo check`
- Backend tests cover modified, added, deleted, renamed, staged-only, unstaged-only, and partially staged files
- Invalid repo paths return structured errors, not panics

### Workspace list state and selection store [frontend]

Refine frontend store state so the production changes workspace can load file-list metadata, track active selection, and refresh predictably.

**Metadata:**

```yaml
depends_on: ["Backend dual-state file model"]
parallel: false
conflicts_with: []
files:
  - src/lib/stores/diff.svelte.ts
```

**Verification:**

- `pnpm check`
- Store exposes changed-file rows, active selection, loading state, and error state
- Selection remains valid after refresh when the file still exists and resets safely when it does not

### Repo changes workspace route and file list UI [ui]

Add a dedicated repo-shell changes route that renders the production changed-files list with trustworthy empty/loading/error states.

**Metadata:**

```yaml
depends_on: ["Workspace list state and selection store"]
parallel: false
conflicts_with: []
files:
  - src/routes/repo/+layout.svelte
  - src/routes/repo/+page.svelte
  - src/routes/repo/changes/+page.svelte
```

**Verification:**

- `pnpm check`
- Users can enter the changes workspace from the repo shell
- File rows show accurate status chips for mixed staged/unstaged states
- Empty, loading, and error states are visually distinct and understandable

### Downstream handoff for staging work [integration]

Leave the route, row contract, and selection semantics ready for `bd-20d.2` to add hunk-level actions without reshaping the workspace.

**Metadata:**

```yaml
depends_on: ["Repo changes workspace route and file list UI"]
parallel: true
conflicts_with: []
files:
  - src/lib/stores/diff.svelte.ts
  - src/routes/repo/changes/+page.svelte
```

**Verification:**

- `pnpm check`
- The workspace exposes one stable selected-file contract that downstream staging features can consume
- `bd-20d.2` can attach hunk actions without changing route identity or row semantics

---

## Notes

- This PRD intentionally reuses the real diff/list foundation already proven in `bd-htm`, but redirects it into the production repo shell created by `bd-6ew`.
- Refinement decision from user: **Dual-state row (Recommended)** — each changed-file row should expose staged and unstaged state together to avoid rework in later staging beads.
- No relevant TODO/FIXME markers were found in targeted source reads during refinement.
