# CodeMirror 6 Diff/Merge + File Watching

**Bead:** bd-2n2  
**Created:** 2026-03-14  
**Status:** Draft

## Bead Metadata

```yaml
depends_on: []
parallel: true
conflicts_with: ["bd-15p"] # Both modify src-tauri/src/lib.rs
blocks: []
estimated_hours: 16
requirements_score:
  total: 90
  breakdown:
    business_value: 27/30
    functional_requirements: 22/25
    user_experience: 18/20
    technical_constraints: 13/15
    scope_and_priorities: 10/10
  status: passed
  rounds_used: 3
  deferred_questions: 1
```

---

## Problem Statement

### What problem are we solving?

mongit needs a diff viewer, merge editor, and file watcher to deliver its core Git client functionality. Before building these into the MVP, we must validate that:

1. **CodeMirror 6 integrates cleanly with Svelte 5** (runes mode, action-based lifecycle)
2. **CM6 works under Tauri's CSP restrictions** (WKWebView, `style-src 'unsafe-inline'`)
3. **The `notify` crate provides reliable file watching** with appropriate debouncing for Git repos
4. **Side-by-side diff and 3-pane merge are feasible** with acceptable performance

Without this validation, the MVP risks hitting architectural dead ends that would require expensive rework.

### Why now?

This is Spike D in the Foundation phase (Weeks 1-4). The commit graph spike (bd-15p) validates rendering; this spike validates the editing/viewing/watching subsystems. Both must pass before MVP decomposition begins.

### Who is affected?

- **Primary users:** mongit developers (us) — need confidence in the tech stack
- **Secondary users:** Future mongit users — the spike directly shapes the diff/merge UX they'll use

---

## Scope

### In-Scope

