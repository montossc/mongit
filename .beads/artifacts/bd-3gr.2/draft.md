# Three-pane editor actions and resolution completion

**Bead:** bd-3gr.2
**Type:** task
**Created:** 2026-03-16
**Status:** PRD

---

## Problem Statement

bd-3gr.1 shipped the conflict detection and 3-pane merge editor UI, but all resolution actions are UI-only — clicking "Accept Ours" or "Accept Theirs" updates the center editor pane locally without persisting to the git index or working tree. Users cannot complete a merge from within the app. They must drop to the CLI to resolve conflicts, stage files, and commit.

bd-3gr.2 closes this gap by adding the backend write operations, wiring them to the existing UI, and providing a complete resolve → stage → commit flow.

## Scope

### In-Scope

1. **Backend: `resolve_conflict` command** — Write resolved content to the working tree file and stage it in the git index (mark as resolved)
2. **Backend: `abort_merge` command** — Abort the current merge (`git merge --abort` equivalent)
3. **Backend: `complete_merge` command** — Create a merge commit after all conflicts are resolved
4. **Frontend: Wire Accept Ours / Accept Theirs / manual edits** to call `resolve_conflict` and persist
5. **Frontend: Visual feedback for resolved files** — Mark files as resolved in the conflict file list
6. **Frontend: Abort Merge button** with confirmation dialog
7. **Frontend: Complete Merge button** (enabled when all conflicts resolved) that creates the merge commit
8. **Conflict store updates** — Track resolution state per file, support resolved/unresolved status

### Out-of-Scope

- Conflict chunk navigation (next/prev conflict within a file) — future enhancement
- Keyboard shortcuts for resolution actions — future enhancement
- Semantic/AI-assisted conflict resolution — V2 (bd-2fs)
- Interactive rebase conflict handling — separate work
- Binary file conflict resolution
- Inline conflict marker editing (only 3-pane view for V1)

## Proposed Solution

### Backend (Rust)

Add three new functions to `src-tauri/src/git/conflict.rs`:

1. **`resolve_conflict(path, file_path, content)`**: Write `content` to the working tree file at `file_path`, then stage it in the git index via `index.add_path()` + `index.write()`. This removes the file from the conflict state.

2. **`abort_merge(path)`**: Remove MERGE_HEAD, MERGE_MSG, MERGE_MODE files and reset the index + working tree to HEAD state. Use git2's `repo.cleanup_state()` + `repo.checkout_head()` with force.

3. **`complete_merge(path, message)`**: Read MERGE_HEAD for parent SHAs, create a merge commit with the provided message (or MERGE_MSG default), then clean up merge state files.

Each function gets a corresponding async IPC command in `commands.rs`.

### Frontend (Svelte)

1. **Conflict store** (`conflict.svelte.ts`): Add `resolvedPaths: Set<string>` state, `resolveFile()`, `abortMerge()`, `completeMerge()` methods.

2. **MergeEditor** (`MergeEditor.svelte`): Wire Accept Ours / Accept Theirs buttons to call `resolveFile()`. Add "Apply Manual" button to save center editor content.

3. **Resolve page** (`resolve/+page.svelte`): Add checkmark icon for resolved files, Abort Merge button (top), Complete Merge button (bottom, enabled when `resolvedPaths.size === conflictedFiles.length`).

## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | User can accept "ours" version for a conflict file and it persists to working tree + index | Must |
| R2 | User can accept "theirs" version for a conflict file and it persists | Must |
| R3 | User can manually edit the result pane and save the resolution | Must |
| R4 | Resolved files show visual distinction from unresolved files in the list | Must |
| R5 | User can abort the entire merge and return to pre-merge state | Must |
| R6 | User can complete the merge (commit) after all conflicts are resolved | Must |
| R7 | Complete Merge button is disabled while unresolved conflicts remain | Must |
| R8 | Abort Merge requires confirmation before executing | Must |
| R9 | After completing merge, UI returns to normal (non-merge) state | Must |
| R10 | Error states are shown clearly (e.g., failed to write, failed to commit) | Must |

## Success Criteria

