# Push and Pull UX Integration

**Bead:** bd-9a4.2
**Type:** task
**Created:** 2026-03-16
**Status:** Draft → PRD

---

## Problem Statement

mongit has working backend commands for `fetch`, `pull`, and `push` (shipped in bd-2uj), but the UI provides no way to invoke them. Users must drop to the terminal for any remote sync operation. There is also no visibility into tracking status (ahead/behind counts), no progress feedback during network operations, and no structured error display for common remote failures (auth, network, diverged branches, conflicts).

## Scope

### In-Scope

1. **Sync store** — new `syncStore` managing fetch/pull/push state, errors, and results
2. **Toolbar sync buttons** — Fetch, Pull, Push buttons in the repo toolbar right section
3. **Ahead/behind tracking** — display commit counts relative to upstream in the toolbar
4. **Loading states** — disable buttons and show spinners during network operations
5. **Error display** — parse `BranchOpError` discriminated unions into user-friendly messages
6. **Success feedback** — brief inline status after successful operations
7. **Backend enhancement** — add `get_ahead_behind` IPC command returning tracking info
8. **Auto-refresh** — refresh repo status after sync operations complete

### Out-of-Scope

- Remote management UI (add/remove/configure remotes)
- Credential management or SSH key setup
- Pull with rebase option (future enhancement)
- Push to non-origin remotes
- Background auto-fetch polling
- Toast/notification system (use inline status in toolbar for now)
- Conflict resolution workflow (covered by separate merge editor bead)

## Proposed Solution

Add a `repo-toolbar-right` section to the existing toolbar with three action buttons (Fetch, Pull, Push) and an ahead/behind badge. Create a `syncStore` following the established `commitStore` pattern (Svelte 5 runes, `formatError()` for discriminated unions, loading flags). Add one new Rust IPC command (`get_ahead_behind`) that returns commit counts relative to upstream.

### Architecture

```
┌─ repo-toolbar ─────────────────────────────────────────────────┐
│ ← back │ repo-name │ branch  │         │ ↓2 ↑1 │ Fetch Pull Push │
│    repo-toolbar-left          │         │  ahead/behind  toolbar-right  │
└────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User clicks Push → syncStore.push(repoPath)
  → invoke('push', { path, force_with_lease: false })
  → Success: refresh repo status + show "Pushed" briefly
  → Error: parse BranchOpError JSON → show user-friendly message
```

## Requirements

### R1: Sync Store (`src/lib/stores/sync.svelte.ts`)

Create a new store mirroring the `commitStore` pattern:

| State | Type | Purpose |
|-------|------|----------|
| `fetching` | `boolean` | Loading flag for fetch operation |
| `pulling` | `boolean` | Loading flag for pull operation |
| `pushing` | `boolean` | Loading flag for push operation |
| `error` | `string \| null` | Current error message (parsed from BranchOpError) |
| `lastResult` | `SyncResult \| null` | Last successful operation result |
| `aheadBehind` | `{ ahead: number, behind: number } \| null` | Tracking counts |

Methods:
- `fetch(repoPath)` — invoke `fetch` command, refresh status on success
- `pull(repoPath)` — invoke `pull` command, refresh status on success
- `push(repoPath, forceWithLease?)` — invoke `push` command, refresh status on success
- `refreshAheadBehind(repoPath)` — invoke `get_ahead_behind`, update counts
- `clearError()` — reset error state
- `formatSyncError(e)` — parse `BranchOpError` JSON into user message

### R2: Ahead/Behind Backend Command

Add `get_ahead_behind` IPC command in `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub async fn get_ahead_behind(path: String) -> Result<AheadBehind, String>
```

Return type:
```rust
#[derive(Serialize)]
pub struct AheadBehind {
    ahead: u32,
    behind: u32,
    upstream: Option<String>,  // e.g., "origin/main"
}
```

Implementation: use `git rev-list --left-right --count HEAD...@{upstream}` via `run_git_async()`. Return `{ ahead: 0, behind: 0, upstream: None }` when no upstream is configured (not an error).

### R3: Toolbar UI (`src/routes/repo/+layout.svelte`)

Add `repo-toolbar-right` div containing:
1. **Ahead/behind badge** — shows `↓N ↑M` when tracking upstream, hidden when no upstream
2. **Fetch button** — icon + "Fetch" label, loading spinner during operation
3. **Pull button** — icon + "Pull" label, disabled when `behind === 0` and no error
4. **Push button** — icon + "Push" label, disabled when `ahead === 0` and no error

Button behavior:
- Disabled during any active sync operation (prevent concurrent ops)
- Show spinner on the active button
- Buttons use existing `Button.svelte` component with `variant="secondary"` and `size="sm"`
- All buttons in `no-drag` region (toolbar is a drag region for window movement)

### R4: Error Handling

Parse `BranchOpError` JSON (discriminated union via `kind` field) into user-friendly messages:

| Error Kind | User Message |
|------------|-------------|
| `NetworkError` | "Network error — check your connection" |
| `AuthFailure` | "Authentication failed — check credentials" |
| `MergeConflicts` | "Pull created merge conflicts in N file(s)" |
| `BranchesDiverged` | "Branches have diverged — pull first, then push" |
| `NoUpstreamBranch` | "No upstream branch configured" |
| `PushNonFastForward` | "Push rejected — pull first to integrate remote changes" |
| `ProtectedBranch` | "Remote rejected push to protected branch" |
| `RemoteNotFound` | "Remote 'origin' not found" |
| `GenericCommandFailed` | Show raw stderr |

Display error inline below the toolbar (dismissible banner), not in a modal.

### R5: Success Feedback

After a successful operation:
1. Clear any previous error
2. Show brief inline status text (e.g., "Fetched", "Pulled", "Pushed") for 3 seconds
3. Refresh ahead/behind counts
4. Refresh repo status (branch, changed_files, staged_files) to pick up pull changes

### R6: Auto-Refresh Integration

After pull completes successfully:
- Refresh `repoStore.repoStatus` (staged/changed counts may change)
- Refresh `changesStore` if on the Changes tab (new files from pull)
- Refresh ahead/behind counts

## Success Criteria

- [ ] Fetch button invokes `fetch` and shows loading state during operation
- [ ] Pull button invokes `pull` and refreshes workspace status on success
- [ ] Push button invokes `push` and refreshes ahead/behind on success
- [ ] Ahead/behind badge shows correct counts after each operation
- [ ] Network errors display user-friendly messages (not raw stderr)
- [ ] Auth failures display clear guidance message
- [ ] Merge conflicts from pull are reported with file count
- [ ] All buttons disabled during active sync operation
- [ ] No upstream branch is handled gracefully (badge hidden, push shows guidance)
- [ ] `pnpm check` passes with 0 errors
- [ ] `cargo check` passes in `src-tauri/`

## Technical Context

### Existing Backend (ready to use)
- `fetch(path)` → `branch::fetch_origin()` via `run_git_async()` — `commands.rs:165`
- `pull(path)` → `branch::pull_origin()` via `run_git_async()` — `commands.rs:175`
- `push(path, force_with_lease)` → `branch::push_origin()` — `commands.rs:186`
- `BranchOpError` enum with 15 variants, serialized as `{"kind": "...", ...}` — `error.rs:86-135`
- Error conversion `GitError::into() -> String` serializes as JSON — `error.rs:58-73`

### Existing Frontend Patterns
- `commitStore` (`stores/commit.svelte.ts`) — model for `syncStore` (runes, formatError, loading flags)
- `repoStore` (`stores/repo.svelte.ts`) — `openRequestId` race guard pattern, `activeRepoPath`
- `Button.svelte` (`components/ui/Button.svelte`) — `loading`, `disabled`, `variant`, `size` props
- `+layout.svelte` (`routes/repo/+layout.svelte`) — toolbar has `repo-toolbar-left`, needs `repo-toolbar-right`
- Design tokens: `--color-*`, `--space-*`, `--radius-*`, `--text-*`, `--transition-*`

### New Backend (to implement)
- `get_ahead_behind(path)` — `git rev-list --left-right --count HEAD...@{upstream}`
- `AheadBehind` struct: `{ ahead: u32, behind: u32, upstream: Option<String> }`
- Register in `lib.rs` invoke handler list

## Affected Files

| File | Change |
|------|--------|
| `src/lib/stores/sync.svelte.ts` | **NEW** — sync operation store |
| `src/routes/repo/+layout.svelte` | Add toolbar-right section with sync buttons |
| `src-tauri/src/commands.rs` | Add `get_ahead_behind` command |
| `src-tauri/src/lib.rs` | Register `get_ahead_behind` in invoke handler |
| `src-tauri/src/git/branch.rs` | Add `ahead_behind()` function (git rev-list) |

## Tasks

1. Add `ahead_behind()` function in `src-tauri/src/git/branch.rs`
2. Add `get_ahead_behind` IPC command in `commands.rs` + register in `lib.rs`
3. Create `syncStore` in `src/lib/stores/sync.svelte.ts`
4. Add toolbar-right section with Fetch/Pull/Push buttons in `+layout.svelte`
5. Add ahead/behind badge in toolbar
6. Add inline error banner below toolbar
7. Wire auto-refresh after sync operations
8. Verify: `pnpm check` + `cargo check`

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Network operations can hang | Medium | Backend uses `GIT_TERMINAL_PROMPT=0` to prevent credential hangs; frontend shows loading state |
| Concurrent sync operations | Low | Disable all sync buttons during any active operation |
| No upstream configured | Low | Graceful fallback: hide badge, show guidance on push attempt |
| Pull creates merge conflicts | Medium | Report conflict count; conflict resolution is out of scope (separate bead) |

## Open Questions

None — all patterns established by existing code.

---

## Metadata

**Parent:** bd-9a4
**Blocked by:** bd-9a4.1 (closed)
