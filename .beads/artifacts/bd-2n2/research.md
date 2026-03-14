# Research: bd-2n2 — CodeMirror 6 Diff/Merge + File Watching

**Date**: 2026-03-14  
**Depth**: Moderate (~30 tool calls)  
**Confidence**: High on all critical APIs

---

## Questions & Answers

### Q1: What CM6 npm packages are required?
**Status**: ✅ Answered — High confidence  
**Source**: https://codemirror.net/docs/ref

**Required packages:**
```bash
pnpm add @codemirror/view @codemirror/state @codemirror/merge \
         @codemirror/language @codemirror/commands \
         @codemirror/lang-javascript @codemirror/lang-rust
```

Minimum for the spike:
- `@codemirror/view` — EditorView, keymap
- `@codemirror/state` — EditorState, Compartment, Extension
- `@codemirror/merge` — MergeView, acceptChunk, rejectChunk, unifiedMergeView
- `@codemirror/language` — defaultHighlightStyle, Language support base
- `@codemirror/commands` — defaultKeymap
- `@codemirror/lang-javascript` — JavaScript/TypeScript syntax
- `@codemirror/lang-rust` — Rust syntax (optional for benchmark panel)

Optional for theme (can use CSS vars instead):
- `@codemirror/theme-one-dark` — if you want a quick dark theme base

---

### Q2: What is the exact MergeView (side-by-side) API?
**Status**: ✅ Answered — High confidence  
**Source**: https://codemirror.net/docs/ref/#merge.MergeView

```typescript
import { MergeView, acceptChunk, rejectChunk } from '@codemirror/merge';
import { EditorState } from '@codemirror/state';

// DirectMergeConfig extends MergeConfig
const mv = new MergeView({
  // Per-editor configs (EditorStateConfig)
  a: {
    doc: originalText,
    extensions: [EditorState.readOnly.of(true), /* ... */],
  },
  b: {
    doc: modifiedText,
    extensions: [EditorState.readOnly.of(true), /* ... */],
  },
  // MergeConfig options
  parent: containerDiv,          // DOM element to mount into
  gutter: true,                  // show gutter markers
  highlightChanges: true,        // mark inserted/deleted text
  revertControls: 'a-to-b',     // built-in revert buttons (a→b direction)
  collapseUnchanged: { margin: 3, minSize: 8 },
});

// Properties
mv.a           // EditorView (left/original)
mv.b           // EditorView (right/modified)
mv.dom         // HTMLElement — the outer container
mv.chunks      // readonly Chunk[] — current diff chunks
mv.reconfigure({ gutter: false })  // dynamic config update
mv.destroy()   // cleanup — call this on unmount!

// Accept/reject chunk at cursor or given position
acceptChunk(mv.b, pos?)   // accepts from a into b
rejectChunk(mv.b, pos?)   // reverts b chunk back to a's content
```

**Critical styling**: `.cm-mergeView` needs `height` + `overflow: auto` to be scrollable.

---

### Q3: How does the Svelte 5 action pattern work for CM6?
**Status**: ✅ Answered — High confidence  
**Source**: CodeMirror docs (action = clean lifecycle for imperative DOM)

```typescript
// src/lib/actions/codemirror.ts
import type { EditorView, EditorViewConfig } from '@codemirror/view';
import type { Action } from 'svelte/action';

export interface CMActionOptions extends EditorViewConfig {
  // Custom options for reactive updates
}

export const codemirror: Action<HTMLElement, CMActionOptions> = (node, options) => {
  let view: EditorView;
  
  // Lazily import to avoid SSR issues (SvelteKit static adapter)
  import('@codemirror/view').then(({ EditorView }) => {
    view = new EditorView({ ...options, parent: node });
  });

  return {
    update(newOptions) {
      // Use Compartment.reconfigure() — never recreate EditorView
      // view.dispatch({ effects: someCompartment.reconfigure(newExt) });
    },
    destroy() {
      view?.destroy();
    }
  };
};
```

**For MergeView** — use a dedicated action or manage in component `onMount`/`onDestroy` since MergeView is not EditorView directly:

```typescript
// In component:
import { onMount, onDestroy } from 'svelte';
import { MergeView } from '@codemirror/merge';

let container: HTMLElement;
let mv: MergeView;

onMount(() => {
  mv = new MergeView({ a: {...}, b: {...}, parent: container, ... });
});

onDestroy(() => {
  mv?.destroy();
});
```

---

### Q4: How does `notify-debouncer-full` 0.5 API work?
**Status**: ✅ Answered — High confidence  
**Source**: https://docs.rs/notify-debouncer-full/0.5.0/

```rust
use notify_debouncer_full::{
    notify::{RecursiveMode, RecommendedWatcher},
    new_debouncer, DebounceEventResult, Debouncer, RecommendedCache,
};
use std::time::Duration;

type WatcherHandle = Debouncer<RecommendedWatcher, RecommendedCache>;

// Create debouncer (callback fires after debounce period)
let mut debouncer = new_debouncer(
    Duration::from_millis(300),
    None,  // tick_rate; None = use debounce duration
    |result: DebounceEventResult| {
        match result {
            Ok(events) => {
                for event in events {
                    // event.paths: Vec<PathBuf>
                    // event.kind: EventKind
                    // event derefs to notify::Event
                }
            }
            Err(errors) => { /* handle errors */ }
        }
    }
).unwrap();

debouncer.watch("/path/to/repo", RecursiveMode::Recursive).unwrap();
// To stop: drop debouncer OR debouncer.stop()
// To replace path: just call watch() with new path; unwatch() old path first if needed
```

**CRITICAL CORRECTION for PRD**: `Debouncer<T, C>` IS `Send + Sync` in v0.5.0 when `T: Send, C: Send`. The PRD note saying "Debouncer is not Send" is **incorrect**. Confirmed by auto trait implementations in docs.

**Correct Tauri State type**: `Mutex<Option<WatcherHandle>>` (not `Arc<Mutex<>>`; Tauri `State<T>` internally uses Arc).

---

### Q5: How do Tauri commands emit events to the frontend?
**Status**: ✅ Answered — High confidence  
**Source**: https://tauri.app/develop/calling-frontend/

```rust
// In Cargo.toml: tauri features already include Emitter
use tauri::{AppHandle, Emitter};

// AppHandle is Clone — capture by value in the watcher callback
#[tauri::command]
pub async fn watch_repo(
    app: AppHandle,
    path: String,
    watcher: tauri::State<'_, Mutex<Option<WatcherHandle>>>,
) -> Result<(), String> {
    let app_clone = app.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(300),
        None,
        move |result: DebounceEventResult| {
            if let Ok(events) = result {
                let should_emit = events.iter().any(|e| {
                    e.paths.iter().any(|p| should_emit_for_path(p))
                });
                if should_emit {
                    let _ = app_clone.emit("repo-changed", ());
                }
            }
        },
    ).map_err(|e| e.to_string())?;

    debouncer
        .watch(&path, RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    *watcher.lock().unwrap() = Some(debouncer);
    Ok(())
}

#[tauri::command]
pub async fn stop_watching(
    watcher: tauri::State<'_, Mutex<Option<WatcherHandle>>>,
) -> Result<(), String> {
    *watcher.lock().unwrap() = None;  // Drop stops the watcher
    Ok(())
}
```

**Frontend listener (Svelte 5)**:
```typescript
import { listen } from '@tauri-apps/api/event';

// $effect pattern (Svelte 5):
$effect(() => {
  let unlisten: (() => void) | undefined;
  listen<void>('repo-changed', () => { /* refresh status */ })
    .then(fn => { unlisten = fn; });
  return () => unlisten?.();
});
```

---

### Q6: What path filtering logic is needed for the watcher?
**Status**: ✅ Answered — High confidence  
**Source**: PRD requirements + notify event structure (Event.paths: Vec<PathBuf>)

```rust
fn should_emit_for_path(path: &std::path::Path) -> bool {
    // Use path components for reliable cross-platform matching
    let components: Vec<_> = path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    // Suppress: target/, node_modules/
    if components.iter().any(|c| c == "target" || c == "node_modules") {
        return false;
    }

    // Find .git component
    if let Some(git_idx) = components.iter().position(|c| c == ".git") {
        if let Some(next) = components.get(git_idx + 1) {
            // Suppress: .git/objects/, .git/logs/
            if next == "objects" || next == "logs" {
                return false;
            }
            // Allow: .git/index, .git/HEAD, .git/refs/
            // These represent staging/commit/branch changes
            return true;
        }
    }

    true
}
```

