# Product Plan: mongit — Standalone Git Client for macOS

**Date:** March 13, 2026 (revised March 16, 2026)
**Status:** Foundation phase in progress — spikes A & C complete, branch ops shipped
**Business Model:** Free app (open source considered)

---

## Executive Summary

mongit is a free, standalone macOS Git client targeting solo power developers. It occupies the empty **"deep features + native performance"** quadrant that no current client fills — porting the specific JetBrains VCS features that create lock-in (3-pane merge with editable center, line-granularity staging, blame-as-navigation, visual interactive rebase, workspace context per branch) into a lightweight Tauri 2.0 desktop app, while going further with **GitButler-inspired fearless rebase conflict resolution** and a **visual worktree manager** for parallel branch workflows.

The competitive moat is the *combination*: no other free client offers native performance + JetBrains-grade staging/history + non-blocking rebase conflict resolution + visual worktree management + undo safety net + keyboard-first UX. Each individual feature exists somewhere; the combination in a free, polished package does not.

---

## Product Thesis

**Solo power developers want a Git client that is simultaneously deep and fast, but every existing option forces a tradeoff:**

| Client | Problem |
|--------|---------|
| GitHub Desktop | Too simple for professionals |
| GitKraken | Electron-slow, subscription fatigue |
| Tower | Paid subscription, limited depth |
| Fork | Good but limited depth |
| Sublime Merge | Fast but limited staging, no merge editor |
| JetBrains VCS | Best UX, but locked inside a JVM IDE |

**Our answer:** JetBrains-grade VCS experience in a standalone, free, native-speed macOS app.

### Why JetBrains Users Stay Locked In

Research into HN comments, JetBrains docs, and developer forums reveals the specific features that create switching cost:

1. **3-pane merge editor with editable center** — the strongest stickiness signal; developers can synthesize resolutions neither side proposed
2. **Line-granularity partial commits** — goes deeper than `git add -p`; surgical commits via visual toggles
3. **Blame-as-navigation** — blame gutter with hover popups, click-to-jump-to-log, annotate previous revision, hide noisy commits
4. **Visual interactive rebase** — transforms opaque `git rebase -i` into a drag-and-drop graph editor with preview
5. **Dual staging models** — changelists (forgiving mental model) or git staging area (explicit `git add` flow)
6. **Workspace context per branch** — saves/restores open files and UI state per branch; invisible but sticky
7. **Ambient VCS status everywhere** — change markers, file tree coloring, branch widget with incoming/outgoing counts
8. **Inline gutter commit** — commit a single hunk without leaving the editor context

**These 8 features define the JetBrains-parity layer of our roadmap.** Every phase replicates, then surpasses, this specific set.

### Where mongit Goes Further

Beyond JetBrains parity, mongit introduces two competitive features no existing standalone Git client offers:

1. **Fearless Rebase Conflict Resolution** (GitButler-inspired) — non-blocking rebase that doesn't halt at conflicts; conflicted commits are marked in the graph, resolved one at a time in an isolated Edit Mode, with automatic downstream continuation and full snapshot-based undo. Transforms rebase from an anxiety-inducing operation into a calm, incremental workflow.
2. **Worktree Manager / Viewer** — visual management of git worktrees for parallel branch workflows. Create, open, inspect, switch, prune, and compare worktrees without dropping to terminal. Turns `git worktree` from an expert CLI feature into an accessible power tool.

See: `docs/research/2026-03-16-gitbutler-rebase-conflict-ux.md`

---

## Target Persona

**Solo Power Developer** — full-stack or backend, 3-10+ years experience, macOS primary, comfortable with terminal but wants visual tools for complex git operations.

**One-liner:** *"A free, premium Git client for developers who care about their commit history."*

### Pain Points We Solve

| Pain Point | Our Solution |
|-----------|-------------|
| "I changed too much in one file" | Line-granularity staging with visual toggles |
| "What happened here?" | Blame-as-navigation with revision walking |
| "I'm scared of rebase" | Visual interactive rebase with graph preview + undo |
| "Rebase conflicts stop me cold" | Non-blocking rebase: conflicted commits marked in graph, isolated resolution, automatic continuation |
| "I juggle multiple branches/tasks" | Visual worktree manager: create, switch, inspect parallel branches without terminal |
| "GitHub Desktop is too simple" | JetBrains-grade depth in every workflow |
| "GitKraken is slow and heavy" | Tauri 2.0: 5-20MB binary, <150MB RAM, <2s startup |
| "JetBrains VCS is amazing but I use VS Code" | Standalone app, identical depth |
| "I don't want another subscription" | Free, forever |

