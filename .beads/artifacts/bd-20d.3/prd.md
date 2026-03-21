# PRD: Line-level selection and patch application

**Bead:** bd-20d.3
**Parent:** bd-20d (Changes Workspace and Partial Staging)
**Depends on:** bd-20d.2
**Type:** task
**Priority:** P1

```yaml
requirements_score:
  total: 93
  breakdown:
    business_value: 26
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

mongit now has production hunk-level stage and unstage actions (bd-20d.2), but users cannot select individual lines within a hunk for partial staging. When a hunk contains multiple logical changes — a bug fix interleaved with a formatting tweak, or an import alongside unrelated deletions — the only option is to stage the entire hunk or nothing. This forces users back to `git add -p` with its manual split workflow, or to accept imprecise commits.

This bead extends partial staging down to line granularity: users can select specific changed lines within a hunk, and stage or unstage only those lines. This is the deepest level of staging precision and a key differentiator that puts mongit on par with JetBrains VCS and ahead of most standalone Git GUIs.

## Scope

### In-Scope

- Add backend `build_line_patch()` that constructs a valid unified diff from selected line indices within a hunk, with automatic context line preservation
- Add `stage_lines` and `unstage_lines` IPC commands that accept file path, hunk index, and selected line indices
- Add line-level selection UI to the existing hunk rendering in `/repo/changes`: clickable change lines (added/removed), multi-select support, visual selection state
- Context lines are automatically included in patches and are not selectable by users
- Validate line selections and return structured errors for invalid or ambiguous selections
- Recalculate hunk header line counts (`@@ -old,count +new,count @@`) based on selected lines
- Refresh workspace state (file list + hunk view) after successful line-level staging
- Support both directions: stage selected unstaged lines, unstage selected staged lines

### Out-of-Scope

- Drag-to-select or rectangular selection (checkbox/click selection only for this bead)
- Keyboard-only line selection shortcuts beyond basic tab/space (defer to polish bead)
- Line-level staging for binary files (continue to block at hunk level)
- Automatic hunk splitting (user manually selects lines; no auto-detection of logical groups)
- Conflict resolution or 3-way merge at line level
- Undo/redo for line-level staging (defer to undo system bead)
- Redesigning the hunk rendering layout or diff viewer

## Proposed Solution

Build on the existing hunk patch infrastructure from bd-20d.2, extending it in three layers:

1. **Backend line patch construction**
   - Add `build_line_patch()` in `staging.rs` that takes a `DiffHunkInfo`, a set of selected line indices, and the file status, then produces a valid unified diff patch containing only the selected change lines plus automatically-included context lines.
   - Recalculate the `@@ -old_start,old_lines +new_start,new_lines @@` header based on which lines are included.
   - For selected lines: include the change line as-is. For unselected `-` lines: convert to context lines (they remain in the file). For unselected `+` lines: omit entirely (they don't exist in the original).
   - Validate that at least one change line is selected and the resulting patch is non-empty.

2. **IPC commands**
   - `stage_lines(path, file_path, hunk_index, line_indices)` — reads unstaged diff, builds line patch, applies via `git apply --cached --unidiff-zero`
   - `unstage_lines(path, file_path, hunk_index, line_indices)` — reads staged diff, builds reverse line patch, applies via `git apply --cached --reverse --unidiff-zero`
   - Reuse the same `StageOpError` discriminated union from bd-20d.2, adding a new `InvalidLineSelection` variant.

3. **Frontend line selection UI**
   - Add line selection state to the diff store: a map of `hunkIndex → Set<lineIndex>` for tracking selected lines per hunk.
   - Make change lines (origin `+` or `-`) clickable to toggle selection. Context lines (origin ` `) are not selectable.
   - Show a "Stage Selected (N)" / "Unstage Selected (N)" button in the hunk header when any lines are selected, alongside the existing "Stage" / "Unstage" whole-hunk button.
   - Visual feedback: selected lines get a highlight/accent background. Unselected change lines remain normally styled.
   - Clear selection state after successful staging or on file change.

### User Flow

1. User opens `/repo/changes` and selects a changed file — sees diff hunks as before.
2. User clicks on individual changed lines (+ or - lines) to toggle their selection. Selected lines highlight.
3. A "Stage Selected (3)" button appears in the hunk header showing the count.
4. User clicks "Stage Selected" — only those 3 lines are staged.
5. The workspace refreshes: the hunk now shows fewer unstaged changes, and the staged side gains those lines.
6. If the resulting patch can't apply, a structured error explains why.

### Line Patch Construction Rules

For a hunk with lines at indices 0..N, given a set of selected indices S:

```
For each line in the hunk:
  if line.origin == ' ' (context):
    → always include as context line ' '
  if line.origin == '-' (removed):
    if index in S:
      → include as removed line '-' (this deletion will be staged)
    else:
      → include as context line ' ' (this deletion stays unstaged, line remains)
  if line.origin == '+' (added):
    if index in S:
      → include as added line '+' (this addition will be staged)
    else:
      → omit entirely (this addition stays only in the working tree)
  if line.origin == '\\' (no newline marker):
    → include if the preceding line was included
