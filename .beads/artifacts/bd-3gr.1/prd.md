# Conflict Detection and Resolver Entry Flow

**Bead:** bd-3gr.1
**Type:** task
**Parent:** bd-3gr (Conflict Resolution v1 with 3-Pane Baseline)
**Status:** PRD

---

## Problem Statement

When a `git pull` creates merge conflicts, mongit shows a toast error ("Pull created merge conflicts in N file(s)") and stops. The user has no way to see which files are conflicted, inspect the conflict content, or enter a resolver — they must drop to the terminal. The changes workspace doesn't distinguish conflicted files from normal modifications, and the existing MergeEditor component (`src/lib/components/MergeEditor.svelte`) is a hardcoded demo disconnected from real data.

This bead wires up the detection → discovery → entry pipeline: the backend queries to detect merge state and read conflict content, the UI indicators that make conflicts visible, and the navigation path into the resolver view with real data.

## Scope

### In-Scope

1. **Rust `conflict` module** — detect merge state, list conflicted files, read base/ours/theirs content via git2
2. **`Conflicted` variant in `FileChangeKind`** — distinguish conflicts from normal modifications in `changed_files()`
3. **Watcher enhancement** — watch `.git/MERGE_HEAD` to detect merge state changes
4. **Tauri IPC commands** — `get_merge_state`, `get_conflict_content`
5. **Conflict store** — Svelte 5 runes store managing merge state and selected conflict
6. **Conflict banner in changes workspace** — visible alert when merge is in progress, with entry point to resolver
7. **Conflict file indicators** — visual badge distinguishing conflicted files in the file list
8. **Resolver route** — `/repo/resolve` page listing conflicted files and showing MergeEditor with real content
9. **Wire MergeEditor** — connect to actual base/ours/theirs content (remove hardcoded SAMPLE_* data)

### Out-of-Scope

- Resolution actions (accept ours, accept theirs, manual edit) — bd-3gr.2
- Resolution completion flow (mark as resolved, stage, continue merge) — bd-3gr.2
- AI conflict assistance — bd-2fs
- Rebase conflict handling (different state files) — future
- Binary file conflict handling — future

## Proposed Solution

### Architecture

```
                     git2 (reads)
                         ↓
┌──────────────────────────────────────────┐
│  src-tauri/src/git/conflict.rs           │
│  ┌─────────────────┐  ┌───────────────┐  │
│  │ get_merge_state  │  │ conflict_     │  │
│  │ (MERGE_HEAD,     │  │ content       │  │
│  │  index stages)   │  │ (base/ours/   │  │
│  │                  │  │  theirs)      │  │
│  └────────┬─────────┘  └──────┬────────┘  │
└───────────┼────────────────────┼──────────┘
            │ Tauri IPC          │
┌───────────┼────────────────────┼──────────┐
│  Frontend │                    │          │
│  ┌────────▼─────────┐  ┌──────▼────────┐ │
│  │ conflictStore     │  │ MergeEditor   │ │
│  │ (merge state,     │  │ (real data)   │ │
│  │  conflict list)   │  │              │  │
│  └────────┬─────────┘  └──────────────┘  │
│           │                              │
│  ┌────────▼──────────────────────────┐   │
│  │ Changes page → conflict banner    │   │
│  │ Conflict badge in file list       │   │
│  │ "Resolve Conflicts" button        │   │
│  └───────────────────────────────────┘   │
│           │                              │
│  ┌────────▼──────────────────────────┐   │
│  │ /repo/resolve route               │   │
│  │ Conflict file list + MergeEditor  │   │
│  └───────────────────────────────────┘   │
└──────────────────────────────────────────┘
```

### Backend Design

**New file: `src-tauri/src/git/conflict.rs`**

Uses git2 read path (not CLI) since conflict detection is a read operation:

```rust
/// Check if the repository is in a merge state.
pub fn get_merge_state(path: &Path) -> Result<MergeState, GitError>
```
- Checks for `.git/MERGE_HEAD` file existence
- Reads MERGE_HEAD to get the incoming commit SHA
- Uses `repo.index()` to enumerate entries with stage > 0 (conflicted)
- Returns `MergeState { is_merging, incoming_sha, conflicted_files }`