### Competitive Positioning

```
                    Deep Features
                         │
           JetBrains ────┼──── mongit
                         │           
          GitKraken ─────┤     Tower
                         │     Fork
                         │     Sublime Merge
                         │
    ─────────────────────┼─────────────────────
    Slow / Heavy         │         Fast / Native
                         │
          SourceTree ────┤     
                         │
      GitHub Desktop ────┤
                         │
                    Simple Features
```

---

## Tech Stack (Locked)

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Desktop framework | Tauri 2.0 (WKWebView) | 5-20MB binary, 60-130MB RAM, proven by GitButler |
| Backend | Rust (stable 1.94+) | Memory safety, performance, git2 crate bindings |
| Frontend | **Svelte 5 + SvelteKit** + TypeScript | Smaller bundle, GitButler precedent, runes reactivity |
| Git engine | Hybrid: **vendored-libgit2** reads + bundled git writes | GitHub Desktop pattern; hooks/signing support via shell |
| Diff/merge editor | CodeMirror 6 | Virtual scrolling, syntax highlighting, 100k+ lines |
| Commit graph | **Canvas 2D** | DOM breaks at 1000+ nodes; WebGL overkill for DAG rendering |
| File watching | notify + notify-debouncer-full | Native FSEvents on macOS |
| Build | Vite 7 + Cargo | Fast HMR, optimized production builds |
| Package manager | pnpm | Disk-efficient, strict dependency resolution |

### Key Constraints

- SvelteKit: `ssr=false`, `prerender=true` (static adapter for Tauri)
- git2 `Repository` is **not Send+Sync** — open per-call or use `Arc<Mutex<>>`
- `notify` watcher must stay alive in Tauri `State<>`
- CSP: `style-src 'unsafe-inline'` required for CodeMirror 6
- Large diffs: use `convertFileSrc()`, not JSON serialization

---

## Design Principles

1. **Ambient status** — VCS state is always visible (gutter markers, file tree colors, branch widget with up/down counts), never hidden behind a panel
2. **Operation preview** — Show what will happen before every destructive action (rebase, force push, reset)
3. **Undo everything** — Every mutation is reversible with one action; reflog-backed recovery for all operations
4. **Keyboard-first** — Every action reachable via shortcut or command palette (CMD+K); zero mouse-required workflows
5. **Progressive disclosure** — Clean surface for daily work; full depth accessible on demand
6. **Premium calm** — Sharp typography, generous spacing, no visual noise; inspired by Raycast, Linear, Arc

### UX Tone

- **Premium, sharp** — not playful, not corporate
- **Keyboard-first** — CMD+K everything
- **Calm power** — confidence without anxiety
- **Operation previews** — "this will happen" before execution
- **Undo/recovery** — fearless git operations

---

## Roadmap

### Phase 1: Foundation (Weeks 1-4)
**Goal:** Technical spikes and architecture validation

- [x] Set up Tauri 2.0 project with Rust backend + Svelte 5 frontend
- [x] Architecture: IPC protocol between frontend and Rust backend
- [x] Spike A: Tauri + SvelteKit scaffold + IPC protocol
- [x] Spike C: Git engine hybrid (git2 reads + bundled git writes)
  - GitResolver: deterministic git binary resolution, PATH/env/version checks
  - Branch operations: create/switch/delete/fetch/pull/push with typed BranchOpError (71 tests)
- [ ] Spike B: Canvas 2D commit graph rendering with 10k+ commits at 60fps
- [ ] Spike D: CodeMirror 6 diff viewer + FSEvents file watching
- [ ] Design system: tokens, colors, typography, components

### Phase 2: MVP Build (Weeks 5-16) — "Commit with Confidence"
**Goal:** Shippable MVP with core git operations and JetBrains-grade staging

**JetBrains stickiness features in MVP:** line-granularity staging, 3-pane merge editor, ambient VCS status, keyboard-first workflow