1. **Accept Ours/Theirs persists**: Clicking either button writes the file and stages it; re-reading the git index shows no conflict for that file
2. **Manual resolution works**: Editing the center pane and clicking Apply writes the custom content and stages it
3. **Resolved files are visually marked**: Resolved files show a checkmark or distinct style in the file list
4. **Abort merge restores state**: After abort, MERGE_HEAD is gone, working tree matches HEAD, conflict UI disappears
5. **Complete merge creates commit**: After resolving all files, Complete Merge creates a merge commit with both parent SHAs
6. **All Rust functions have tests**: Unit tests for resolve_conflict, abort_merge, complete_merge
7. **Verification passes**: `pnpm check` 0 errors, `cargo check` clean

## Technical Context

### Existing Code (from bd-3gr.1)

- **`conflict.rs`**: `get_merge_state()`, `get_conflict_content()`, `MergeState`, `ConflictContent` structs
- **`commands.rs`**: `get_merge_state`, `get_conflict_content` IPC commands (async + spawn_blocking)
- **`conflict.svelte.ts`**: Store with `loadMergeState()`, `loadConflictContent()`, `selectFile()`, `refresh()`, `reset()`
- **`MergeEditor.svelte`**: 3-pane CodeMirror layout with `acceptOurs()`, `acceptTheirs()`, `applyToCenter()` (UI-only)
- **`resolve/+page.svelte`**: Split panel with conflict file list + MergeEditor

### Key git2 APIs Needed

- `index.add_path(path)` — stage a file (removes conflict entries)
- `index.write()` — persist index changes
- `repo.cleanup_state()` — remove MERGE_HEAD/MERGE_MSG/MERGE_MODE
- `repo.checkout_head()` — reset working tree to HEAD (for abort)
- `repo.find_commit()` — find parent commits for merge commit
- `repo.commit()` — create the merge commit with multiple parents

### Hybrid Write Pattern

Per project architecture (git2 reads + bundled git writes), resolution writes could use either:
- **git2 directly**: For index staging (`index.add_path`) — simpler, no shell overhead
- **Bundled git**: For merge commit — safer for hooks/signing

For V1, use git2 directly for all operations (resolve + commit). Hook/signing support is out of scope.

## Affected Files

| File | Change |
|------|--------|
| `src-tauri/src/git/conflict.rs` | Add `resolve_conflict()`, `abort_merge()`, `complete_merge()` |
| `src-tauri/src/commands.rs` | Add 3 new IPC commands |
| `src-tauri/src/lib.rs` | Register 3 new commands in invoke_handler |
| `src/lib/stores/conflict.svelte.ts` | Add resolved state tracking, resolveFile/abort/complete methods |
| `src/lib/components/MergeEditor.svelte` | Wire buttons to store methods, add Apply Manual button, emit events |
| `src/routes/repo/resolve/+page.svelte` | Add resolved file styling, Abort/Complete buttons, completion flow |

## Tasks

1. **Backend: resolve_conflict** — Write resolved content + stage in index
2. **Backend: abort_merge** — Clean merge state + reset working tree
3. **Backend: complete_merge** — Create merge commit with both parents
4. **Backend: IPC commands** — Register all 3 in commands.rs + lib.rs
5. **Backend: Tests** — Unit tests for all 3 functions
6. **Frontend: Store updates** — resolvedPaths tracking, new methods
7. **Frontend: MergeEditor wiring** — Connect buttons to backend via store
8. **Frontend: Resolve page UI** — Resolved indicators, Abort/Complete buttons
9. **Verification** — pnpm check + cargo check + cargo test

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| git2 index operations may not fully clean conflict state | Medium | Test with real merge conflicts; verify index has no conflict entries after resolve |
| Merge commit with git2 may miss hooks/signing | Low | Out of scope for V1; document as known limitation |
| File write race with watcher triggering refresh | Low | Debouncer should handle; test manually |

## Open Questions

None — scope is well-defined from parent requirements and codebase exploration.

---

## Metadata

**Parent:** bd-3gr
**Blocked by:** bd-3gr.1 (completed)