```rust
/// Read the base, ours, and theirs content for a conflicted file.
pub fn get_conflict_content(path: &Path, file_path: &str) -> Result<ConflictContent, GitError>
```
- Uses `repo.index()?.conflicts()` iterator to find the file
- Reads blob content for each stage (1=base, 2=ours, 3=theirs)
- Returns `ConflictContent { base, ours, theirs, file_path }`

**Types:**

```rust
#[derive(Debug, Clone, Serialize)]
pub struct MergeState {
    pub is_merging: bool,
    pub incoming_sha: Option<String>,
    pub conflicted_files: Vec<ConflictFileEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictFileEntry {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictContent {
    pub file_path: String,
    pub base: Option<String>,    // None for new files
    pub ours: String,
    pub theirs: String,
}
```

**`FileChangeKind` extension:**

Add `Conflicted` variant to `src-tauri/src/git/repository.rs`. Update `changed_files()` to check `Status::CONFLICTED` before other status flags.

**Watcher enhancement:**

Add `.git/MERGE_HEAD` to `should_emit_for_path()` in `src-tauri/src/watcher.rs`. This ensures the frontend refreshes when merge state changes (after pull creates conflicts, or after all conflicts are resolved).

### Frontend Design

**New store: `src/lib/stores/conflict.svelte.ts`**

```typescript
// State
mergeState: MergeState | null  // from get_merge_state
selectedFile: string | null     // currently selected conflict
content: ConflictContent | null // base/ours/theirs for selected file
loading: boolean
error: string | null

// Methods
loadMergeState(repoPath)       // fetch merge state
selectConflict(filePath)       // select + load content
refresh()                      // re-check merge state
```

**Changes workspace modifications (`src/routes/repo/changes/+page.svelte`):**

1. On mount (and on `repo-changed` event), call `conflictStore.loadMergeState()`
2. When `mergeState.is_merging`, show a conflict banner above the split layout:
   ```
   ⚠ Merge in progress — N file(s) have conflicts
   [Resolve Conflicts]
   ```
3. In the file list, files with `staged === 'Conflicted'` or `unstaged === 'Conflicted'` get a distinct badge (e.g., red "C" or "!" icon)

**New route: `src/routes/repo/resolve/+page.svelte`**

- Left panel: list of conflicted files with resolved/unresolved status
- Right panel: MergeEditor component wired to real content
- File selection loads content via `get_conflict_content`

**MergeEditor wiring (`src/lib/components/MergeEditor.svelte`):**