| Surface | Features | JetBrains Parity |
|---------|----------|-------------------|
| **Repo Home** | Open repo (drag-and-drop/file picker), recent repos, status overview | Branch widget with up/down counts |
| **Commit Graph** | Visual DAG (Canvas 2D), branch labels, commit details, basic filtering by branch/author, 10k+ commits at 60fps | Color-coded labels, 4-pane layout |
| **Local Changes** | File list (modified/added/deleted), hunk-level staging, **line-level staging** with visual toggles, side-by-side diff (CodeMirror 6), discard with confirmation | Partial commit at line granularity (JB pattern 2.4) |
| **Commit Authoring** | Message editor with subject/body split, message history, amend, commit+push | Pre-commit hook detection and display |
| **Branch Operations** | Create/switch/delete, branch list (local+remote), fetch/pull/push | Already shipped in Rust backend |
| **Merge Resolution** | 3-pane merge editor: left=local (read-only), **center=editable result** (full CodeMirror 6), right=remote (read-only), per-chunk accept/ignore, auto-apply non-conflicting, one-click simple conflict resolve | JB pattern 2.7 — the #1 stickiness feature |
| **UX Foundation** | Keyboard shortcuts for all ops, command palette (CMD+K), operation preview, undo notification, dark/light theme (system), native macOS menu bar | Ambient VCS status (gutter markers, file tree colors) |
| **Packaging** | dmg, Homebrew cask | — |

**MVP Validation Criteria:**
- [ ] Open a local repo and see commit graph with 10k+ commits at 60fps
- [ ] Stage/unstage individual lines and hunks
- [ ] Commit with message, amend, and push
- [ ] Resolve merge conflicts in 3-pane editor with editable center
- [ ] All operations accessible via keyboard
- [ ] Startup < 2 seconds, RAM < 150MB, binary < 25MB

### Phase 3: V1.0 Release (Weeks 17-28) — "Full Solo Workflow"
**Goal:** Complete solo developer workflow with history investigation and history editing

**JetBrains stickiness features added:** blame-as-navigation, visual interactive rebase

**Competitive features added:** fearless rebase conflict resolution (GitButler-inspired)

