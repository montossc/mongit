# Technical Subsystems & Risk Map

**Date:** March 13, 2026
**Purpose:** Map the technical subsystems required to build a JetBrains-quality standalone Git client, with complexity/risk ratings to guide implementation prioritization.

---

## Subsystem Inventory

### 1. Git Process Wrapper / Library Layer
**Complexity:** MEDIUM | **Risk:** LOW

Hybrid approach: libgit2 (via `git2` Rust crate) for read/query operations + bundled Git binary for write/complex operations.

- Manages async git process execution for mutations
- Parses structured output (porcelain v2, NUL-delimited)
- Exposes progress callbacks for long operations
- Handles encoding edge cases, cancel/abort mid-operation
- libgit2 for: status, diff, blame, log, index manipulation, conflict data
- Bundled git for: commit (hooks), rebase, push, fetch, merge, cherry-pick, signing

### 2. Change Tracking Engine (File-system Watcher → Index Differ)
**Complexity:** HIGH | **Risk:** MEDIUM

Continuous background computation of which files/lines changed vs HEAD.

- Incremental (not re-diffing whole tree on every change)
- Debounced file system watching
- Consistent with editor buffer state (unsaved changes appear immediately)
- Tracks staged vs unstaged at hunk/line level
- Must reconcile against external git operations (terminal `git add`)

### 3. Diff Computation Engine
**Complexity:** MEDIUM | **Risk:** LOW

Full Myers-diff at line, word, and character granularity.

- Ignore-whitespace modes
- Detect-move-within-file (for blame)
- Three-way merge diff
- Binary file detection (limited display)
- Performance: real-time diff preview as files are modified
- Can leverage libgit2's diff engine for most operations

### 4. Commit Graph Renderer
**Complexity:** HIGH | **Risk:** HIGH

Renders a dense, scrollable, interactive commit topology graph.

- Must handle: 50+ active branches, octopus merges, 100k+ commits
- Linear branch collapse/expand without layout shifts
- Multi-repo interleaved commits (color-coded) — V2
- Branch label positioning (avoid overlap)
- Jump-arrow rendering for discontinuous traversal
- IntelliSort-style merge display
- WebGL/Canvas rendering in Tauri webview

**Risk notes:** Most open-source graph implementations have visual glitches. Budget significant engineering time. Consider libraries: `d3-dag`, custom WebGL shader, or port GitKraken's approach.

### 5. Changelist / Work Bucket Manager
**Complexity:** MEDIUM-HIGH | **Risk:** MEDIUM

IDEA-style named staging areas, reinterpreted for a standalone client.

- Track file → work bucket assignments
- Persist across sessions (stored in app data, not .git)
- Handle moves between buckets
- Map to Git staging area on commit
- Reconcile against external git operations

### 6. Shelf / Stash Manager
**Complexity:** MEDIUM | **Risk:** LOW

Persistent work-in-progress storage.

- Generate patches from work buckets
- Apply patches with 3-way merge fallback
- Handle conflicts on unshelve
- Track applied/unapplied state
- Import external .patch files
- Leverage git stash for git-native operations

### 7. Blame / Annotation Engine
**Complexity:** MEDIUM | **Risk:** MEDIUM

Asynchronous git blame with navigation.

- Run `git blame` via libgit2 or subprocess
- Cache results per file+revision
- Map byte offsets to line numbers (accounting for unsaved changes)
- Support revision navigation (Annotate Previous Revision)
- "Hide Revision" concept (`--ignore-rev` equivalent)
- Progressive results display for large files

**Risk notes:** Performance on 10k+ line files. Must be async with cancellation.

### 8. Log Index / Search Engine
**Complexity:** MEDIUM | **Risk:** LOW

Fast commit history search.

- In-process index of commit metadata (message, author, date, files)
- Prefix-search for branch names
- Full-text search for commit messages
- Keep current as new commits arrive (fetch, local commits)
- Leverage libgit2's revwalk for initial population

### 9. Workspace Context Serializer
**Complexity:** LOW | **Risk:** LOW

Per-branch state save/restore.

- Serialize: open panels, scroll positions, selected files, UI state
- Store keyed by branch name in app data
- Restore on branch checkout
- Handle: file no longer exists on checked-out branch

### 10. Conflict Detection & Merge Orchestrator
**Complexity:** HIGH | **Risk:** MEDIUM

Intercepts git operations and manages conflict resolution flow.

- Captures conflicted file list from failed merge/rebase/cherry-pick
- Presents conflict data in 3-pane merge editor
- Tracks resolution state per file
- Enables "Continue" action when all conflicts resolved
- Uses libgit2's `git_index_conflict_iterator` for structured conflict data
- Delegates actual merge/rebase operations to bundled git binary

### 11. Pre-Commit Check Pipeline
**Complexity:** MEDIUM | **Risk:** LOW

Runs checks before commit, reports results inline.

- Invokes user's `.git/hooks/pre-commit` via bundled git
- Optional app-native checks (formatting, TODO scan)
- Async execution with progress
- Reports results inline in commit panel
- Supports abort/override

---

## Risk Heat Map

| Subsystem | Complexity | Risk | Priority | Notes |
|-----------|-----------|------|----------|-------|
| Commit Graph Renderer | HIGH | HIGH | P0 — MVP | Core UX surface. Budget 4-6 weeks. |
| Change Tracking Engine | HIGH | MEDIUM | P0 — MVP | Foundation for all other features. |
| Conflict / Merge Orchestrator | HIGH | MEDIUM | P0 — MVP | 3-pane merge is key differentiator. |
| Changelist / Work Bucket Manager | MEDIUM-HIGH | MEDIUM | P1 — V1.1 | Differentiator but can ship without. |
| Blame / Annotation Engine | MEDIUM | MEDIUM | P1 — V1.1 | Important but not blocking MVP. |
| Diff Computation Engine | MEDIUM | LOW | P0 — MVP | Well-understood, libraries available. |
| Git Process Wrapper | MEDIUM | LOW | P0 — MVP | Hybrid pattern is proven. |
| Log Index / Search | MEDIUM | LOW | P1 — V1.1 | Nice-to-have for MVP. |
| Pre-Commit Pipeline | MEDIUM | LOW | P1 — V1.1 | Important for correctness users. |
| Shelf / Stash Manager | MEDIUM | LOW | P1 — V1.1 | Leverage git stash initially. |
| Workspace Context Serializer | LOW | LOW | P2 — V2 | Invisible retention feature. |

---

## Key Technical Decisions

1. **Graph rendering in WebGL/Canvas** — not DOM. DOM-based graphs break at 1000+ nodes.
2. **libgit2 for reads, bundled git for writes** — proven hybrid pattern.
3. **File system watcher with debouncing** — not polling. macOS FSEvents API via Tauri.
4. **CodeMirror 6 for diff/merge views** — battle-tested virtual scrolling for large files.
5. **All git mutations on a serial queue** — libgit2 objects are not thread-safe for writes.
6. **App data stored outside .git** — work buckets, shelf, UI state in ~/Library/Application Support/.
