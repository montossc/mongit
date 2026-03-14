# PRD: Canvas 2D Commit Graph Renderer

## Bead Metadata

```yaml
bead_id: bd-15p
type: feature
priority: P0
title: Canvas 2D commit graph renderer (10k+ commits, 60fps)
requirements_score:
  total: 92
  breakdown:
    business_value: 28/30
    functional_requirements: 23/25
    user_experience: 18/20
    technical_constraints: 14/15
    scope_and_priorities: 9/10
  status: passed
  rounds_used: 4
  deferred_questions: 0
```

---

## Problem Statement

mongit's commit graph is the highest-risk architectural component. DOM-based rendering breaks at ~1,000 nodes (layout thrashing, frame drops). No production-ready Canvas 2D git graph library exists — all major tools (gitgraph.js, GitButler, GitHub web) use SVG which shares the same scaling problem.

This spike must prove that Canvas 2D can render a full commit DAG with 10k+ commits at 60fps, with rich interaction (select, hover, branch labels, context menu, keyboard nav). The result validates the architecture before MVP investment.

**Why first:** This is the highest-risk item in `roadmap.md`, the biggest architecture unknown, and its outcome shapes the entire commit-log UI structure.

---

## Scope

### In Scope

- Rust backend: `git2` revwalk command returning commit DAG data via Tauri IPC
- TypeScript: DAG layout algorithm (swimlane lane assignment, segment building)
- Svelte 5 component: Canvas 2D renderer with virtual scrolling
- Rich interaction: click-to-select, hover highlight, branch/tag labels on graph, context menu (right-click), keyboard navigation (up/down/enter)
- Retina (devicePixelRatio) support
- 10-color branch cycle from design system
- Commit detail sidebar showing selected commit info (hash, author, date, message, parents)
- Performance measurement: log layout time, render frame time, FPS counter
- Resizable graph panel (basic drag handle)

### Out of Scope

- Light theme (dark only for spike)
- Repo cloning/creation UI (open existing repos only)
- Graph filtering/search (no author/date/branch filters)
- Diff viewer integration (no inline diffs on commit selection)
- Interactive rebase UI
- IntelliSort merge grouping (standard topological order for now)
- Web Worker for layout (main thread first; optimize later if needed)
- Stash/tag decorations beyond basic labels
- Multi-repo support

---

## Proposed Solution

### Architecture: Full Vertical Slice

```
┌─────────────────────────────────────────────────────┐
│ Rust Backend (src-tauri/src/)                       │
│                                                     │
│  git2::Repository::revwalk()                        │
│    → CommitInfo { oid, parents, message, author,    │
│                   time, refs }                      │
│    → Serialized via serde → IPC response            │
│                                                     │
│  Commands:                                          │
│    get_commit_log(path, limit) → Vec<CommitInfo>    │
│    get_refs(path) → Vec<RefInfo>                    │
└───────────┬─────────────────────────────────────────┘
            │ invoke()
┌───────────▼─────────────────────────────────────────┐
│ Frontend (src/lib/graph/)                           │
│                                                     │
│  types.ts     — CommitNode, GraphSegment, etc.      │
│  layout.ts    — assignLanes() + buildSegments()     │
│  render.ts    — Canvas 2D draw calls (batched)      │
│  hitTest.ts   — mouse → graph element mapping       │
│  GraphCanvas.svelte — Svelte 5 component            │
│                                                     │
│  Data flow:                                         │
│    IPC → CommitDTO[] → layout → LayoutResult        │
│    → $state → $effect → scheduleRender()            │
│    → render.ts → Canvas 2D                          │
└─────────────────────────────────────────────────────┘
```

### Virtual Scrolling Strategy

- Fixed row height (32px per commit row, from design system)
- Sticky canvas positioned at scroll container top
- Spacer div creates total scrollable height (`commits.length * ROW_HEIGHT`)
- On scroll: compute visible row range with 3-row overdraw buffer
- Only render visible rows + edges that cross the viewport
- O(1) row lookup via `Math.floor(scrollY / ROW_HEIGHT)`