```

After filtering, recalculate:
- `old_lines` = count of context lines + count of removed lines
- `new_lines` = count of context lines + count of added lines

## Requirements

### Functional Requirements

#### R1. Line-level patch construction

The backend must construct valid unified diff patches from a subset of changed lines within a hunk.

**Scenarios:**
- **WHEN** the user selects a subset of added lines from a hunk **THEN** only those additions appear in the staged patch; unselected additions remain in the working tree.
- **WHEN** the user selects a subset of removed lines **THEN** only those deletions are staged; unselected deletions remain as unstaged changes.
- **WHEN** the user selects a mix of added and removed lines **THEN** the patch correctly includes both the selected additions and deletions with proper context.
- **WHEN** no change lines are selected (only context) **THEN** the backend rejects the request with an `InvalidLineSelection` error.
- **WHEN** all change lines in a hunk are selected **THEN** the result is functionally equivalent to staging the whole hunk.

#### R2. Context preservation in line patches

Generated patches must include sufficient context for `git apply` to locate the correct position.

**Scenarios:**
- **WHEN** a line patch is constructed **THEN** all context lines from the original hunk are preserved (they are structural, not optional).
- **WHEN** an unselected `-` line exists **THEN** it becomes a context line in the patch (the line remains in the file).
- **WHEN** an unselected `+` line exists **THEN** it is omitted from the patch entirely (it exists only in the working tree).
- **WHEN** the patch includes a `\ No newline at end of file` marker **THEN** it follows the same inclusion rules as its preceding line.

#### R3. Line-level IPC commands

The backend must expose `stage_lines` and `unstage_lines` IPC commands.

**Scenarios:**
- **WHEN** `stage_lines` is called with valid repo path, file path, hunk index, and line indices **THEN** only the selected lines are staged via `git apply --cached --unidiff-zero`.
- **WHEN** `unstage_lines` is called with valid parameters **THEN** only the selected lines are unstaged via `git apply --cached --reverse --unidiff-zero`.
- **WHEN** the line indices contain out-of-range values **THEN** an `InvalidLineSelection` error is returned.
- **WHEN** the line indices contain only context line indices **THEN** an `InvalidLineSelection` error is returned (no change to apply).

#### R4. Line selection UI

The changes workspace must allow users to select individual changed lines within a hunk.

**Scenarios:**
- **WHEN** a hunk is rendered **THEN** each added (`+`) and removed (`-`) line is clickable to toggle selection.
- **WHEN** one or more lines are selected **THEN** a "Stage Selected (N)" or "Unstage Selected (N)" button appears in the hunk header.
- **WHEN** no lines are selected **THEN** only the existing whole-hunk "Stage" / "Unstage" button is visible (no regression).
- **WHEN** the user switches to a different file **THEN** line selection state is cleared.
- **WHEN** a line-level staging action succeeds **THEN** line selection state is cleared and the workspace refreshes.
- **WHEN** context lines are displayed **THEN** they are not clickable or selectable.

#### R5. Refresh and consistency after line-level mutation

The workspace must stay consistent after line-level staging, following the same patterns as hunk-level.

**Scenarios:**
- **WHEN** line-level staging succeeds **THEN** both diffStore and changesStore refresh to reflect the new state.
- **WHEN** the staged lines cause the hunk to fully disappear from the unstaged side **THEN** the workspace handles the missing hunk cleanly.
- **WHEN** the file is still selected after refresh **THEN** selection persists on that file.

### Non-Functional Requirements

- **Performance:** Line selection toggle must be instantaneous (<16ms). Patch construction and application should feel immediate on normal repos.
- **Security:** Same repo-boundary enforcement as hunk-level staging.
- **Accessibility:** Selected/unselected line state must be distinguishable beyond color alone (e.g., accent bar or checkbox). Line selection must work with keyboard (tab to line, space/enter to toggle).
- **Compatibility:** Must work alongside existing hunk-level stage/unstage without regression. Both actions coexist on the same hunk.
- **Robustness:** Invalid selections produce clear structured errors, not silent failures or panics.

## Success Criteria

- [ ] Users can click individual changed lines to toggle selection and see visual feedback.
  - Verify: manual app check in Tauri dev session
- [ ] "Stage Selected (N)" button stages only the selected lines, leaving unselected changes untouched.
  - Verify: backend tests with multi-line hunks verifying selective staging
- [ ] "Unstage Selected (N)" button unstages only the selected lines from the index.
  - Verify: backend tests for staged diff reversal with line subsets
- [ ] Context lines are always preserved; unselected `-` lines become context; unselected `+` lines are omitted.
  - Verify: unit tests for `build_line_patch()` covering all line origin combinations
- [ ] Invalid selections (no change lines, out-of-range indices) return structured errors.
  - Verify: backend tests for error variants
- [ ] Existing hunk-level stage/unstage still works without regression.
  - Verify: existing hunk-level tests pass unchanged
- [ ] Workspace refreshes correctly after line-level staging.
  - Verify: manual app check + store behavior tests
- [ ] Project verification remains green.
  - Verify: `cargo check` (in `src-tauri/`)
  - Verify: `cargo test` (in `src-tauri/`)
  - Verify: `pnpm check`
  - Verify: `pnpm build`

## Technical Context

### Existing Patterns

- `src-tauri/src/git/staging.rs` — contains `build_hunk_patch()`, `stage_hunk()`, `unstage_hunk()` that this bead extends with line-level equivalents.
- `src-tauri/src/git/repository.rs` — defines `DiffHunkInfo`, `DiffLineInfo`, `DiffFileEntry` with full line metadata (origin, content, line numbers).
- `src-tauri/src/git/error.rs` — `StageOpError` discriminated union with `PatchFailed`, `InvalidHunkIndex`, `FileNotInDiff`, `BinaryNotSupported`, `GenericStageFailed` variants.
- `src-tauri/src/commands.rs` — `stage_hunk`/`unstage_hunk` IPC commands with `spawn_blocking` + `Result<(), String>` pattern.
- `src/lib/stores/diff.svelte.ts` — diff store with request-ID guards, `stageHunk()`/`unstageHunk()` methods, `staging` flag for in-flight protection.
- `src/routes/repo/changes/+page.svelte` — hunk rendering with per-line display (origin + content), hunk-level action buttons.
- `--unidiff-zero` flag already used in hunk staging — allows zero-context patches which is essential for line-level patches.

### Key Constraints

- `build_line_patch()` must produce patches compatible with `git apply --cached --unidiff-zero` — this is the same apply path used for hunk staging.
- Unselected `-` lines must become context lines (` ` prefix) in the patch, not be omitted — omitting them would create an invalid patch because the old file content wouldn't match.
- Unselected `+` lines must be omitted entirely — they don't exist in the old file content, so including them as context would be invalid.
- The `\ No newline at end of file` marker must follow the inclusion fate of its preceding line.
- Line indices are relative to `hunk.lines` (0-based), not to file line numbers.
- Hunk header recalculation must be exact: `old_lines` = context + removed lines in patch; `new_lines` = context + added lines in patch.

### Affected Files

```yaml
files:
  - src-tauri/src/git/staging.rs    # Add build_line_patch(), stage_lines(), unstage_lines()
  - src-tauri/src/git/error.rs      # Add InvalidLineSelection error variant
  - src-tauri/src/commands.rs       # Add stage_lines and unstage_lines IPC commands
  - src-tauri/src/lib.rs            # Register new commands
  - src/lib/stores/diff.svelte.ts   # Add line selection state and stageLines/unstageLines methods
  - src/routes/repo/changes/+page.svelte  # Add line selection UI and "Stage Selected" buttons
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Line patch construction produces invalid patches for edge cases (single-line hunks, all-add hunks) | Medium | High | Comprehensive unit tests covering every combination: add-only, delete-only, mixed, single-line, no-newline-at-EOF |
| Unselected `-` lines incorrectly omitted instead of converted to context, causing patch apply failure | Medium | High | Explicit test for unselected-removal-becomes-context rule; verify against real git repo |
| Hunk header line count recalculation off-by-one | Medium | High | Test patches against `git apply --check` before applying; compare expected header values in unit tests |
| Line selection state gets out of sync with hunk data after refresh | Medium | Medium | Clear selection on any diff refresh; tie selection to hunk identity |
| Performance degrades with many selectable lines in large diffs | Low | Medium | Selection is a simple Set lookup; no DOM overhead beyond existing line rendering |
| Users confused by which lines are selectable vs context | Low | Medium | Clear visual distinction: cursor change on hover for selectable lines; muted style for context |