| Surface | Features | JetBrains Parity |
|---------|----------|-------------------|
| **Enhanced Graph** | IntelliSort-style merge display, collapse/expand linear branches, search by message/author/hash, multi-branch filtering, context menus | JB log tab parity |
| **History Investigation** | File history with rename tracking, **blame gutter** (author+date per line), hover popup (commit message, clickable hash jumps to log), **"Annotate Previous Revision"** navigation, compare any two commits | JB pattern 2.8 — blame-as-navigation |
| **History Editing** | Merge/rebase/cherry-pick, **visual interactive rebase** (drag-to-reorder, pick/squash/fixup/drop, inline message editor, **graph preview before execution**) | JB pattern 2.6 — visual rebase editor |
| **Stash Management** | Stash with message, stash list with preview, apply/pop/drop | — |
| **Safety Net** | Undo last commit/rebase/merge/discard, **operation history with snapshots**, reflog-backed recovery | "Undo anything" (top acquisition hook) |
| **Fearless Rebase** | **Pre-flight conflict preview** (show which commits will conflict before execution), **non-blocking rebase** (don't halt at conflicts; mark conflicted commits in graph), **per-commit isolated resolution** (Edit Mode: stash everything else, resolve one commit at a time), **Save and Exit / Cancel** (two clean exit paths), **automatic downstream continuation** (resolved commit triggers re-rebase of downstream chain) | GitButler-inspired — no other standalone client offers this |
| **Tag Management** | Create (lightweight + annotated), delete, push, tag list in branch panel | — |
| **Public Launch** | GitHub, Hacker News, Homebrew cask | — |

### Phase 4: V1.1 Power-Up (Weeks 29-40) — "JetBrains Parity"
**Goal:** Complete JetBrains feature parity + differentiation + power workflows

**JetBrains stickiness features added:** dual staging models (work buckets), workspace context per branch, inline gutter commit

**Competitive features added:** worktree manager / viewer

| Surface | Features | JetBrains Parity |
|---------|----------|-------------------|
| **Work Buckets** | Named groupings of changes (reinterpreted changelists), files assigned to buckets, commit from specific bucket, move files/hunks between buckets, persisted across sessions | JB pattern 2.3 — dual staging models |
| **Shelf** | Named WIP storage, selective shelving (files/hunks), unshelve with conflict handling, shelf list with diff preview | JB WIP management |
| **Workspace Context** | Save/restore UI state per branch (selected files, scroll positions, panel layout), automatic on branch checkout | JB pattern 2.10 — strongest retention feature |
| **Advanced Blame** | "Hide Revision" (suppress noisy reformats), blame for revision range, performance optimization for large files | JB pattern 2.8 extensions |
| **Inline Gutter Commit** | Click change marker, mini-toolbar, commit that hunk without leaving context | JB pattern 2.2 |
| **Pre-Commit Checks** | Detect and run `.git/hooks/pre-commit`, show results inline, allow override | — |
| **Log Search** | Full-text search across commit messages, search by file path/date range, fast incremental indexing | — |
| **Worktree Manager** | List all worktrees with branch + dirty/clean state, **create/remove/prune worktrees**, open worktree in Finder/editor, switch context between worktrees, compare worktree status side-by-side | No standalone Git client offers visual worktree management |

### Phase 5: V2.0 Expansion (Weeks 41+) — "Beyond JetBrains"
**Goal:** Features JetBrains doesn't have — AI intelligence, stacked diffs, platform expansion

| Surface | Features | Competitive Edge |
|---------|----------|-----------------|
| **Stacked Diffs** | Visual dependency chain for branch stacks, rebase-on-merge management, stack status overview | Tower (basic), Sapling (CLI-only) — no GUI client nails this |
| **AI Workflow Intelligence** | Semantic conflict detection, diff narration (explain changes in English), interactive rebase advisor, commit message generation | GitKraken AI is bolt-on; ours is workflow-native |
| **Windows Support** | Tauri 2.0 WebView2 backend, Windows-native installer (MSI), platform-specific shortcuts | — |
| **Optional Cloud** | Settings sync, backup of work buckets and shelves, opt-in analytics | — |

### Architectural Considerations (Designed-for from Day 1)

These features require architectural support laid down early:

| Feature | Day-1 Architecture Need |
|---------|------------------------|
| Stacked diffs | Branch metadata model must support parent-child relationships; graph renderer must handle stack visualization |
| AI workflows | Diff computation must be structured (not just text); semantic analysis hooks in the diff pipeline |
| Windows support | No macOS-only APIs in the frontend; Tauri's cross-platform abstractions used consistently |
| Cloud sync | State serialization format designed for sync; no local-only state assumptions |
| Fearless rebase | Operation snapshot model from MVP (undo system); conflict metadata storage; graph renderer must support "conflicted commit" visual state; Edit Mode requires workspace isolation |
| Worktree manager | git2 worktree APIs (`Repository::worktrees()`); IPC commands for worktree CRUD; UI state model must be worktree-aware (not assume single working directory) |

---

## Product Surfaces (16 Screens/Panels)

Derived from JetBrains VCS analysis + competitive differentiation features:

| # | Surface | Phase |
|---|---------|-------|
| 1 | **Commit Tool Window** — staged/unstaged file trees, message field with history, pre-commit checks, diff preview | MVP |
| 2 | **Diff Viewer (2-pane)** — side-by-side or unified, chunk accept buttons, whitespace modes, sync scrolling | MVP |
| 3 | **3-Pane Merge Editor** — local (read-only), editable result (full CodeMirror 6), remote (read-only) | MVP |
| 4 | **Commit Graph** — branch topology, color-coded labels, filterable, IntelliSort, collapse linear branches | MVP / V1.0 |
| 5 | **Branch Widget** — current branch in header, up/down commit counts, click opens branch popup | MVP |
| 6 | **Partial Commit / Hunk Selector** — chunk checkboxes in diff, per-line toggle, split chunks | MVP |
| 7 | **Push Dialog** — commit list preview, editable remote target, tag options, force push guard | MVP |
| 8 | **Branches Popup** — recent/local/remote/tags, prefix-grouped, per-branch context menu, search | MVP / V1.0 |
| 9 | **Interactive Rebase Editor** — commit list, drag reorder, pick/reword/squash/fixup/drop, graph preview | V1.0 |
| 10 | **File History Tab** — commit list per file, diff per revision, rename tracking | V1.0 |
| 11 | **Blame Gutter** — per-line author+date, hover popup, click-to-jump-to-log, annotate previous, hide revision | V1.0 / V1.1 |
| 12 | **Gutter Change Markers** — colored bars (new/modified/deleted), click for inline commit/staging | V1.1 |
| 13 | **Shelf / Stash Panel** — named shelves, diff preview, unshelve with conflict handling | V1.1 |
| 14 | **Code Vision Author Hints** — inlay hints showing last author per function/class | V1.1 |
| 15 | **Fearless Rebase Flow** — pre-flight conflict preview, conflicted commit markers in graph, per-commit Edit Mode, save/continue/cancel, automatic downstream continuation | V1.0 |
| 16 | **Worktree Manager** — list all worktrees with branch + status, create/remove/prune, open in Finder/editor, switch context, compare | V1.1 |

---

## Risk Register

| Risk | Severity | Mitigation |
|------|----------|------------|
| Commit graph rendering at scale (10k+ commits) | HIGH | Budget 4-6 weeks; Canvas 2D spike (B) validates approach |
| 3-pane merge editor complexity | HIGH | CodeMirror 6 + careful state management; spike early in MVP |
| Line-granularity staging UX | HIGH | Reference JetBrains "Split Chunks" interaction; prototype in spike D |
| libgit2 hook/signing gaps | MEDIUM | Hybrid pattern locked — shell out for all writes |
| Scope creep into team features | MEDIUM | Strict persona discipline; V2+ for team features |
| Performance regression as features grow | MEDIUM | Continuous benchmarking from MVP; perf budget per operation |
| CodeMirror 6 CSP restrictions | LOW | `style-src 'unsafe-inline'` already configured in tauri.conf.json |
| Fearless rebase workflow complexity | HIGH | Requires conflict metadata model, operation snapshots, graph UI for conflicted commits, and continue/cancel lifecycle; reference GitButler's `cherry_pick.rs` implementation; budget spike + iteration time |
| Worktree management edge cases | MEDIUM | git2 supports worktree listing/creation; main risk is UX for status display and lifecycle (prune, linked worktree deletion) |

---

## Explicitly Out of Scope (All Versions)

| Feature | Reason |
|---------|--------|
| PR review inside the app | Separate concern; browser is adequate |
| Issue tracker integration | Not relevant for solo power dev persona |
| Enterprise SSO/SAML | Not the target audience |
| Multi-repo project views | Complex; address in V3+ if demand exists |
| Plugin/extension system | Focus on core quality first |
| Linux support | Small market for this persona; V3+ |
| SVN/Mercurial support | Git-only product |
| Built-in terminal | Not a productivity multiplier for target persona |

---

## Business Model: Free App

- Open source (build community, attract contributors)
- Optional cloud sync / backup (future paid tier)
- Optional team features (future paid tier)
- Sponsorships / GitHub Sponsors
- The app itself builds reputation and talent pipeline

**Why free works:** The target user values time over money. A free app with excellent UX builds word-of-mouth faster than a paid app. The competitive landscape shows that free + good beats paid + mediocre.

---

## Research Documents

| Document | Path |
|----------|------|
| JetBrains VCS UI Analysis | `docs/research/2026-03-13-jetbrains-vcs-ui-analysis.md` |
| Competitor Matrix | `docs/research/2026-03-13-version-control-client-competitor-matrix.md` |
| Product Opportunities | `docs/research/2026-03-13-product-opportunities-for-new-vcs-client.md` |
| Technical Subsystems & Risk Map | `docs/research/2026-03-13-technical-subsystems-and-risk-map.md` |
| Desktop Architecture | `docs/research/2026-03-13-standalone-desktop-architecture-recommendation.md` |
| Git Engine Architecture | `docs/research/2026-03-13-git-engine-architecture-recommendation.md` |
| Solo Power Developer Persona | `docs/research/2026-03-13-solo-power-developer-persona-and-positioning.md` |
| V1 Feature Scope | `docs/research/2026-03-13-v1-feature-scope-for-macos-git-client.md` |
| Bundled Git Strategy | `docs/research/2026-03-bundled-git-strategy.md` |
| GitButler Rebase Conflict UX | `docs/research/2026-03-16-gitbutler-rebase-conflict-ux.md` |

---

## Next Steps

1. **Execute Spike B** — Canvas 2D commit graph (10k+ commits at 60fps)
2. **Execute Spike D** — CodeMirror 6 diff viewer + FSEvents file watching
3. **Design system** — establish tokens, colors, typography before MVP build
4. **Decompose MVP** — vertical slices after spikes validate the approach
