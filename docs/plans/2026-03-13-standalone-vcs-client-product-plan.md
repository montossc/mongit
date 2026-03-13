# Product Plan: Standalone Git Client for macOS

**Date:** March 13, 2026
**Status:** Research complete, ready for technical spikes
**Business Model:** Free app (open source considered)

---

## Executive Summary

Build a free, standalone macOS Git client targeting solo power developers. The product occupies the empty "deep features + native performance" quadrant that no current client fills. It ports JetBrains' best VCS ideas (4-pane log, line-level staging, 3-pane merge, blame-as-navigation) into a lightweight Tauri 2.0 desktop app.

---

## Product Thesis

**Solo power developers want a Git client that is simultaneously deep and fast, but every existing option forces a tradeoff:**

| Client | Problem |
|--------|---------|
| GitHub Desktop | Too simple for professionals |
| GitKraken | Electron-slow, subscription fatigue |
| Tower | Paid subscription, macOS-only (acceptable) |
| Fork | Good but limited depth |
| Sublime Merge | Fast but no team features, limited staging |
| JetBrains VCS | Best UX, but locked inside a JVM IDE |

**Our answer:** JetBrains-grade VCS experience in a standalone, free, native-speed macOS app.

---

## Tech Stack Decision

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Desktop framework | Tauri 2.0 | 5-20MB binary, 60-130MB RAM, proven by GitButler |
| Backend | Rust | Memory safety, performance, `git2` crate bindings |
| Frontend | Svelte 5 + SvelteKit + TypeScript | Smaller bundle, GitButler precedent, native-feel performance |
| Git engine | Hybrid: libgit2 reads + bundled git writes | GitHub Desktop pattern, hooks/signing support |
| Diff/editor | CodeMirror 6 | Virtual scrolling, syntax highlighting, battle-tested |
| Commit graph | WebGL/Canvas | GPU-accelerated, handles 100k+ commits |

See: `docs/research/2026-03-13-standalone-desktop-architecture-recommendation.md`
See: `docs/research/2026-03-13-git-engine-architecture-recommendation.md`

---

## Roadmap

### Phase 1: Foundation (Weeks 1-4)
**Goal:** Technical spikes and architecture validation

- [ ] Set up Tauri 2.0 project with Rust backend + React/Svelte frontend
- [ ] Spike: libgit2 (`git2` crate) — status, diff, log, blame
- [ ] Spike: Bundled git binary — commit, push, rebase
- [ ] Spike: WebGL/Canvas commit graph rendering with 10k+ commits
- [ ] Spike: CodeMirror 6 diff viewer in Tauri webview
- [ ] Spike: macOS FSEvents file watching for change tracking
- [ ] Design system: tokens, colors, typography, components
- [ ] Architecture: IPC protocol between frontend and Rust backend

### Phase 2: MVP Build (Weeks 5-16)
**Goal:** Shippable MVP with core git operations

- [ ] Repo home (open repo, recent repos, status overview)
- [ ] Commit graph (visual DAG, branch labels, commit details)
- [ ] Local changes workspace (file list, hunk/line staging, diff viewer)
- [ ] Commit authoring (message, amend, push)
- [ ] Branch operations (create, switch, delete, fetch, pull, push)
- [ ] Conflict resolution (3-pane merge editor, per-chunk accept/ignore)
- [ ] UX: keyboard shortcuts, command palette, undo, themes
- [ ] macOS packaging (dmg, Homebrew cask)

### Phase 3: V1.0 Release (Weeks 17-28)
**Goal:** Full solo developer workflow

- [ ] Enhanced commit graph (IntelliSort, collapse/expand, search, context menus)
- [ ] History investigation (file history, blame, compare commits)
- [ ] Advanced branch ops (merge, rebase, cherry-pick, interactive rebase)
- [ ] Stash management
- [ ] Safety net / undo system
- [ ] Tag management
- [ ] Public launch (GitHub, Hacker News, Homebrew)

### Phase 4: V1.1 Power-Up (Weeks 29-40)
**Goal:** Differentiation features

- [ ] Work buckets (named changelists)
- [ ] Shelf (persistent WIP storage)
- [ ] Advanced blame (hide revision, revision range)
- [ ] Pre-commit check pipeline
- [ ] Workspace context per branch
- [ ] Log index / full-text search

### Phase 5: V2.0 Expansion (Weeks 41+)
**Goal:** Intelligence and reach

- [ ] Stacked diffs (visual dependency chain)
- [ ] AI workflow intelligence (semantic conflicts, diff narration, rebase advisor)
- [ ] Windows support (Tauri WebView2)
- [ ] Optional cloud features (settings sync, backup)

---

## Key Design Principles

1. **Ambient status** — VCS state is always visible, never hidden behind a panel
2. **Operation preview** — Show what will happen before every destructive action
3. **Undo everything** — Every mutation is reversible with one action
4. **Keyboard-first** — Every action reachable via shortcut or command palette
5. **Progressive disclosure** — Clean surface for beginners; depth for experts
6. **Premium calm** — Sharp typography, generous spacing, no visual noise

---

## UX Tone

- **Premium, sharp** — like Raycast, Linear, or Arc
- **Keyboard-first** — CMD+K everything
- **Calm power** — confidence without anxiety
- **Operation previews** — "this will happen" before execution
- **Undo/recovery** — fearless git operations

---

## Research Documents

| Document | Path |
|----------|------|
| JetBrains VCS UI Analysis | `docs/research/2026-03-13-jetbrains-vcs-ui-analysis.md` |
| Competitor Matrix | `docs/research/2026-03-13-version-control-client-competitor-matrix.md` |
| Product Opportunities | `docs/research/2026-03-13-product-opportunities-for-new-vcs-client.md` |
| Technical Subsystems & Risk Map | `docs/research/2026-03-13-technical-subsystems-and-risk-map.md` |
| Desktop Architecture Recommendation | `docs/research/2026-03-13-standalone-desktop-architecture-recommendation.md` |
| Git Engine Architecture | `docs/research/2026-03-13-git-engine-architecture-recommendation.md` |
| Solo Power Developer Persona | `docs/research/2026-03-13-solo-power-developer-persona-and-positioning.md` |
| V1 Feature Scope | `docs/research/2026-03-13-v1-feature-scope-for-macos-git-client.md` |

---

## Next Steps

1. ~~**Choose frontend framework**~~ — **Svelte 5 + SvelteKit** (locked March 13, 2026)
2. **Run technical spikes** — see `docs/plans/2026-03-13-phase-1-technical-spikes.md`
3. **Design system** — establish visual language before building features
4. **MVP implementation** — follow Phase 2 checklist

---

## Risk Register

| Risk | Severity | Mitigation |
|------|----------|------------|
| Commit graph rendering at scale | HIGH | Budget 4-6 weeks; consider `d3-dag` or custom WebGL |
| WebView inconsistency across platforms (future) | HIGH | Automated cross-platform testing; avoid bleeding-edge CSS |
| Rust hiring/learning curve | MEDIUM | Split team: web devs own UI, 1-2 Rust engineers own backend |
| libgit2 hook/signing gaps | MEDIUM | Hybrid pattern; always shell out for writes |
| Scope creep into team features | MEDIUM | Strict persona discipline; V2+ for team features |
| Performance regression as features grow | MEDIUM | Continuous benchmarking from MVP; perf budget per operation |