---

### Q7: Does CSP need to be changed for CM6?
**Status**: ✅ Answered — High confidence  
**Source**: tech-stack.md + tauri.conf.json (already configured)

CSP already has `style-src 'self' 'unsafe-inline'`. CM6's `style-mod` library injects `<style>` elements dynamically — this is already covered. **No CSP changes needed**.

CM6 does **not** require `script-src 'unsafe-eval'`.

---

### Q8: How to register watcher State in lib.rs?
**Status**: ✅ Answered — High confidence

```rust
// src-tauri/src/lib.rs
mod commands;
mod git;
mod watcher;  // NEW

use std::sync::Mutex;
use notify_debouncer_full::{Debouncer, RecommendedCache};
use notify_debouncer_full::notify::RecommendedWatcher;

type WatcherHandle = Debouncer<RecommendedWatcher, RecommendedCache>;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(None::<WatcherHandle>))  // NEW
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_repo_status,
            commands::get_commit_log,
            commands::get_refs,
            watcher::watch_repo,    // NEW
            watcher::stop_watching, // NEW
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Key Findings Summary

1. **CM6 packages**: 7 packages needed — `@codemirror/{view,state,merge,language,commands,lang-javascript,lang-rust}`
2. **MergeView**: `new MergeView({ a: EditorStateConfig, b: EditorStateConfig, parent: div, gutter: true, collapseUnchanged: {...} })`. Access editors via `.a` and `.b`. Call `.destroy()` on unmount.
3. **acceptChunk / rejectChunk**: Called on `EditorView` (e.g., `mv.b`), not on `MergeView`. Returns `boolean`.
4. **3-pane merge**: Not directly supported by `MergeView`. Compose as: left `MergeView`(ours vs base) + center `EditorView`(result, editable) + right `MergeView`(theirs vs base). Three separate `.destroy()` calls needed.
5. **Svelte 5 action**: Standard `(node, options) => { ...; return { update, destroy } }` pattern. Use `Compartment.reconfigure()` for updates, never recreate EditorView.
6. **`Debouncer` IS Send+Sync** in v0.5.0. PRD claim "Debouncer is not Send" is incorrect. Use `Mutex<Option<Debouncer<...>>>` in Tauri State (no Arc needed).
7. **Tauri emit**: `use tauri::{AppHandle, Emitter}`. Accept `app: AppHandle` in command signature, call `app.clone()`, capture clone in callback closure, call `.emit("repo-changed", ())`.
8. **Path filtering**: Use path component matching on `event.paths` (not string contains).
9. **CSP**: Already configured correctly — no changes needed.
10. **Watcher type alias**: `type WatcherHandle = Debouncer<RecommendedWatcher, RecommendedCache>;`

---

## PRD Corrections

| PRD Claim | Actual | Impact |
|---|---|---|
| "Debouncer is not Send — wrap in Arc<Mutex<>>" | Debouncer IS Send+Sync in 0.5.0; just Mutex<Option<>> needed | Simpler code, remove Arc |
| "Arc<Mutex<WatcherState>> in Tauri State<>" | Mutex<Option<WatcherState>> is sufficient | Minor simplification |

---

## Open Questions (Deferred to Implementation)

| Question | Context | Decision |
|---|---|---|
| Should `notify-debouncer-full` be upgraded to 0.7? | 0.5 is pinned in Cargo.toml | Keep 0.5 for spike per PRD |
| Optimal `collapseUnchanged.minSize` for git diffs? | Default is 4; PRD specifies 8 | Use 8 per PRD, tune during MVP |
| CM6 theme: custom vs extend oneDark? | Design tokens available in app.css | Use CSS custom properties referencing design tokens |

---

## Recommended Package Install Command

```bash
pnpm add @codemirror/view @codemirror/state @codemirror/merge \
         @codemirror/language @codemirror/commands \
         @codemirror/lang-javascript @codemirror/lang-rust
```

Approximate bundle size: ~380KB gzipped for these packages combined (within 400KB PRD budget).
