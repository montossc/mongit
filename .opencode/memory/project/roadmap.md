---
purpose: Project roadmap with phases, milestones, and bead organization
updated: 2026-03-14
---

# Roadmap

## Overview

| Phase      | Goal                                          | Status      | Weeks  |
| ---------- | --------------------------------------------- | ----------- | ------ |
| Foundation | Technical spikes and architecture validation  | In Progress | 1-4    |
| MVP        | Shippable MVP with core git operations        | Not Started | 5-16   |
| V1.0       | Full solo developer workflow                  | Not Started | 17-28  |
| V1.1       | Differentiation features (power-ups)          | Not Started | 29-40  |
| V2.0       | Intelligence and platform expansion           | Not Started | 41+    |

---

## Phase 1: Foundation (Weeks 1-4)

**Goal:** Technical spikes and architecture validation

**Success Criteria:**

- [x] Tauri + SvelteKit scaffold with working IPC
- [ ] Canvas 2D commit graph rendering 10k+ commits at 60fps
- [ ] git2 hybrid engine (reads via git2, writes via bundled git)
- [ ] CodeMirror 6 diff viewer + FSEvents file watching
- [ ] Design system tokens established

**Spikes:**

| Spike | Title                                           | Status      |
| ----- | ----------------------------------------------- | ----------- |
| A     | Tauri + SvelteKit scaffold + IPC protocol       | Done        |
| B     | Commit graph renderer (Canvas 2D, 10k+ commits) | Not Started |
| C     | Git engine hybrid (git2 reads + git writes)     | Not Started |
| D     | CodeMirror 6 diff/merge + FSEvents watching     | Not Started |

---

## Phase 2: MVP Build (Weeks 5-16)

**Goal:** Shippable MVP with core git operations

**Success Criteria:**

- [ ] Repo home (open repo, recent repos, status overview)
- [ ] Commit graph (visual DAG, branch labels, commit details)
- [ ] Local changes workspace (file list, hunk/line staging, diff viewer)
- [ ] Commit authoring (message, amend, push)
- [ ] Branch operations (create, switch, delete, fetch, pull, push)
- [ ] Conflict resolution (3-pane merge editor)
- [ ] Keyboard shortcuts + command palette
- [ ] macOS packaging (dmg, Homebrew cask)

---

## Phase 3: V1.0 Release (Weeks 17-28)

**Goal:** Full solo developer workflow

- Enhanced commit graph (IntelliSort, collapse/expand, search)
- History investigation (file history, blame, compare commits)
- Advanced branch ops (merge, rebase, cherry-pick, interactive rebase)
- Stash management
- Safety net / undo system
- Tag management

---

## Phase 4: V1.1 Power-Up (Weeks 29-40)

**Goal:** Differentiation features

- Work buckets (named changelists)
- Shelf (persistent WIP storage)
- Advanced blame (hide revision, revision range)
- Pre-commit check pipeline
- Workspace context per branch
- Log index / full-text search

---

## Phase 5: V2.0 Expansion (Weeks 41+)

**Goal:** Intelligence and platform expansion

- Stacked diffs (visual dependency chain)
- AI workflow intelligence (semantic conflicts, diff narration, rebase advisor)
- Windows support (Tauri WebView2)
- Optional cloud features (settings sync, backup)

---

## Risk Register

| Risk                               | Severity | Mitigation                                            |
| ---------------------------------- | -------- | ----------------------------------------------------- |
| Commit graph rendering at scale    | HIGH     | Budget 4-6 weeks; Canvas 2D spike first               |
| libgit2 hook/signing gaps          | MEDIUM   | Hybrid pattern; always shell out for writes            |
| Scope creep into team features     | MEDIUM   | Strict persona discipline; V2+ for team features       |
| Performance regression over time   | MEDIUM   | Continuous benchmarking from MVP; perf budget per op   |

---

_Update this file when phases complete or roadmap changes._