## Open Questions

None. The patch construction rules (unselected `-` → context, unselected `+` → omit) follow standard Git partial staging semantics used by `git add -p` split mode.

## Tasks

### Task 1: Line patch construction engine [backend]

Add `build_line_patch()` to `staging.rs` that constructs a valid unified diff from selected line indices within a hunk, with proper context preservation and hunk header recalculation.

**Metadata:**

```yaml
depends_on: []
parallel: false
conflicts_with: []
files:
  - src-tauri/src/git/staging.rs
  - src-tauri/src/git/error.rs
```

**Verification:**

- `cargo check`
- `cargo test`
- Unit tests cover: add-only selection, delete-only selection, mixed selection, all-lines selection (equivalent to hunk), single-line selection, no-newline-at-EOF handling, invalid selection errors

### Task 2: Line-level IPC commands [backend]

Add `stage_lines` and `unstage_lines` IPC commands that use `build_line_patch()` and apply via `git apply --cached --unidiff-zero`.

**Metadata:**

```yaml
depends_on: ["Task 1: Line patch construction engine"]
parallel: false
conflicts_with: []
files:
  - src-tauri/src/git/staging.rs
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
```

**Verification:**

- `cargo check`
- `cargo test`
- Integration tests cover: stage selected lines from unstaged diff, unstage selected lines from staged diff, multi-hunk isolation, error cases (invalid indices, no change lines, binary file)

