---
purpose: Current project state, active decisions, blockers, and position tracking
updated: 2026-03-15
---

# State

## Current Position

**Active Work:** Phase 1 — Foundation spikes + early MVP backend
**Status:** Branch operations shipped, spikes B/C/D pending
**Started:** 2026-03-13
**Phase:** Foundation → MVP overlap

## Recent Completed Work

| Date       | Title                        | Summary                                                      |
| ---------- | ---------------------------- | ------------------------------------------------------------ |
| 2026-03-15 | MVP Branch Operations (bd-2uj) | create/switch/delete/fetch/pull/push with typed BranchOpError, 71 tests |
| 2026-03-15 | GitResolver spike (bd-2na)   | Deterministic git binary resolution, PATH/env/version checks |
| 2026-03-14 | Project scaffold             | Tauri 2.0 + SvelteKit + Rust backend, all builds passing     |
| 2026-03-13 | Research & product plan      | 8 research docs, product plan locked, frontend decision made |
| 2026-03-14 | Git repo + remote            | Initialized, pushed to github.com/montossc/mongit            |

## Active Decisions

| Date       | Decision                              | Rationale                                                   |
| ---------- | ------------------------------------- | ----------------------------------------------------------- |
| 2026-03-13 | Svelte 5 + SvelteKit over React       | Smaller bundle, GitButler precedent                         |
| 2026-03-13 | Canvas 2D for commit graph            | DOM breaks at 1000+ nodes, WebGL overkill                   |
| 2026-03-13 | git2 reads + bundled git writes       | GitHub Desktop pattern; git2 lacks hooks/signing            |
| 2026-03-13 | notify crate for file watching        | Native FSEvents on macOS                                    |
| 2026-03-14 | CSP includes unsafe-inline for styles | Required for CodeMirror 6                                   |

## Blockers

| Item    | Blocker | Since | Notes |
| ------- | ------- | ----- | ----- |
| (none)  | —       | —     | —     |

## Open Questions

| Question                                        | Context                    | Priority |
| ----------------------------------------------- | -------------------------- | -------- |
| Canvas 2D vs WebGL for 100k+ commit graphs?     | Spike B will validate      | High     |
| Bundle git binary or rely on system git?         | Spike C will validate      | Medium   |

## Context Notes

### Technical

- Tauri dev server on port 1420 (strictPort, HMR)
- SvelteKit: adapter-static, ssr=false, prerender=true
- Rust: vendored-libgit2 (no system dep needed)
- Release profile: strip + lto + codegen-units=1 + panic=abort

### Process

- Git remote: https://github.com/montossc/mongit.git
- Main branch: `main`
- Ask before committing/pushing

## Next Actions

1. [ ] Execute Spike B: Canvas 2D commit graph renderer
2. [ ] Execute Spike D: CodeMirror 6 diff + FSEvents
3. [ ] Decompose MVP into vertical slices after spikes complete
4. [ ] Design system tokens

## Session Handoff

**Last Session:** 2026-03-15
**Next Session Priority:** Execute remaining spikes (B, D); design system tokens
**Known Issues:** None currently blocking

---

_Update this file at the end of each significant session or when state changes._
