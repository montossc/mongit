# Implementation Plan: bd-3gr.1 — Conflict Detection and Resolver Entry Flow

## Overview

9 tasks across 4 waves. All work in worktree `/Users/montossc/.opencode/worktrees/mongit/bd-3gr.1-conflict-detection`.

## Wave 1: Backend Infrastructure (T1 + T2 + T3 — parallel)

### T1: Conflict types + detection module

**File: `src-tauri/src/git/conflict.rs` (new)**

Create the conflict detection module using git2 read path:

```rust
use std::path::Path;
use serde::Serialize;
use super::GitError;

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
    pub base: Option<String>,
    pub ours: String,
    pub theirs: String,
}

/// Detect merge state by checking MERGE_HEAD and index conflicts.
pub fn get_merge_state(path: &Path) -> Result<MergeState, GitError> {
    let repo = git2::Repository::open(path)?;
    let merge_head_path = path.join(".git/MERGE_HEAD");
    let is_merging = merge_head_path.exists();
    
    let incoming_sha = if is_merging {
        std::fs::read_to_string(&merge_head_path)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    };
    
    let mut conflicted_files = Vec::new();
    if is_merging {
        let index = repo.index()?;
        if index.has_conflicts() {
            let conflicts = index.conflicts()?;
            let mut seen = std::collections::HashSet::new();
            for conflict in conflicts {
                let conflict = conflict?;
                // Get path from whichever entry exists (ours, theirs, or ancestor)
                let path_str = conflict.our
                    .as_ref()
                    .or(conflict.their.as_ref())
                    .or(conflict.ancestor.as_ref())
                    .map(|e| String::from_utf8_lossy(&e.path).to_string())
                    .unwrap_or_default();
                if !path_str.is_empty() && seen.insert(path_str.clone()) {
                    conflicted_files.push(ConflictFileEntry { path: path_str });
                }
            }
        }
    }
    
    Ok(MergeState { is_merging, incoming_sha, conflicted_files })
}

/// Read base/ours/theirs content for a conflicted file.
pub fn get_conflict_content(path: &Path, file_path: &str) -> Result<ConflictContent, GitError> {
    let repo = git2::Repository::open(path)?;
    let index = repo.index()?;
    
    if !index.has_conflicts() {
        return Err(GitError::NotFound("No merge conflicts in index".into()));
    }
    
    let conflicts = index.conflicts()?;
    for conflict in conflicts {
        let conflict = conflict?;
        let conflict_path = conflict.our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
            .map(|e| String::from_utf8_lossy(&e.path).to_string())
            .unwrap_or_default();
        
        if conflict_path == file_path {
            let base = conflict.ancestor.as_ref()
                .and_then(|e| repo.find_blob(e.id).ok())
                .and_then(|b| String::from_utf8(b.content().to_vec()).ok());
            
            let ours = conflict.our.as_ref()
                .and_then(|e| repo.find_blob(e.id).ok())
                .map(|b| String::from_utf8_lossy(b.content()).to_string())
                .unwrap_or_default();
            
            let theirs = conflict.their.as_ref()
                .and_then(|e| repo.find_blob(e.id).ok())
                .map(|b| String::from_utf8_lossy(b.content()).to_string())
                .unwrap_or_default();
            
            return Ok(ConflictContent { file_path: file_path.to_string(), base, ours, theirs });
        }
    }
    
    Err(GitError::NotFound(format!("File '{}' not found in conflicts", file_path)))
}
```

Add `pub mod conflict;` to `src-tauri/src/git/mod.rs` (after `pub mod commit;`).

Tests: merge state detection with/without MERGE_HEAD, conflict content reading.

### T2: FileChangeKind::Conflicted

**File: `src-tauri/src/git/repository.rs`**

1. Add `Conflicted` variant to `FileChangeKind` enum (line 33, before closing `}`):
   ```rust
   Conflicted,
   ```

2. In `changed_files()` (line 248), add conflict check BEFORE the existing staged/unstaged checks:
   ```rust
   // Check for conflicts first — conflicted files get special treatment
   if bits.intersects(git2::Status::CONFLICTED) {
       entries.push(ChangedFileEntry {
           path,
           staged: Some(FileChangeKind::Conflicted),
           unstaged: Some(FileChangeKind::Conflicted),
       });
       continue;
   }
   ```

3. Update frontend `FileChangeKind` type in `src/lib/stores/changes.svelte.ts` to add `"Conflicted"`.

4. Update `kindLabel()` and `kindClass()` in `src/routes/repo/changes/+page.svelte`:
   ```typescript
   case 'Conflicted': return '!';
   case 'Conflicted': return 'conflicted';
   ```

