# PRD: Hunk-level stage and unstage actions

**Bead:** bd-20d.2
**Parent:** bd-20d (Changes Workspace and Partial Staging)
**Depends on:** bd-20d.1
**Type:** task
**Priority:** P0

```yaml
requirements_score:
  total: 94
  breakdown:
    business_value: 27
    functional_requirements: 24
    user_experience: 18
    technical_constraints: 15
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

mongit now has a production changed-files workspace with accurate dual-state file rows, selection state, and route integration, but it still stops before the first real partial-staging action. Power users can see that files are partially staged, yet they cannot act on individual hunks without falling back to the terminal. That leaves the changes workspace short of the parent bead's core promise: making partial staging practical and understandable inside the app.

This bead defines the first mutation slice of that promise: reliable hunk-level stage and unstage actions. It should let users inspect the hunks for the currently selected file, stage or unstage one hunk at a time, and see the workspace refresh into the new staged/unstaged state without widening scope into line-level precision work owned by `bd-20d.3`.

## Scope

### In-Scope

- Add production hunk-level stage and unstage operations for text-file diffs in the changes workspace
- Support both directions of patch application:
  - stage an unstaged hunk into the index
  - unstage a staged hunk back to the working tree
- Reuse the existing `/repo/changes` route, `changesStore`, and diff foundation instead of creating a competing staging surface
- Load and render the selected file's diff hunks so users can act on them from the workspace
- Keep file selection and diff refresh behavior stable after successful hunk mutations
- Surface trustworthy loading, pending, and error states around hunk actions
- Return structured backend errors for patch-application failures so the frontend can explain what happened
- Leave the contract ready for `bd-20d.3` to add line-level selection on top of the same file and hunk identity model

### Out-of-Scope

- Line-level staging, patch splitting, or manual line selection (`bd-20d.3`)
- Whole-file stage/unstage shortcuts beyond whatever already exists in Git outside this workspace
- Commit authoring, amend, push, or branch operations
- Binary-file partial staging support
- Conflict-resolution or 3-way patch conflict UX
- Redesigning the repo shell or replacing the file-list UI delivered in `bd-20d.1`
- Multi-file batch staging flows

## Proposed Solution

Split this slice into a backend patch-application engine and a user-facing hunk action UI on the existing changes route.

1. **Backend hunk mutation contract**
   - Build on the current diff/hunk model in `src-tauri/src/git/repository.rs` and the existing CLI write path in `src-tauri/src/git/cli.rs`.
   - Generate or reconstruct a minimal unified patch for a single selected hunk, then apply it via the real git binary rather than libgit2 mutation APIs.
   - Use `git apply --cached` for staging and reverse/corresponding apply semantics for unstaging, with validation and structured failure reporting.
   - Keep hunk mutation text-only for this bead; binary or unsupported patch cases must fail safely with a clear structured error.

2. **Workspace state and refresh flow**
   - Reuse the request-guard pattern already established in `changesStore` and `diffStore`.
   - Keep file-list metadata (`changesStore`) separate from diff payloads (`diffStore`), but make them refresh together after a successful hunk action so badges and hunks stay aligned.
   - Preserve the existing `/repo/changes` route identity and selection semantics from `bd-20d.1`.

3. **User-facing hunk actions in `/repo/changes`**
   - Extend the route so selecting a changed file also reveals its diff hunks in the same workspace.
   - Show explicit hunk actions only at hunk granularity in this bead: stage or unstage each hunk based on whether the hunk belongs to the unstaged or staged side.
   - While an action is in flight, disable duplicate actions for that hunk and communicate pending state clearly.
   - On success, refresh the workspace and keep focus/selection predictable so users can continue staging multiple hunks quickly.

### User Flow

1. User opens `/repo/changes` and selects a changed file from the existing file list.
2. The workspace loads that file's diff hunks and indicates which hunks are currently stageable or unstageable.
3. User clicks a hunk-level action to stage or unstage exactly one hunk.
4. The backend applies the patch via git CLI, the UI shows pending state, and then the file list + hunk view refresh.
5. If the patch no longer applies cleanly, the workspace shows a specific error instead of silently failing or corrupting state.

## Requirements

### Functional Requirements

#### R1. Hunk-level patch application contract

The backend must expose a production mutation contract that can stage or unstage exactly one hunk at a time for a selected file.

**Scenarios:**
- **WHEN** the frontend requests staging for a valid unstaged hunk **THEN** the backend applies only that hunk to the index and leaves unrelated hunks untouched.
- **WHEN** the frontend requests unstaging for a valid staged hunk **THEN** the backend removes only that hunk from the index while preserving working-tree content appropriately.
- **WHEN** a patch cannot be applied because the file changed, the hunk is stale, or the patch shape is unsupported **THEN** the backend returns a structured error instead of panicking or partially mutating unknown state.

#### R2. Stable hunk identity and action targeting

The workspace must target the intended hunk predictably enough that later line-level work can build on the same model.

**Scenarios:**
- **WHEN** a file has multiple hunks **THEN** the workspace can address one hunk without affecting its siblings.
- **WHEN** the selected file refreshes after a successful action **THEN** the refreshed hunk list still maps back to the same selected file path and current repo state.
- **WHEN** `bd-20d.3` extends this flow **THEN** it can add line-level targeting without redefining file selection or route identity.

#### R3. User-facing hunk actions in the existing changes workspace

`bd-20d.2` must ship the first visible hunk-level stage/unstage controls on `/repo/changes`, not only backend plumbing.

**Scenarios:**
- **WHEN** a selected file has stageable or unstageable hunks **THEN** the route displays actionable controls for each hunk.
- **WHEN** a hunk action is pending **THEN** duplicate clicks are prevented and the UI communicates that the action is running.
- **WHEN** the selected file has no renderable hunks for this bead's supported cases **THEN** the workspace explains why instead of showing a blank pane.

#### R4. Refresh and consistency after mutation

The file list, staged/unstaged badges, and hunk panel must stay trustworthy after each mutation.

**Scenarios:**
- **WHEN** a hunk stage/unstage succeeds **THEN** the file list and hunk data refresh to reflect the new repo state.
- **WHEN** the selected file still exists after refresh **THEN** selection remains on that file.
- **WHEN** the action changes the file so it no longer has remaining hunks in the current view **THEN** the workspace transitions cleanly rather than showing stale hunk UI.

#### R5. Text-only safety boundary for this bead

This bead must support common text diffs while failing safely for unsupported patch cases.

**Scenarios:**
- **WHEN** the selected diff represents a normal text-file hunk **THEN** the hunk action flow is supported.
- **WHEN** the target is binary, unsupported, or otherwise not safely patch-applicable **THEN** the workspace blocks the action and explains the limitation.
- **WHEN** the file path is invalid or escapes repo assumptions **THEN** the command fails safely with structured error handling.

### Non-Functional Requirements

- **Performance:** Hunk actions should feel immediate on normal local repos; avoid reloading more workspace data than needed, but always prefer correctness over speculative partial updates.
- **Security:** All mutation targets must stay inside the active repository; patch application must not allow path traversal or arbitrary file writes.
- **Accessibility:** Hunk action controls must remain keyboard reachable, expose disabled/pending state, and preserve focus predictably after refresh.
- **Compatibility:** Must fit the existing Tauri 2.0 + SvelteKit repo shell architecture, reuse the resolved git binary path, and preserve the open-per-call git2 pattern.
- **Robustness:** Patch failures must be surfaced as structured, typed errors suitable for frontend rendering and debugging.
- **Reuse:** The file selection route and hunk identity model must remain stable for `bd-20d.3` line-level selection.

## Success Criteria

- [ ] Users can select a changed file in `/repo/changes` and see stageable/unstageable hunks with explicit hunk-level actions.
  - Verify: manual app check in Tauri dev/build session
- [ ] Staging one unstaged hunk updates the index without staging unrelated hunks from the same file.
  - Verify: targeted backend tests for multi-hunk files and partial staging behavior
- [ ] Unstaging one staged hunk removes only that hunk from the index while keeping the workspace consistent.
  - Verify: targeted backend tests for staged diff reversal behavior
- [ ] Patch-application failures return structured errors and produce clear UI feedback.
  - Verify: backend tests for stale/unsupported patch cases + manual UI check
- [ ] File selection and staged/unstaged row badges refresh predictably after each successful hunk action.
  - Verify: manual app check + focused store/state test coverage where available
- [ ] Project verification remains green after implementation.
  - Verify: `cargo check` (run in `src-tauri/`)
  - Verify: `cargo test` (run in `src-tauri/`)
  - Verify: `pnpm check`
  - Verify: `pnpm build`

## Technical Context

### Existing Patterns

- `src-tauri/src/git/repository.rs` — already defines `DiffHunkInfo`, `DiffLineInfo`, `DiffFileEntry`, and `ChangedFileEntry`; this is the read-side contract hunk actions should build on.
- `src-tauri/src/git/cli.rs` — existing write path for git mutations via resolved git binary; hunk actions should follow the same CLI-backed mutation pattern as branch operations.
- `src-tauri/src/git/error.rs` — structured error serialization pattern (`#[serde(tag = "kind")]`) already exists for branch operations and should be mirrored for hunk-action failures.
- `src-tauri/src/commands.rs` — all git work runs through `spawn_blocking` wrappers and returns `Result<T, String>` for Tauri IPC.
- `src/lib/stores/diff.svelte.ts` — request-ID guarded diff loading and selected-file content loading already exist; hunk UI should reuse this store or its pattern instead of inventing a parallel async model.
- `src/lib/stores/changes.svelte.ts` — production file-list metadata store with selection and refresh semantics already shipped in `bd-20d.1`.
- `src/routes/repo/changes/+page.svelte` — stable route delivered in `bd-20d.1`; this bead should extend it rather than add another workspace route.
- `src-tauri/src/watcher.rs` and current route listener pattern — repo-changed refresh is already wired into `/repo/changes`; mutation success should cooperate with the same refresh model.