### Lane Assignment Algorithm

Standard swimlane algorithm (gitk/Sublime Merge pattern):
1. Walk commits in topological order (newest first)
2. First-parent inherits child's lane (keeps `main` on lane 0)
3. Additional parents get new lanes from a free-list
4. Lanes are freed when no longer needed for edge drawing
5. Colors assigned by `lane % 10` using design system `--graph-color-N`

### Canvas Rendering Pipeline

1. Clear canvas (opaque background — `alpha: false` for ~15% perf gain)
2. Render selection highlight (full-width row rect)
3. Render hover highlight (subtle row rect)
4. Batch edges by color → single `beginPath()` + `stroke()` per color
5. Batch nodes by color → single `beginPath()` + `fill()` per color
6. Render branch/tag labels (text + rounded rect badges)
7. Render commit metadata text (hash, message, author, date)

All rendering gated by `requestAnimationFrame` with dirty flag — never render directly from event handlers.

---

## Requirements

### R1: Rust Commit Log Command

The backend must provide a Tauri IPC command that returns commit history as structured data.

**Command signature:**
```rust
#[tauri::command]
pub fn get_commit_log(path: String, max_count: usize) -> Result<Vec<CommitInfo>, String>
```

**CommitInfo fields:** `oid` (hex string), `parents` (Vec<String>), `message` (first line), `author_name`, `author_email`, `author_time` (unix timestamp), `refs` (Vec<RefInfo>).

**RefInfo fields:** `name`, `kind` (branch | remote_branch | tag), `is_head` (bool).

**Constraints:**
- Use `git2::Revwalk` with `SORT_TOPOLOGICAL | SORT_TIME`
- Open repo per-call (avoid Send+Sync issues)
- `max_count` default: 10,000
- Include all branch tips as starting points (not just HEAD)

### R2: DAG Layout Engine

Pure TypeScript module that assigns lane positions and builds edge segments.

**Input:** Array of `CommitDTO` from IPC (topological order).
**Output:** `LayoutResult { nodes: CommitNode[], segments: GraphSegment[] }`

**CommitNode:** original commit data + `row: number`, `lane: number`, `color: string`.
**GraphSegment:** `fromRow`, `toRow`, `fromLane`, `toLane`, `color`, `isMerge: boolean`.

**Algorithm:** Free-list lane assignment. First-parent inherits lane. O(N) time complexity.

**Must handle:**
- Linear chains (single lane)
- Feature branches (2+ lanes)
- Merge commits (edges converging)
- Octopus merges (3+ parents)
- Detached HEAD

### R3: Canvas 2D Renderer

Svelte 5 component rendering the graph to a `<canvas>` element.

**Visual elements:**
- Commit nodes: 5px radius circles, filled with lane color
- Edges: 2px lines — straight for same-lane, bezier curves for lane changes
- Selection: full-width highlight rect (accent color at 15% opacity)
- Hover: full-width highlight rect (subtle, 8% opacity)
- Branch labels: rounded rect badges with ref name text
- Tag labels: distinct style (outline badge)
- HEAD indicator: bold/special decoration on current branch label
- Commit text columns: short hash (mono), message (truncated), author, relative date

**Performance:**
- Path batching: group draw calls by `fillStyle`/`strokeStyle`
- Retina: scale canvas by `devicePixelRatio`, round to avoid subpixel blur
- Canvas context: `{ alpha: false, desynchronized: true }`
- Dirty flag + `requestAnimationFrame` coalescing

### R4: Virtual Scrolling

- Row height: 32px (fixed)
- Overdraw: 3 rows above/below viewport
- Edge visibility: include segments whose `[toRow, fromRow]` range intersects the visible range
- Scroll container: CSS `overflow-y: auto` with sticky canvas + spacer div
- Handle window resize: `ResizeObserver` on canvas, re-init DPR scaling