- CodeMirror 6 side-by-side diff viewer with hunk-level staging controls
- CodeMirror 6 three-pane merge editor (ours | result | theirs)
- Rust file watcher using `notify` + `notify-debouncer-full` with coarse `repo-changed` events
- CSP compatibility validation (confirm CM6 works under Tauri's WKWebView CSP)
- Tabbed spike dashboard page (temporary — removed before MVP)
- Performance benchmarks for each subsystem
- Svelte 5 action-based CM6 wrapper pattern

### Out-of-Scope

- Line-level staging (deferred to MVP — requires custom StateField)
- Unified diff mode (side-by-side is the primary target; unified can be added later)
- Syntax highlighting for all languages (spike validates with 2-3 languages only)
- Production-quality theming (spike uses design tokens but not final theme)
- Real git2 diff integration (spike uses hardcoded sample data; git2 commands are Spike C)
- Watcher filtering heuristics tuning (spike proves the pattern; tuning is MVP work)
- File watcher for multiple repos simultaneously (single repo only)

---

## Proposed Solution

### Overview

Build four isolated subsystems, each as a Svelte 5 component/module backed by appropriate Rust infrastructure, and wire them into a temporary tabbed dashboard page at `/spike-d`. Each subsystem proves its core API works, measures performance, and validates Tauri integration.

### Architecture

```
Tabbed Dashboard (/spike-d)
├── Tab 1: Side-by-Side Diff Viewer
│     └── MergeView (CM6 @codemirror/merge)
│           ├── Left pane: original (read-only)
│           ├── Right pane: modified (read-only)
│           └── Gutter: hunk accept/reject buttons
│
├── Tab 2: Three-Pane Merge Editor
│     ├── Left: ours vs base (MergeView, read-only)
│     ├── Center: result (EditorView, editable)
│     └── Right: theirs vs base (MergeView, read-only)
│
├── Tab 3: File Watcher Monitor
│     ├── Rust: notify debouncer → app.emit("repo-changed")
│     ├── Svelte: listen("repo-changed") → event log
│     └── Controls: start/stop watching, select repo path
│
└── Tab 4: CSP & Performance Report
      ├── CM6 style injection test (proves unsafe-inline works)
      ├── Render timing for diff (1k, 10k, 50k line files)
      └── Watcher event latency measurement
```

### User Flow (spike validation)

1. Developer opens the app → navigates to `/spike-d` route
2. **Diff tab:** Sees hardcoded sample diff in side-by-side view. Clicks hunk accept/reject buttons. Observes gutter controls and collapsed unchanged regions.
3. **Merge tab:** Sees hardcoded 3-way conflict. Edits center result pane. Clicks "Accept Ours" / "Accept Theirs" per chunk.
4. **Watcher tab:** Selects a local repo path. Modifies a file externally. Sees `repo-changed` event appear in the log with timestamp and latency.
5. **Benchmark tab:** Clicks "Run Benchmarks". Sees render time for increasing file sizes. Sees watcher event latency histogram.

---

## Requirements

### Functional Requirements

#### Side-by-Side Diff Viewer

The diff viewer displays two file versions side-by-side using `@codemirror/merge` MergeView, with hunk-level accept/reject controls in the gutter.

**Scenarios:**

- **WHEN** a diff is loaded with original and modified text **THEN** two panes render side-by-side with highlighted changes and connected change indicators
- **WHEN** unchanged regions exceed 8 lines **THEN** they collapse with an expandable "N unchanged lines" widget (`collapseUnchanged: { margin: 3, minSize: 8 }`)
- **WHEN** user clicks a hunk accept button **THEN** `acceptChunk()` fires and the chunk highlight clears
- **WHEN** user clicks a hunk reject button **THEN** `rejectChunk()` fires and the chunk reverts to original
- **WHEN** the component unmounts **THEN** `MergeView.destroy()` is called (no memory leaks)

#### Three-Pane Merge Editor

The merge editor displays a 3-pane layout for conflict resolution: ours (left), result (center, editable), theirs (right).

**Scenarios:**

- **WHEN** a conflict is loaded with base, ours, and theirs text **THEN** three panes render: left shows ours-vs-base diff, center shows editable result, right shows theirs-vs-base diff
- **WHEN** user clicks "Accept Ours" on a chunk **THEN** the ours version replaces that chunk in the center result editor
- **WHEN** user clicks "Accept Theirs" on a chunk **THEN** the theirs version replaces that chunk in the center result editor
- **WHEN** user manually edits the center pane **THEN** changes persist and the conflict for that region is considered manually resolved
- **WHEN** the component unmounts **THEN** all three EditorView/MergeView instances are destroyed

#### File Watcher (Coarse Events)

The Rust file watcher uses `notify` + `notify-debouncer-full` to emit a single `repo-changed` Tauri event when any relevant file in the watched repository changes.

**Scenarios:**

- **WHEN** `watch_repo` is called with a valid repo path **THEN** a recursive FSEvents watcher starts on that path with 300ms debounce
- **WHEN** a file in the working tree is created, modified, or deleted **THEN** a `repo-changed` event is emitted to the frontend within 500ms
- **WHEN** changes occur inside `.git/objects`, `.git/logs`, `target/`, or `node_modules/` **THEN** they are filtered out (no event emitted)
- **WHEN** changes occur to `.git/index`, `.git/HEAD`, or `.git/refs/heads/*` **THEN** a `repo-changed` event IS emitted (staging area, branch switch, commit)
- **WHEN** `watch_repo` is called while already watching a different path **THEN** the old watcher is dropped and a new one starts
- **WHEN** `stop_watching` is called **THEN** the watcher stops and no further events are emitted

#### CSP Compatibility

CM6 renders correctly under Tauri's Content Security Policy without errors.

**Scenarios:**

- **WHEN** CM6 EditorView is created inside Tauri's WKWebView **THEN** no CSP violation errors appear in the console
- **WHEN** CM6 injects styles via `style-mod` **THEN** styles apply correctly (visible syntax highlighting, gutter styling, selection colors)
- **WHEN** the app runs with `style-src 'self' 'unsafe-inline'` **THEN** all CM6 features work without `script-src 'unsafe-eval'`

#### Svelte 5 Integration Pattern

CM6 lifecycle is managed via a Svelte action (`use:codemirror`), not a wrapper component.

**Scenarios:**

- **WHEN** props change (e.g., new document content) **THEN** the editor updates via `Compartment.reconfigure()` or `view.dispatch({ changes })` — NOT by recreating EditorView
- **WHEN** the Svelte component unmounts **THEN** the action's `destroy()` callback fires and calls `EditorView.destroy()`
- **WHEN** the editor is in read-only mode **THEN** `EditorState.readOnly` compartment is active and keyboard input is ignored

### Non-Functional Requirements

- **Performance:** Side-by-side diff renders a 10,000-line file in < 200ms. 50,000-line file in < 1s. Watcher event latency < 500ms from file save to frontend event.
- **Memory:** Each CM6 editor instance uses < 50MB for a 50k-line file. Max 5 editor instances alive simultaneously.
- **Bundle size:** CM6 packages add < 400KB gzipped to the frontend bundle.
- **Compatibility:** macOS 13+ (Ventura), WKWebView (Safari 16+).

---

## Success Criteria

- [ ] Side-by-side diff viewer renders with highlighted changes, collapsed unchanged regions, and functional hunk accept/reject buttons
  - Verify: `pnpm tauri dev` → navigate to `/spike-d` → Diff tab shows two-pane diff with gutter controls
- [ ] Three-pane merge editor renders with editable center pane and functional Accept Ours/Theirs buttons
  - Verify: Merge tab shows 3 panes, clicking accept buttons updates center editor content
- [ ] File watcher emits `repo-changed` event within 500ms of a file modification
  - Verify: Watcher tab → select repo → `touch <file>` in terminal → event appears in log with timestamp
- [ ] No CSP violation errors in WKWebView console for any CM6 operation
  - Verify: Open Safari Web Inspector → Console tab → no `Content-Security-Policy` errors during all tab interactions
- [ ] 10k-line diff renders in < 200ms
  - Verify: Benchmark tab → "Run Diff Benchmark" → 10k result shows < 200ms
- [ ] 50k-line diff renders in < 1000ms
  - Verify: Benchmark tab → "Run Diff Benchmark" → 50k result shows < 1000ms
- [ ] Watcher correctly filters `.git/objects` and `node_modules/` (no events for changes in those paths)
  - Verify: Watcher tab → `touch .git/objects/test` → no event in log; `touch src/test` → event appears
- [ ] `pnpm check` passes with 0 errors after all spike code is added
  - Verify: `pnpm check`
- [ ] `cargo check` passes in `src-tauri/` after watcher module is added
  - Verify: `cd src-tauri && cargo check`
- [ ] All CM6 editor instances are properly destroyed on component unmount (no memory leaks)
  - Verify: Navigate away from `/spike-d` → Safari Memory Inspector shows no retained EditorView objects

---

## Technical Context

### Existing Patterns

- **IPC commands:** `src-tauri/src/commands.rs` — `#[tauri::command]` returning `Result<T, String>`, called via `invoke()` from frontend
- **Tauri builder:** `src-tauri/src/lib.rs` — plugin registration, command handler registration via `generate_handler![]`
- **Svelte 5 runes:** `src/routes/+page.svelte` — `$state()`, `$effect()`, `onMount()` pattern
- **Static adapter:** `src/routes/+layout.ts` — `ssr=false`, `prerender=true`
- **Design tokens:** `src/app.css` — diff colors, merge conflict colors, monospace font stack all pre-defined

### Key Files

- `src-tauri/tauri.conf.json` — CSP already includes `style-src 'self' 'unsafe-inline'`
- `src-tauri/Cargo.toml` — `notify = "8"`, `notify-debouncer-full = "0.5"`, `tokio = "1"` already present
- `src/app.css` — Diff color tokens: `--color-diff-added-bg`, `--color-diff-removed-bg`, `--color-diff-hunk-header`; merge conflict tokens: ours `#3ECF8E`, theirs `#60A5FA`, base `#FBBF24`

### Affected Files

Files this bead will create or modify (for conflict detection):

```yaml
files:
  # NEW files
  - src/lib/components/DiffViewer.svelte       # Side-by-side diff component
  - src/lib/components/MergeEditor.svelte      # 3-pane merge component
  - src/lib/components/WatcherMonitor.svelte   # Watcher event log component
  - src/lib/components/BenchmarkPanel.svelte   # Performance benchmark component
  - src/lib/actions/codemirror.ts              # Svelte action for CM6 lifecycle
  - src/lib/utils/codemirror-config.ts         # CM6 extensions, theme, shared config
  - src/lib/utils/codemirror-theme.ts          # mongit CM6 theme using design tokens
  - src/lib/stores/watcher.svelte.ts           # Watcher event state (Svelte 5 runes)
  - src/routes/spike-d/+page.svelte            # Tabbed spike dashboard
  - src-tauri/src/watcher.rs                   # Rust file watcher module
  # MODIFIED files
  - src-tauri/src/lib.rs                       # Register watcher commands + State<>
  - src-tauri/src/commands.rs                  # Add watch_repo, stop_watching commands
  - package.json                               # Add @codemirror/* dependencies
```

### Technology Decisions

| Decision | Rationale |
| --- | --- |
| Svelte action (`use:codemirror`) over component wrapper | CM6 owns its DOM; action provides clean mount/destroy lifecycle without fighting Svelte's reactivity |
| `Compartment.reconfigure()` for reactive updates | Avoids recreating EditorView on prop changes; surgical reconfiguration |
| `MergeView` (side-by-side) as primary diff mode | User-selected; aligns with JetBrains VCS side-by-side default |
| Compose 2× MergeView + center EditorView for 3-pane | CM6 provides 2-pane natively; 3-pane requires composition |
| Coarse `repo-changed` event (not per-file) | Simplest reliable model; frontend re-fetches status on event |
| 300ms debounce in Rust + 100ms frontend debounce | Prevents event storms during git operations without excessive latency |
| `Arc<Mutex<WatcherState>>` in Tauri State<> | Debouncer is not Send; mutex provides safe cross-thread access |
| Filter `.git/objects`, `.git/logs`, `target/`, `node_modules/` | These generate hundreds of events during git operations with no user-visible change |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| CM6 3-pane merge scroll sync is difficult | Medium | Medium | Start with independent scrolling; sync is enhancement, not blocker |
| 50k-line diff exceeds 1s render target | Low | Medium | Use aggressive `collapseUnchanged`; fall back to chunked rendering |
| `notify-debouncer-full` 0.5 has API differences from 0.7 docs | Medium | Low | Pin to 0.5 for spike; evaluate upgrade in MVP |
| WKWebView CSP blocks CM6 in unexpected way | Low | High | Test CSP early (Task 1); if blocked, investigate nonce-based workaround |
| Hunk-level staging via `acceptChunk` doesn't map cleanly to git staging | Medium | Medium | Spike validates UI interaction only; real staging integration is MVP work |

---

## Open Questions

| Question | Owner | Due Date | Status |
| --- | --- | --- | --- |
| Should `notify-debouncer-full` be upgraded from 0.5 to 0.7? | Dev | During implementation | Open |
| What's the optimal `collapseUnchanged.minSize` for typical git diffs? | Dev | During benchmarking | Open |
| Should the CM6 theme be a full custom theme or extend `oneDark`? | Dev | During implementation | Open |

---

## Tasks

### Install CM6 dependencies and validate CSP [setup]

All CodeMirror 6 packages are installed, a minimal EditorView renders inside Tauri's WKWebView with no CSP errors, and the Svelte action pattern is established.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - package.json
  - src/lib/actions/codemirror.ts
  - src/lib/utils/codemirror-config.ts
  - src/lib/utils/codemirror-theme.ts
```

**Verification:**

- `pnpm check` passes with 0 errors
- `pnpm tauri dev` → CM6 editor renders text with syntax highlighting
- Safari Web Inspector Console shows no CSP violation errors

### Build side-by-side diff viewer component [frontend]

A `DiffViewer.svelte` component renders two file versions side-by-side using `MergeView`, with collapsed unchanged regions and hunk-level accept/reject gutter buttons.

**Metadata:**

```yaml
depends_on: ["Install CM6 dependencies and validate CSP"]
parallel: false
conflicts_with: []
files:
  - src/lib/components/DiffViewer.svelte
```

**Verification:**

- Component renders with hardcoded original/modified text showing highlighted changes
- Unchanged regions > 8 lines are collapsed
- Hunk accept/reject buttons appear in gutter and fire `acceptChunk()`/`rejectChunk()`
- `pnpm check` passes

### Build three-pane merge editor component [frontend]

A `MergeEditor.svelte` component renders a 3-pane conflict resolution view: ours (left, read-only), result (center, editable), theirs (right, read-only), with Accept Ours/Theirs per-chunk buttons.

**Metadata:**

```yaml
depends_on: ["Install CM6 dependencies and validate CSP"]
parallel: true
conflicts_with: []
files:
  - src/lib/components/MergeEditor.svelte
```

**Verification:**

- Three panes render with base/ours/theirs content
- Center pane is editable; left and right are read-only
- "Accept Ours" and "Accept Theirs" buttons update center editor content
- All three editor instances are destroyed on unmount
- `pnpm check` passes

### Implement Rust file watcher module [backend]

A `watcher.rs` module provides `watch_repo` and `stop_watching` Tauri commands using `notify` + `notify-debouncer-full`, emitting coarse `repo-changed` events with path filtering.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src-tauri/src/watcher.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/commands.rs
```

**Verification:**

- `cargo check` passes in `src-tauri/`
- `watch_repo` command starts FSEvents watcher on provided path
- File modifications emit `repo-changed` event within 500ms
- Changes in `.git/objects`, `node_modules/`, `target/` do NOT emit events
- Changes to `.git/index`, `.git/HEAD` DO emit events
- Calling `watch_repo` with a new path drops the old watcher

### Build watcher monitor frontend component [frontend]

A `WatcherMonitor.svelte` component provides controls to start/stop watching a repo and displays an event log with timestamps and latency measurements.

**Metadata:**

```yaml
depends_on: ["Implement Rust file watcher module"]
parallel: false
conflicts_with: []
files:
  - src/lib/components/WatcherMonitor.svelte
  - src/lib/stores/watcher.svelte.ts
```

**Verification:**

- Start watching → modify file externally → event appears in log
- Event log shows timestamp and latency (ms from file change to frontend receipt)
- Stop watching → modify file → no new events
- `pnpm check` passes

### Build benchmark panel component [frontend]

A `BenchmarkPanel.svelte` component runs and displays performance benchmarks: diff render time at 1k/10k/50k lines, merge editor render time, and watcher event latency.

**Metadata:**

```yaml
depends_on: ["Build side-by-side diff viewer component", "Build three-pane merge editor component", "Build watcher monitor frontend component"]
parallel: false
conflicts_with: []
files:
  - src/lib/components/BenchmarkPanel.svelte
```

**Verification:**

- "Run Diff Benchmark" generates timing results for 1k, 10k, 50k line files
- 10k-line diff result < 200ms
- 50k-line diff result < 1000ms
- Watcher latency measurement shows < 500ms
- `pnpm check` passes

### Assemble tabbed spike dashboard [integration]

A `/spike-d` route renders a tabbed dashboard with all four subsystems (Diff, Merge, Watcher, Benchmarks), providing a single entry point for spike validation.

**Metadata:**

```yaml
depends_on: ["Build side-by-side diff viewer component", "Build three-pane merge editor component", "Build watcher monitor frontend component", "Build benchmark panel component"]
parallel: false
conflicts_with: []
files:
  - src/routes/spike-d/+page.svelte
```

**Verification:**

- `pnpm tauri dev` → navigate to `/spike-d` → four tabs render
- Each tab shows its respective component with functional interactions
- Tab switching destroys previous CM6 instances (no memory leaks)
- `pnpm check` passes
- `cargo check` passes in `src-tauri/`
- No CSP errors in Safari Web Inspector across all tabs

---

## Notes

- **GitButler does NOT use CodeMirror 6** — they use Shiki for syntax highlighting and Lexical for rich text. mongit's CM6 integration is a novel pattern for Tauri + Svelte 5.
- **CM6 `style-mod` library** injects `<style>` elements dynamically — this is the sole reason `style-src 'unsafe-inline'` is required. No `eval()` is used.
- **Debouncer is not Send** — the `notify-debouncer-full` Debouncer type cannot be moved across threads. Wrap in `Arc<Mutex<>>` and store in Tauri `State<>`.
- The spike dashboard at `/spike-d` is **temporary** — it will be removed before MVP. Components will be extracted and integrated into real pages.
- Hardcoded sample data is intentional — real git2 diff integration is Spike C's scope.