### Key Constraints

- `bd-20d.2` must advance the parent bead's user-facing partial staging promise, not stop at backend-only plumbing.
- `bd-20d.3` depends on this bead, so hunk actions must define a stable base contract rather than a throwaway implementation.
- The project intentionally routes writes through the real git CLI because libgit2 does not cover the needed mutation behavior safely enough for this product direction.
- `git apply --cached` applies only to the index; `git apply` semantics differ depending on flags, so staged vs unstaged flows must be modeled explicitly and tested carefully.
- Official Git docs note that `git apply --cached` touches only the index, while `--check` can validate applicability first; `--index` expects index and working tree copies to match exactly, so it is the wrong default for partial staging in a dirty worktree.
- `git apply` expects real unified diff patch text with proper file headers and hunk headers; context-free or malformed patches are brittle and should not be the default contract.
- Text-only support is the safe MVP boundary here; binary partial staging should be explicitly excluded for this bead.
- Recent fixes in `bd-20d.1`, `bd-6ew.1`, and `bd-htm.3` all reinforced lifecycle/race-guard correctness, so mutation flows should preserve that discipline.

### Affected Files

```yaml
files:
  - src-tauri/src/git/cli.rs # Add hunk stage/unstage git CLI mutation helpers
  - src-tauri/src/git/error.rs # Add structured hunk-operation error types and stderr parsing
  - src-tauri/src/git/repository.rs # Extend/read staged vs unstaged diff data needed to target hunks
  - src-tauri/src/git/mod.rs # Add backend integration tests and test helpers for hunk mutation flows
  - src-tauri/src/commands.rs # Expose Tauri IPC commands for stage_hunk and unstage_hunk
  - src-tauri/src/lib.rs # Register new Tauri commands
  - src/lib/stores/diff.svelte.ts # Load selected-file hunk data and coordinate refresh after mutation
  - src/lib/stores/changes.svelte.ts # Refresh file-row dual-state metadata after hunk actions
  - src/routes/repo/changes/+page.svelte # Render hunk list and stage/unstage controls inside existing workspace
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Single-hunk patch generation is incorrect and stages too much or too little | Medium | High | Require targeted backend tests with multi-hunk files and verify unaffected hunks remain unchanged |
| Stale diff data causes patch apply failures after external edits or watcher refreshes | Medium | High | Return structured stale-patch errors and force workspace refresh after mutation attempts |
| Unstage semantics are modeled incorrectly against the index/working tree boundary | Medium | High | Test staged-only, partially staged, and reverse-apply scenarios explicitly before shipping |
| Route complexity grows into a premature diff editor redesign | Medium | Medium | Keep UI at hunk-level actions only; defer line-level precision and broader redesign to `bd-20d.3` |
| Unsupported cases (binary, malformed, rename edge cases) confuse users | Medium | Medium | Fail safely, explain unsupported state, and document text-only boundary in UI and PRD |
| Mutation actions race with existing watcher/store refresh behavior | Medium | Medium | Reuse current request-guard patterns and refresh sequencing instead of optimistic local mutation |

## Open Questions

None. Refinement resolved the key scope question: `bd-20d.2` should ship backend patch application plus the first user-facing hunk controls in `/repo/changes`.

## Tasks

### Backend hunk patch mutation engine [backend]

Add production backend commands that can stage or unstage one text hunk at a time using the resolved git CLI and structured error handling.

**Metadata:**

```yaml
depends_on: []
parallel: false
conflicts_with: []
files:
  - src-tauri/src/git/cli.rs
  - src-tauri/src/git/error.rs
  - src-tauri/src/git/repository.rs
  - src-tauri/src/git/mod.rs
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
```

**Verification:**

- `cargo check`
- `cargo test`
- Backend tests cover stage hunk, unstage hunk, multi-hunk isolation, stale patch failure, and unsupported/binary-safe failure
- Invalid repo/file paths return structured errors, not panics

### Workspace diff and hunk action state [frontend]

Extend the frontend state layer so the selected file can expose actionable hunks and refresh both diff + changed-file metadata after mutations.

**Metadata:**

```yaml
depends_on: ["Backend hunk patch mutation engine"]
parallel: false
conflicts_with: []
files:
  - src/lib/stores/diff.svelte.ts
  - src/lib/stores/changes.svelte.ts