### R5: Rich Interaction

**Click:** Select commit row. Show commit details in sidebar panel.
**Hover:** Highlight row under cursor. Change cursor to `pointer` over commit nodes.
**Branch labels:** Clickable — select the commit they point to.
**Context menu:** Right-click on commit shows menu with: copy hash, copy message, show in terminal.
**Keyboard navigation:**
- `↑`/`↓`: Move selection up/down
- `Enter`: Toggle commit detail panel
- `Home`/`End`: Jump to first/last commit
- `Page Up`/`Page Down`: Scroll by viewport height

**Hit testing:** Pure math — map mouse coordinates to row index (O(1)) and check node/label bounds.

### R6: Commit Detail Sidebar

When a commit is selected, show a detail panel (right side, 360px) with:
- Full commit hash (copyable)
- Author name + email
- Commit date (absolute + relative)
- Full commit message
- Parent hashes (clickable — navigate to parent)
- Branch/tag refs on this commit

### R7: Performance Targets

| Metric                   | Target          | How to Measure                           |
| ------------------------ | --------------- | ---------------------------------------- |
| Layout (10k commits)     | < 100ms         | `console.time('layout')`                |
| Render frame             | < 8ms           | rAF timestamp delta                      |
| Sustained FPS (scroll)   | >= 55fps        | FPS counter overlay                      |
| First paint              | < 500ms         | Time from IPC response to first frame    |
| Hit test                 | < 1ms           | `performance.now()` around hit test      |
| Memory (10k commits)     | < 50MB JS heap  | Chrome DevTools / WKWebView inspector    |

### R8: Performance Measurement

Include a toggleable FPS counter overlay (top-right corner) showing:
- Current FPS
- Frame time (ms)
- Visible row count
- Total commit count
- Layout time (one-shot)

---

## Success Criteria

1. **Canvas renders real git data from a 10k+ commit repo at 60fps during continuous scroll**
   - Verify: Open a real repo (e.g., clone `torvalds/linux` or `git/git`), load 10k commits, scroll continuously, FPS counter shows >= 55fps
   - `pnpm tauri dev`, open repo, scroll, read FPS overlay

2. **Lane assignment correctly handles branches, merges, and octopus merges**
   - Verify: Open a repo with merge commits — visual inspection shows correct topology
   - No crossing edges that should be parallel
   - Octopus merge test: create a test commit with 3+ parents, verify all edges render

3. **Branch coloring uses design system 10-color cycle consistently**
   - Verify: Visual check — each branch lane uses a distinct color from `--graph-color-0` through `--graph-color-9`

4. **All interactions work: click, hover, context menu, keyboard nav**
   - Verify: Click commit → detail panel shows. Hover → row highlights. Right-click → context menu appears. Arrow keys → selection moves. Home/End → jumps to boundary.

5. **Retina rendering is crisp (no blur on HiDPI displays)**
   - Verify: Run on Retina Mac, zoom into graph with screenshot — lines and text are sharp at 2x

6. **Virtual scrolling shows no visual glitches (no blank frames, no missing edges)**
   - Verify: Scroll rapidly through 10k commits — no blank flashes, edges don't disappear at viewport boundaries

7. **TypeScript and Rust code both pass checks**
   - Verify: `pnpm check` (0 errors) and `cargo check` in `src-tauri/` (0 errors)

---

## Technical Context

### Existing Codebase

- **IPC pattern:** `src-tauri/src/commands.rs` — `#[tauri::command]` functions returning `Result<T, String>`, registered in `src-tauri/src/lib.rs` via `generate_handler![]`
- **git2 usage:** `Repository::open()`, `head()`, `statuses()` already working — revwalk is available but not yet implemented
- **Frontend:** Svelte 5 with runes (`$state`, `$derived`, `$effect`), SvelteKit adapter-static, SSR disabled
- **Design tokens:** `src/app.css` — 10 graph colors (`--graph-color-0` to `--graph-color-9`), backgrounds, fonts, spacing
- **CSP:** `style-src 'unsafe-inline'` already set in `src-tauri/tauri.conf.json` — Canvas 2D is not CSP-restricted

