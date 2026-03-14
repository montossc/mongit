# Research: bd-15p — Canvas 2D Commit Graph Renderer

**Date:** 2026-03-14
**Depth:** Moderate (~25 tool calls)
**Bead:** bd-15p — Canvas 2D commit graph renderer (10k+ commits, 60fps)

---

## Questions & Answers

### Q1: How to push all branch tips into git2 revwalk?

**Answer:** HIGH confidence  
**Source:** git2 docs + 5 real-world GitHub code examples (schaltwerk, Gram-ax/gramax, zhukunpenglinyutong/mossx, etc.)

```rust
revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;
revwalk.push_glob("refs/heads/*")?;     // all local branches
revwalk.push_glob("refs/remotes/*")?;   // all remote tracking branches
revwalk.push_glob("refs/tags/*")?;      // tags pointing to commits
// Fallback: if repo has no refs yet, push_head() won't error
let _ = revwalk.push_head();
```

`push_glob` accepts a glob pattern; leading `refs/` is implied if missing. Trailing `/*` is added automatically if glob lacks `?`, `*`, or `[`. Any ref not pointing to a commitish is silently ignored (safe to call).

### Q2: How to build ref → commit mapping (branch labels per row)?

**Answer:** HIGH confidence  
**Source:** git2 docs + existing `branches()` function in `repository.rs`

```rust
let mut refs_by_oid: HashMap<String, Vec<RefInfo>> = HashMap::new();
let head_oid = repo.head().ok().and_then(|h| h.target()).map(|o| o.to_string());

for ref_result in repo.references()? {
    let reference = ref_result?;
    // Peel to commit (handles annotated tags → tag → commit)
    let commit_oid = reference.peel_to_commit().ok()
        .map(|c| c.id().to_string());
    if let (Some(oid), Some(name)) = (commit_oid, reference.shorthand()) {
        let kind = if reference.is_branch() { "branch" }
                   else if reference.is_remote() { "remote_branch" }
                   else if reference.is_tag() { "tag" }
                   else { continue };
        let is_head = head_oid.as_deref() == Some(&oid)
                      && reference.is_branch();
        refs_by_oid.entry(oid).or_default().push(RefInfo {
            name: name.to_string(),
            kind: kind.to_string(),
            is_head,
        });
    }
}
```

`peel_to_commit()` handles annotated tags (peels through tag object to commit). This is the correct API to use.

### Q3: Can the existing `CommitInfo` struct be extended or must we create new types?

**Answer:** HIGH confidence  
**Source:** Codebase analysis (repository.rs)

The existing `CommitInfo` in `repository.rs` has `#[allow(dead_code)]` and is `pub`. It's missing `refs: Vec<RefInfo>`. Two options:
- **Option A (preferred):** Add `refs: Vec<RefInfo>` to existing `CommitInfo` + new `RefInfo` struct in `repository.rs`
- **Option B:** Create separate types in `src-tauri/src/git/types.rs` (as PRD proposes)

The PRD's naming in `commands.rs` uses `CommitInfo` which aligns with the existing name. Using the same type and extending it avoids confusion. **Recommend Option A** — extend existing `CommitInfo` with `refs` field and add `RefInfo` struct to `repository.rs`.

### Q4: Svelte 5 canvas integration pattern?