```

**Verification:**

- `pnpm check`
- Selected-file diff state exposes the hunks needed for rendering and action targeting
- Successful hunk actions refresh both diff and file-row metadata consistently
- Selection remains valid after refresh when the file still exists

### Changes workspace hunk action UI [ui]

Extend `/repo/changes` so users can inspect the selected file's hunks and stage or unstage each hunk from the existing workspace.

**Metadata:**

```yaml
depends_on: ["Workspace diff and hunk action state"]
parallel: false
conflicts_with: []
files:
  - src/routes/repo/changes/+page.svelte
```

**Verification:**

- `pnpm check`
- `pnpm build`
- Manual app check shows actionable hunk controls on `/repo/changes`
- Pending, success, empty, and error states are visually distinct and understandable
- Hunk actions are keyboard reachable and preserve focus predictably after refresh

### Downstream handoff for line-level staging [integration]

Leave the selected-file, hunk identity, and route contracts stable so `bd-20d.3` can add line-level selection without reshaping this workspace.

**Metadata:**

```yaml
depends_on: ["Changes workspace hunk action UI"]
parallel: true
conflicts_with: []
files:
  - src/lib/stores/diff.svelte.ts
  - src/routes/repo/changes/+page.svelte
  - src-tauri/src/git/repository.rs
```

**Verification:**

- `pnpm check`
- `cargo check`
- The workspace exposes one stable selected-file + hunk contract for downstream line-level targeting
- `bd-20d.3` can attach line-level selection without replacing `/repo/changes` or redefining hunk semantics

---

## Notes

- Refinement decision from user: **Backend + hunk UI** — this bead must ship the first user-facing hunk stage/unstage actions, not only backend plumbing.
- Official Git documentation confirms `git apply --cached` is the correct index-only primitive to center this bead around; `--index` is stricter and expects index + working tree parity, which does not fit the dirty-worktree partial-staging use case.
- The current external scout run returned no useful output, so technical guidance in this PRD relies on codebase evidence plus official Git docs fetched during refinement.