- Accept props: `base`, `ours`, `theirs`, `filePath` instead of hardcoded SAMPLE_* data
- Remove hardcoded sample content
- Keep Accept Ours / Accept Theirs buttons in place (they'll be wired in bd-3gr.2)

## Requirements

### Functional

1. **FR-1:** Backend detects merge state by checking `.git/MERGE_HEAD` existence and index conflict entries
2. **FR-2:** `get_merge_state` returns the list of conflicted file paths
3. **FR-3:** `get_conflict_content` returns base/ours/theirs text for a given conflicted file
4. **FR-4:** `changed_files()` reports conflicted files with `FileChangeKind::Conflicted` instead of `Modified`
5. **FR-5:** File watcher detects `.git/MERGE_HEAD` creation/deletion and emits `repo-changed`
6. **FR-6:** Changes workspace shows a conflict banner when merge is in progress
7. **FR-7:** Conflicted files in the file list have a distinct visual badge
8. **FR-8:** "Resolve Conflicts" button navigates to `/repo/resolve`
9. **FR-9:** Resolver page lists conflicted files and shows MergeEditor with real content
10. **FR-10:** MergeEditor receives actual base/ours/theirs content via props

### Non-Functional

1. **NFR-1:** Merge state detection completes in <100ms (git2 index read, no shell)
2. **NFR-2:** Conflict content loading completes in <200ms per file
3. **NFR-3:** All new Rust code has unit tests

## Success Criteria

- [ ] `get_merge_state` correctly detects merge state and lists conflicted files
- [ ] `get_conflict_content` returns base/ours/theirs text content
- [ ] `changed_files()` returns `Conflicted` for files with unresolved conflicts
- [ ] Watcher detects `.git/MERGE_HEAD` changes
- [ ] Conflict banner appears in changes workspace during merge
- [ ] Conflicted files have distinct visual badge in file list
- [ ] "Resolve Conflicts" button navigates to resolver page
- [ ] MergeEditor displays real conflict content (no hardcoded samples)
- [ ] `pnpm check` passes with 0 errors
- [ ] `cargo check` and `cargo test` pass
- [ ] No regressions in existing staging/commit/changes functionality

## Technical Context

### Existing Patterns to Follow

| Pattern | Location | Description |
|---------|----------|-------------|
| `FileChangeKind` enum | `src-tauri/src/git/repository.rs:28-34` | Add `Conflicted` variant |
| `changed_files()` status flags | `src-tauri/src/git/repository.rs:234-288` | Add `Status::CONFLICTED` check |
| Module structure | `src-tauri/src/git/commit.rs` | Standalone functions with `map_cli_error()` |
| Error patterns | `src-tauri/src/git/error.rs` | Discriminated unions with `#[serde(tag = "kind")]` |
| Tauri commands | `src-tauri/src/commands.rs` | `spawn_blocking`, `Result<T, String>` |
| Svelte 5 store | `src/lib/stores/commit.svelte.ts` | Runes state object with methods |
| Watcher path filter | `src-tauri/src/watcher.rs:62-94` | `should_emit_for_path()` |
| MergeEditor component | `src/lib/components/MergeEditor.svelte` | CodeMirror MergeView (demo state) |

### Files to Create

| File | Purpose |
|------|---------|
| `src-tauri/src/git/conflict.rs` | Merge state detection + conflict content reading |
| `src/lib/stores/conflict.svelte.ts` | Conflict state management |
| `src/routes/repo/resolve/+page.svelte` | Resolver entry page |

### Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/git/mod.rs` | Add `pub mod conflict;` |
| `src-tauri/src/git/repository.rs` | Add `Conflicted` to `FileChangeKind`, update `changed_files()` |
| `src-tauri/src/commands.rs` | Add `get_merge_state` and `get_conflict_content` commands |
| `src-tauri/src/lib.rs` | Register new commands |
| `src-tauri/src/watcher.rs` | Add `.git/MERGE_HEAD` to watched paths |
| `src/routes/repo/changes/+page.svelte` | Add conflict banner, conflict badge styling |
| `src/lib/components/MergeEditor.svelte` | Accept props instead of hardcoded data |
| `src/routes/repo/+layout.svelte` | Add `/repo/resolve` to navigation (if applicable) |

## Tasks

1. **T1: Conflict types + detection module** — Create `conflict.rs` with `MergeState`, `ConflictContent`, `get_merge_state()`, `get_conflict_content()`
2. **T2: FileChangeKind::Conflicted** — Add variant, update `changed_files()` to detect conflicts
3. **T3: Watcher MERGE_HEAD** — Add `.git/MERGE_HEAD` to `should_emit_for_path()`
4. **T4: Tauri commands** — Add `get_merge_state` and `get_conflict_content`, register in `lib.rs`
5. **T5: Conflict store** — Create `conflict.svelte.ts` with merge state, selected file, content
6. **T6: Changes page integration** — Conflict banner, conflict badge, "Resolve Conflicts" button
7. **T7: Resolver route** — Create `/repo/resolve` with conflict file list + MergeEditor
8. **T8: Wire MergeEditor** — Replace hardcoded samples with props-based content
9. **T9: Verification** — `pnpm check`, `cargo check`, `cargo test`

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| git2 conflict iterator may not work for all merge types | Medium | Test with real merge conflicts in integration tests |
| Binary files in conflicts | Low | Return error for non-UTF-8 content, defer binary handling |
| Rebase conflicts use different state files (REBASE_HEAD) | Low | Out of scope; document for future |
| MergeEditor refactor breaks existing demo | Low | Demo was never user-facing; clean replacement |

## Open Questions

None — the git2 API for conflicts is well-documented and the existing patterns are clear.

---

## Metadata

**Parent:** bd-3gr
**Dependencies:** bd-20d (CLOSED), bd-htm (CLOSED), bd-9a4 (CLOSED)
**Dependents:** bd-3gr.2 (Three-pane editor actions and resolution completion — blocked by this)