### Key Constraints

- `git2::Repository` is not `Send+Sync` — open per-call in command handlers
- Canvas context should use `{ alpha: false }` for opaque background (15% perf boost)
- Retina scaling: `Math.round(devicePixelRatio)` to avoid subpixel artifacts
- Window has 38px title bar overlay — graph must account for this padding
- Min window size: 900x600

### Research Findings

- **No production Canvas 2D git graph library exists** — must build custom
- **Lane assignment algorithm:** Standard free-list approach from gitk/Sublime Merge is O(N) and handles all edge cases
- **Path batching** is the most impactful Canvas perf technique — group draw calls by color
- **Bezier curves** for lane-change edges (1-2 row transitions), straight diagonals for longer ranges
- **GitButler uses SVG** for their graph (not Canvas) — their approach doesn't scale to 10k+

---

## Affected Files

### New Files

| File                               | Purpose                              |
| ---------------------------------- | ------------------------------------ |
| `src/lib/graph/types.ts`            | CommitNode, GraphSegment, interfaces |
| `src/lib/graph/layout.ts`           | Lane assignment + segment builder    |
| `src/lib/graph/render.ts`           | Canvas 2D draw functions             |
| `src/lib/graph/hitTest.ts`          | Mouse → element mapping              |
| `src/lib/graph/GraphCanvas.svelte`  | Main graph canvas component          |
| `src/lib/graph/CommitDetail.svelte` | Commit detail sidebar panel          |
| `src/lib/graph/ContextMenu.svelte`  | Right-click context menu             |
| `src/lib/graph/FpsOverlay.svelte`   | Performance measurement overlay      |
| `src-tauri/src/git/mod.rs`          | Git module root                      |
| `src-tauri/src/git/log.rs`          | Commit log + revwalk logic           |
| `src-tauri/src/git/types.rs`        | CommitInfo, RefInfo serde types      |

### Modified Files

| File                           | Change                                       |
| ------------------------------ | -------------------------------------------- |
| `src-tauri/src/commands.rs`      | Add `get_commit_log`, `get_refs` commands    |
| `src-tauri/src/lib.rs`          | Register new commands in `generate_handler!` |
| `src/routes/+page.svelte`       | Replace placeholder with graph view          |
| `src/app.css`                    | Minor additions (graph-specific utilities)   |

---

## Tasks

### Task 1: [backend] Rust commit log IPC command

Implement `get_commit_log` and `get_refs` Tauri commands that return structured commit DAG data from `git2::Revwalk`. Create `src-tauri/src/git/` module with types and log logic. Register commands in `lib.rs`.

```
depends_on: none
parallel: true (with Task 2)
conflicts_with: none
files: src-tauri/src/git/mod.rs, src-tauri/src/git/log.rs, src-tauri/src/git/types.rs, src-tauri/src/commands.rs, src-tauri/src/lib.rs
```

Verify: `cargo check` passes. Manual test: `invoke('get_commit_log', { path: '.', maxCount: 100 })` returns commit array with parents.

### Task 2: [frontend] TypeScript types and layout engine

Define `CommitNode`, `GraphSegment`, `LayoutResult` types. Implement `assignLanes()` and `buildSegments()` as pure functions. Layout 10k synthetic commits in < 100ms.

```
depends_on: none
parallel: true (with Task 1)
conflicts_with: none
files: src/lib/graph/types.ts, src/lib/graph/layout.ts
```

Verify: `pnpm check` passes. Unit-testable: generate 10k synthetic commits, run layout, verify lane count < 50 and no lane < 0. Console.time layout < 100ms.