5. Add `.status-badge.staged.conflicted` and `.status-badge.unstaged.conflicted` CSS.

### T3: Watcher MERGE_HEAD

**File: `src-tauri/src/watcher.rs`**

The watcher already allows all `.git/` paths except `.git/objects/` and `.git/logs/` (line 83-87). Since `.git/MERGE_HEAD` passes through the existing filter (it's a `.git/` path with `next == "MERGE_HEAD"`, not "objects" or "logs"), no code change is needed — it already works.

**Verify with test:** Add a test confirming `.git/MERGE_HEAD` emits:
```rust
#[test]
fn test_should_emit_git_merge_head() {
    assert!(should_emit_for_path(&PathBuf::from("/repo/.git/MERGE_HEAD")));
}
```

## Wave 2: Tauri Commands (T4)

### T4: IPC commands + registration

**File: `src-tauri/src/commands.rs`**

Add imports:
```rust
use crate::git::conflict;
use crate::git::conflict::{MergeState, ConflictContent};
```

Add commands:
```rust
/// Get the merge state of a repository (is merging, conflicted files).
#[tauri::command]
pub async fn get_merge_state(path: String) -> Result<MergeState, String> {
    tokio::task::spawn_blocking(move || {
        let path = PathBuf::from(path);
        conflict::get_merge_state(&path).map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Get the base/ours/theirs content for a conflicted file.
#[tauri::command]
pub async fn get_conflict_content(
    path: String,
    file_path: String,
) -> Result<ConflictContent, String> {
    tokio::task::spawn_blocking(move || {
        let path = PathBuf::from(path);
        conflict::get_conflict_content(&path, &file_path).map_err(String::from)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
```

**File: `src-tauri/src/lib.rs`**

Register new commands in `invoke_handler`:
```rust
commands::get_merge_state,
commands::get_conflict_content,
```

## Wave 3: Frontend (T5 + T6 + T7 + T8 — mostly parallel)

### T5: Conflict store

**File: `src/lib/stores/conflict.svelte.ts` (new)**

```typescript
import { invoke } from "@tauri-apps/api/core";

export interface ConflictFileEntry { path: string; }
export interface MergeState {
    is_merging: boolean;
    incoming_sha: string | null;
    conflicted_files: ConflictFileEntry[];
}
export interface ConflictContent {
    file_path: string;
    base: string | null;
    ours: string;
    theirs: string;
}

function createConflictStore() {
    let mergeState = $state<MergeState | null>(null);
    let selectedFile = $state<string | null>(null);
    let content = $state<ConflictContent | null>(null);
    let loading = $state(false);
    let contentLoading = $state(false);
    let error = $state<string | null>(null);

    async function loadMergeState(repoPath: string) {
        loading = true;
        error = null;
        try {
            mergeState = await invoke<MergeState>("get_merge_state", { path: repoPath });
        } catch (e) {
            error = String(e);
            mergeState = null;
        } finally {
            loading = false;
        }
    }

    async function selectConflict(repoPath: string, filePath: string) {
        selectedFile = filePath;
        contentLoading = true;
        error = null;
        try {
            content = await invoke<ConflictContent>("get_conflict_content", {
                path: repoPath,
                filePath,
            });
        } catch (e) {
            error = String(e);
            content = null;
        } finally {
            contentLoading = false;
        }
    }

    async function refresh(repoPath: string) {
        await loadMergeState(repoPath);
    }

    function reset() {
        mergeState = null;
        selectedFile = null;
        content = null;
        loading = false;
        contentLoading = false;
        error = null;
    }

    return {
        get mergeState() { return mergeState; },
        get selectedFile() { return selectedFile; },
        get content() { return content; },
        get loading() { return loading; },
        get contentLoading() { return contentLoading; },
        get error() { return error; },
        get isMerging() { return mergeState?.is_merging ?? false; },
        get conflictCount() { return mergeState?.conflicted_files.length ?? 0; },
        loadMergeState,
        selectConflict,
        refresh,
        reset,
    };
}

export const conflictStore = createConflictStore();
```

### T6: Changes page integration

**File: `src/routes/repo/changes/+page.svelte`**

1. Import `conflictStore`:
   ```typescript
   import { conflictStore } from '$lib/stores/conflict.svelte';
   ```

2. In `onMount`, load merge state:
   ```typescript
   if (repoStore.activeRepoPath) {
       conflictStore.loadMergeState(repoStore.activeRepoPath);
   }
   ```

3. In `repo-changed` listener, refresh conflict state:
   ```typescript
   listen('repo-changed', () => {
       changesStore.refresh();
       diffStore.refresh();
       if (repoStore.activeRepoPath) {
           conflictStore.loadMergeState(repoStore.activeRepoPath);
       }
   });
   ```

4. Add conflict banner just inside the `{:else}` branch (before `<div class="split-layout">`):
   ```svelte
   {#if conflictStore.isMerging}
       <div class="conflict-banner">
           <span class="conflict-banner-icon">⚠</span>
           <span>Merge in progress — {conflictStore.conflictCount} file{conflictStore.conflictCount !== 1 ? 's' : ''} have conflicts</span>
           <button class="conflict-resolve-btn" onclick={() => goto('/repo/resolve')}>
               Resolve Conflicts
           </button>
       </div>
   {/if}
   ```

5. Import `goto`:
   ```typescript
   import { goto } from '$app/navigation';
   ```

6. Add `Conflicted` handling to `kindLabel()` and `kindClass()`.

7. Add CSS for conflict banner + conflicted badge:
   ```css
   .conflict-banner { ... }
   .status-badge.staged.conflicted { background: var(--color-danger); color: white; }
   .status-badge.unstaged.conflicted { border: 1px solid var(--color-danger); color: var(--color-danger); }
   ```

### T7: Resolver route

**File: `src/routes/repo/resolve/+page.svelte` (new)**

Create a resolver page with:
- Left panel: list of conflicted files
- Right panel: MergeEditor component with real content
- File selection loads content via conflictStore

```svelte
<script lang="ts">
    import { onMount } from 'svelte';
    import { repoStore } from '$lib/stores/repo.svelte';
    import { conflictStore } from '$lib/stores/conflict.svelte';
    import MergeEditor from '$lib/components/MergeEditor.svelte';

    onMount(() => {
        if (repoStore.activeRepoPath) {
            conflictStore.loadMergeState(repoStore.activeRepoPath);
        }
    });

    function selectFile(path: string) {
        if (repoStore.activeRepoPath) {
            conflictStore.selectConflict(repoStore.activeRepoPath, path);
        }
    }
</script>

<!-- conflict file list + MergeEditor -->
```

Add "Resolve" tab to `src/routes/repo/+layout.svelte` tabs array (conditionally shown when merging).

### T8: Wire MergeEditor

**File: `src/lib/components/MergeEditor.svelte`**

1. Remove hardcoded SAMPLE_BASE, SAMPLE_OURS, SAMPLE_THEIRS constants
2. Change props defaults to empty strings:
   ```typescript
   let {
       base = '',
       ours = '',
       theirs = ''
   }: { base?: string; ours?: string; theirs?: string } = $props();
   ```
3. Component is already prop-driven (lines 100-104 use $props()), just need to remove sample data and update defaults

## Wave 4: Verification (T9)

### T9: Full verification

1. `cargo check` in src-tauri/
2. `cargo test conflict` — run conflict-specific tests
3. `pnpm check` — svelte-check 0 errors
4. Verify no regressions in staging/commit/changes functionality

## Dependency Graph

```
Wave 1:  T1 (conflict.rs)  ──┐
         T2 (Conflicted)   ──┼── Wave 2: T4 (commands) ──┐
         T3 (watcher test) ──┘                            │
                                                          │
Wave 3:  T5 (store) ──────────────────────────────────────┤
         T6 (changes page) ───────────────────────────────┤
         T7 (resolver route) ─────────────────────────────┤
         T8 (MergeEditor) ────────────────────────────────┘
                                                          │
Wave 4:  T9 (verification) ───────────────────────────────┘
```

## Files Changed Summary

| File | Action |
|------|--------|
| `src-tauri/src/git/conflict.rs` | CREATE |
| `src-tauri/src/git/mod.rs` | MODIFY (add `pub mod conflict;`) |
| `src-tauri/src/git/repository.rs` | MODIFY (add `Conflicted` variant + check) |
| `src-tauri/src/commands.rs` | MODIFY (2 new commands) |
| `src-tauri/src/lib.rs` | MODIFY (register commands) |
| `src-tauri/src/watcher.rs` | MODIFY (add test only) |
| `src/lib/stores/conflict.svelte.ts` | CREATE |
| `src/lib/stores/changes.svelte.ts` | MODIFY (add `Conflicted` type) |
| `src/routes/repo/changes/+page.svelte` | MODIFY (banner + badge) |
| `src/routes/repo/resolve/+page.svelte` | CREATE |
| `src/routes/repo/+layout.svelte` | MODIFY (conditional Resolve tab) |
| `src/lib/components/MergeEditor.svelte` | MODIFY (remove samples) |
