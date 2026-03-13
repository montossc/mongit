# Handoff: mongit Phase 1 Planning

**Date:** 2026-03-13
**Branch:** (no git repo initialized yet)
**Commit:** N/A

## Done

- All research docs complete (8 docs in `docs/research/`)
- Product plan complete (`docs/plans/2026-03-13-standalone-vcs-client-product-plan.md`)
- **Frontend decision locked: Svelte 5 + SvelteKit** — updated in product plan line 38
- Product plan "Next Steps" section updated to reflect Svelte decision and spike plan reference
- Technical research completed for spike plan:
  - Tauri 2.0 + SvelteKit scaffold pattern (adapter-static, SSR disabled)
  - CodeMirror 6 integration in Svelte 5 (vanilla JS API, `$effect` rune pattern)
  - Commit graph rendering: Custom Canvas 2D recommended (not WebGL/DOM)
  - git2 crate v0.20.4 — supports status/diff/log/blame; no hooks/signing (shell out)
  - Tauri 2.0 IPC: commands (req/resp), events (push), channels (streaming)
  - File watching: `notify` v8.2.0 + `notify-debouncer-full` in Rust

## In Progress

- **Phase 1 Technical Spike Plan** — research complete, document not yet written
  - Target path: `docs/plans/2026-03-13-phase-1-technical-spikes.md`
  - All research findings gathered from scout agent

## Remaining

- Write the spike plan document with 4 spikes:
  - Spike A: Tauri + SvelteKit scaffold + IPC protocol
  - Spike B: Commit graph renderer (Canvas 2D, 10k+ commits)
  - Spike C: Git engine hybrid (git2 reads + bundled git writes)
  - Spike D: CodeMirror 6 diff/merge + FSEvents file watching
- Initialize git repo
- Create beads for the 4 spikes
- Execute spikes
- Then decompose MVP into vertical slices

## Files Touched

- `docs/plans/2026-03-13-standalone-vcs-client-product-plan.md` — locked Svelte 5 in tech stack table, updated Next Steps section

## Decisions

- **Svelte 5 + SvelteKit over React**: Smaller bundle, GitButler precedent, native-feel performance in Tauri
- **Custom Canvas 2D for commit graph**: DOM breaks at 1000+ nodes, WebGL overkill; Canvas 2D is what major git clients use
- **git2 with vendored-libgit2**: No system dependency needed; shell out to bundled git for hooks/signing
- **notify crate in Rust for file watching**: FSEvents on macOS, emit Tauri events to frontend

## Blockers

- No git repo initialized yet (blocks bead creation and commits)

## Key Research Findings

- SvelteKit needs `adapter-static` + `ssr=false` for Tauri
- CodeMirror 6 needs `style-src 'unsafe-inline'` in Tauri CSP
- git2 `Repository` is not Send+Sync — use `Arc<Mutex<>>` or open per-call
- `notify` watcher must be kept alive in Tauri `State<>`
- GitButler uses CSS flexbox for graph (simple branch view), not Canvas

## Resume Instructions

1. Write the Phase 1 spike plan to `docs/plans/2026-03-13-phase-1-technical-spikes.md`
2. Initialize git repo (`git init`)
3. Create beads for the 4 spikes
4. Begin spike execution