### Task 3: Line selection state and store methods [frontend]

Add line selection tracking to `diff.svelte.ts` with `stageLines()` and `unstageLines()` methods that invoke the new IPC commands.

**Metadata:**

```yaml
depends_on: ["Task 2: Line-level IPC commands"]
parallel: false
conflicts_with: []
files:
  - src/lib/stores/diff.svelte.ts
```

**Verification:**

- `pnpm check`
- Store exposes line selection state and methods for toggling, clearing, and submitting selections

### Task 4: Line selection UI in changes workspace [ui]

Extend `/repo/changes` hunk rendering with clickable change lines, visual selection state, and "Stage Selected (N)" / "Unstage Selected (N)" buttons.

**Metadata:**

```yaml
depends_on: ["Task 3: Line selection state and store methods"]
parallel: false
conflicts_with: []
files:
  - src/routes/repo/changes/+page.svelte
```

**Verification:**

- `pnpm check`
- `pnpm build`
- Change lines are clickable with visual feedback
- "Stage Selected" / "Unstage Selected" buttons appear with correct count
- Context lines are not selectable
- Selection clears after successful action or file change
- Existing hunk-level buttons still work

---

## Notes

- The line patch construction rules (unselected `-` → context, unselected `+` → omit) are the same semantics used by `git add -p` when splitting hunks and by JetBrains IDE line-level staging.
- `--unidiff-zero` is already used in hunk staging and is essential here since line-level patches may have zero surrounding context after filtering.
- This bead intentionally keeps the selection mechanism simple (click to toggle). More advanced selection (shift-click ranges, drag selection) can be added in a polish pass.