### Task 3: [frontend] Canvas 2D renderer with virtual scrolling

Implement `render.ts` with batched draw functions (edges, nodes, labels, text, selection, hover). Implement virtual scrolling with sticky canvas + spacer div pattern. Handle Retina DPR scaling.

```
depends_on: Task 2 (needs LayoutResult types)
parallel: false
conflicts_with: none
files: src/lib/graph/render.ts
```

Verify: `pnpm check` passes. Visual: renders synthetic data without glitches. Frame time < 8ms in performance overlay.

### Task 4: [frontend] Hit testing and interaction

Implement `hitTest.ts` for mouse → element mapping. Wire click, hover, right-click, and keyboard events in `GraphCanvas.svelte`. Implement context menu component.

```
depends_on: Task 2, Task 3
parallel: false
conflicts_with: none
files: src/lib/graph/hitTest.ts, src/lib/graph/GraphCanvas.svelte, src/lib/graph/ContextMenu.svelte
```

Verify: `pnpm check` passes. Click selects commit. Hover highlights row. Arrow keys navigate. Right-click shows menu.

### Task 5: [frontend] Commit detail sidebar

Build `CommitDetail.svelte` showing full commit info when a commit is selected. Include: hash (copyable), author, date, message, parent links, refs.

```
depends_on: Task 4
parallel: false
conflicts_with: none
files: src/lib/graph/CommitDetail.svelte
```

Verify: `pnpm check` passes. Selecting a commit shows detail panel with all fields populated.

### Task 6: [frontend] FPS overlay and performance measurement

Build `FpsOverlay.svelte` showing FPS, frame time, visible rows, total commits, layout time. Toggleable with a keyboard shortcut.

```
depends_on: Task 3
parallel: true (with Task 4, 5)
conflicts_with: none
files: src/lib/graph/FpsOverlay.svelte
```

Verify: `pnpm check` passes. FPS counter visible in top-right corner. Shows accurate frame timing.

### Task 7: [integration] Wire full vertical slice

Connect Rust backend to frontend: invoke `get_commit_log` on page load, feed results through layout engine, render in canvas. Update `+page.svelte` to show the graph view with sidebar. Add repo path selection (open folder dialog via Tauri).

```
depends_on: Task 1, Task 4, Task 5, Task 6
parallel: false
conflicts_with: none
files: src/routes/+page.svelte
```

Verify: `pnpm check` + `cargo check` both pass. Open a real git repo with 1k+ commits — graph renders, interactions work, FPS overlay shows data.

### Task 8: [validation] 10k commit performance test

Open a large real repo (git/git, torvalds/linux, or generate synthetic 10k DAG). Verify all success criteria: 60fps scroll, correct topology, crisp Retina rendering, no visual glitches.

```
depends_on: Task 7
parallel: false
conflicts_with: none
files: none (testing only)
```

Verify: FPS overlay shows >= 55fps during continuous scroll with 10k commits. Layout time < 100ms. No blank frames. All interactions responsive.

---

## Risks

| Risk                                    | Severity | Mitigation                                                             |
| --------------------------------------- | -------- | ---------------------------------------------------------------------- |
| 50+ active lanes make graph very wide   | Medium   | Aggressive lane reuse via free-list; minimum lane width 16px at scale  |
| Bezier curves look cluttered on merges  | Low      | Straight diagonals for segments > 10 rows; bezier only for short hops  |
| Layout of 10k commits blocks main thread| Medium   | Measure first; move to Web Worker only if > 100ms                      |
| git2 revwalk performance on huge repos  | Low      | Limit to `max_count`; pagination if needed                             |
| Branch label collision / overlap        | Medium   | Pre-compute label positions at layout time; offset overlapping labels  |
| Text rendering performance on Canvas    | Medium   | Batch text by font/size; consider caching text metrics                 |

---

## Open Questions

None — all questions resolved during refinement rounds.