**Answer:** HIGH confidence  
**Source:** Official Svelte 5 docs (https://svelte.dev/docs/svelte/%24effect)

```svelte
<script lang="ts">
  let canvas: HTMLCanvasElement;

  $effect(() => {
    const ctx = canvas.getContext('2d', { alpha: false, desynchronized: true });
    // draw — runs on mount and when $state dependencies change
    return () => {
      // cleanup (e.g., cancel animation frame)
    };
  });
</script>

<canvas bind:this={canvas}></canvas>
```

`$effect` runs after the DOM mounts (canvas is not undefined inside effect). Returning a cleanup function is the correct pattern for `cancelAnimationFrame`.

### Q5: How to read CSS custom properties for Canvas colors?

**Answer:** HIGH confidence  
**Source:** MDN Web API (standard browser API)

Canvas 2D cannot directly use CSS custom properties (`var(--graph-color-0)`). Must resolve at runtime:

```typescript
// In GraphCanvas.svelte, inside $effect or onMount:
function readGraphColors(): string[] {
  const root = document.documentElement;
  const style = getComputedStyle(root);
  return Array.from({ length: 10 }, (_, i) =>
    style.getPropertyValue(`--graph-color-${i}`).trim()
  );
}

const GRAPH_COLORS = readGraphColors();
// Result: ['#53C1DE', '#3ECF8E', '#F87171', ...]
```

Call once on mount. Colors are stable (dark-only for spike).

### Q6: Does `desynchronized: true` canvas hint work in WKWebView?

**Answer:** MEDIUM confidence  
**Source:** WebKit source + MDN + safari release notes (no direct confirmation found for WKWebView)

The `desynchronized` hint is a rendering optimization that allows the compositor to skip waiting for JavaScript synchronization. It was added to Chrome in ~2018. WebKit/Safari has added this hint support in modern versions (Safari 17+). WKWebView uses the same WebKit engine, so it should be supported on macOS 14+.

**Risk:** WKWebView silently ignores unsupported canvas context attributes. If not supported, it falls back gracefully to default (synchronized) rendering. **Safe to include** as a hint; no negative effect if unsupported.

### Q7: Does the dialog plugin (`tauri-plugin-dialog`) need to be added?

**Answer:** HIGH confidence  
**Source:** Tauri docs + Cargo.toml analysis

**Current state:** `tauri-plugin-dialog` is NOT in `Cargo.toml` or `package.json`. The folder picker for Task 7 requires it.

**Setup required (NEW DEPENDENCY — needs user approval):**
```bash
# Rust side
cargo add tauri-plugin-dialog   # in src-tauri/
# Frontend side
pnpm add @tauri-apps/plugin-dialog
```

Register in `lib.rs`:
```rust
.plugin(tauri_plugin_dialog::init())
```

Add capabilities file `src-tauri/capabilities/main-capability.json`:
```json
{
  "identifier": "main-capability",
  "description": "Main window capabilities",
  "windows": ["main"],
  "permissions": ["dialog:default"]
}
```

**Alternative for spike (no new dependency):** Use a plain `<input>` element for repo path entry (text box). This avoids the dependency and is sufficient for the architecture validation spike.

### Q8: What does the current `Git2Repository::log()` push?

**Answer:** HIGH confidence  
**Source:** Codebase analysis (repository.rs lines ~140-170)

Current `log()` calls `revwalk.push_head()` only. It **does not** push all branch tips. This means commits only reachable from HEAD will be included. For the commit graph, this misses commits on other branches that aren't ancestors of HEAD.

The new `get_commit_log` IPC command must use `push_glob("refs/heads/*")` + `push_glob("refs/remotes/*")` instead.

### Q9: What is the IPC command naming conflict to avoid?

**Answer:** HIGH confidence  
**Source:** Codebase analysis (commands.rs, lib.rs)

Currently registered commands: `greet`, `get_repo_status`. The new commands `get_commit_log` and `get_refs` don't conflict. However, the **frontend type** for `CommitInfo` will be auto-generated from Rust types. If adding `refs` to the Rust `CommitInfo`, the TypeScript call site needs to match.

---

## Key Insights

1. **No new npm packages needed** for the canvas renderer itself — Canvas 2D is native to the browser. Only the dialog plugin adds a new dependency (optional for spike).

2. **The existing `Git2Repository::log()` is unsuitable** for the graph. It only pushes HEAD, missing orphaned branches. New command needs `push_glob("refs/heads/*")`.

3. **`peel_to_commit()`** is the correct git2 API for annotated tags — don't use `target()` directly on tags as it returns the tag object OID, not the commit OID.

4. **CSS vars must be resolved at runtime** via `getComputedStyle()` — not usable directly in canvas draw calls.

5. **Svelte 5 `$effect` + `bind:this`** is the correct pattern for canvas lifecycle. The `$effect` cleanup function cancels animation frames.

6. **`requestAnimationFrame` + dirty flag pattern**: Don't render directly from event handlers or `$effect`. Instead, set a dirty flag and let rAF coalesce renders.

7. **Extending `CommitInfo` in `repository.rs`** is simpler than creating `src-tauri/src/git/types.rs`. The PRD mentions `types.rs` but this creates unnecessary indirection.

8. **Tauri 2.0 capabilities:** Currently no `src-tauri/capabilities/` directory exists. The dialog plugin will need one created. For the spike, skip the dialog and use a text input.

---

## Recommended Approach

### Backend (Task 1)

Extend `CommitInfo` in `repository.rs` with `refs: Vec<RefInfo>`. Add `RefInfo` struct. Create new `get_commit_log` function (separate from existing `log()`) that:
1. Builds refs-by-oid HashMap upfront
2. Uses `push_glob` for all refs
3. Attaches refs to each commit from the HashMap

Register as a new Tauri command in `commands.rs`. **No new Cargo dependencies needed.**

### Frontend (Tasks 2-6)

```
src/lib/graph/
  types.ts      — CommitDTO, CommitNode, GraphSegment, LayoutResult, RefInfo
  layout.ts     — assignLanes(), buildSegments() pure functions
  render.ts     — Canvas 2D batched drawing (edges, nodes, labels)
  hitTest.ts    — rowFromY(), elementAtPoint()
  GraphCanvas.svelte — bind:this + $effect + rAF loop
  CommitDetail.svelte
  ContextMenu.svelte
  FpsOverlay.svelte
```

Graph colors: read once from `getComputedStyle(document.documentElement)` at init.

Canvas context: `{ alpha: false, desynchronized: true }` — safe in WKWebView.

### Task 7 (Integration)

For the spike, use a simple text `<input>` for repo path (no dialog plugin needed). The dialog plugin can be added in MVP phase with user approval.

---

## Open Items

| Item | Status | Notes |
|------|--------|-------|
| Dialog plugin dependency | ⚠️ Needs approval | Required for folder picker in Task 7; skip for spike |
| `desynchronized` WKWebView support | ❓ Unconfirmed | Safe to include; ignored if unsupported |
| Performance target validation | ⏳ Needs testing | 60fps on 10k commits — only verified after Task 8 |

---

## Sources

- git2 Revwalk API: https://docs.rs/git2/latest/git2/struct.Revwalk
- git2 push_glob examples: grep.app search (schaltwerk, gramax, mossx)
- Svelte 5 $effect + canvas: https://svelte.dev/docs/svelte/%24effect
- Tauri dialog plugin: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/plugin/dialog.mdx
- Codebase: `src-tauri/src/git/repository.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src/app.css`
