# Main Route Event Bridge + Scoped Refresh Guards Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use skill({ name: "executing-plans" }) to implement this plan task-by-task.

**Goal:** Wire `repo-changed` events into the main graph route so only the active repo surface refreshes, with explicit guards for synthetic and non-Tauri modes.

**Architecture:** Keep the refresh bridge route-local in `src/routes/+page.svelte` by reusing the existing `loadRepo()` controller. Add narrow guard/coalescing logic in that route only (no global invalidation or cross-route bus). Preserve watcher monitor usage as diagnostics/manual trigger path, not as refresh orchestrator.

**Tech Stack:** Svelte 5 (runes), SvelteKit, Tauri event API (`@tauri-apps/api/event`), existing route-local repo loading via Tauri IPC.

---

## Must-Haves

**Goal:** Main route visibly refreshes repo-backed graph/status state on watcher events without full-page/full-app invalidation.

### Observable Truths

1. When `src/routes/+page.svelte` has a real repo loaded in Tauri mode, `repo-changed` events trigger route-local refresh through existing repo load flow.
2. When synthetic mode is active, watcher events do not clobber synthetic graph state.
3. When Tauri IPC/event runtime is unavailable, no watcher listener or refresh logic runs and no runtime error appears.
4. No global invalidation pattern is introduced (`invalidateAll`, full-page reload, global refresh bus).

### Required Artifacts

| Artifact | Provides | Path |
|----------|----------|------|
| Main route event bridge + cleanup | `repo-changed` listener lifecycle scoped to route mount/unmount | `src/routes/+page.svelte` |
| Main route refresh guards | Gating by Tauri availability, real repo mode, and active load behavior | `src/routes/+page.svelte` |
| Bead implementation plan | Executable task sequence and verification matrix | `.beads/artifacts/bd-htm.3/plan.md` |

### Key Links

| From | To | Via | Risk |
|------|----|-----|------|
| `src/routes/+page.svelte` | backend watcher event stream | `listen('repo-changed', ...)` | Missing cleanup could leak duplicate listeners |
| `repo-changed` handler | route refresh path | `loadRepo()` reuse | Unguarded refresh could run during synthetic mode |
| watcher diagnostics (`WatcherMonitor`) | manual validation workflow | start/stop watcher controls | Could be mistaken as primary refresh target if route bridge is not explicit |

### Task Dependencies

- **Task 1** has no dependencies.
- **Task 2** depends on Task 1.
- **Task 3** depends on Tasks 1 and 2.

**Wave 1:** Task 1  
**Wave 2:** Task 2  
**Wave 3:** Task 3 (verification/review checkpoint)

---

## Task 1: Add main-route `repo-changed` event bridge

**Files:**
- Modify: `src/routes/+page.svelte`

**Implementation Steps:**
1. Add route-local subscription to `repo-changed` in route lifecycle (`onMount`) using Tauri event listener pattern and explicit unlisten cleanup.
2. Route listener callback must call existing route refresh controller (`loadRepo()` or direct equivalent), not introduce new global state/store bus.
3. Keep listener registration gated so it only activates when Tauri runtime is available.

**Verification Commands + Expected Outcomes:**
- `pnpm check`  
  Expected: `svelte-check` exits 0 with no new type/runtime diagnostics.
- `rg "listen\('repo-changed'|listen<void>\('repo-changed'" src/routes/+page.svelte`  
  Expected: exactly one route-local listener registration in main route.
- `rg "unlisten|return \(\) =>" src/routes/+page.svelte`  
  Expected: explicit listener cleanup on teardown is present.

---

## Task 2: Add scoped refresh guards (no global invalidation)

**Files:**
- Modify: `src/routes/+page.svelte`

**Implementation Steps:**
1. Add explicit handler guards so watcher-driven refresh only runs when:
   - Tauri mode is active,
   - a real repo path is active,
   - route is not in synthetic-data mode.
2. Ensure repeated watcher events during active load remain understandable (e.g., simple in-route guard/coalescing condition), without introducing cross-route orchestration.
3. Confirm no use of `invalidateAll`, `window.location.reload`, or app-wide/global refresh constructs.

**Verification Commands + Expected Outcomes:**
- `pnpm check`  
  Expected: passes with no new errors.
- `rg "invalidateAll|window\.location\.reload|dispatch\(|global.*refresh|refreshBus" src/routes/+page.svelte src/lib/stores/watcher.svelte.ts src/routes/spike-d/+page.svelte`  
  Expected: no new global invalidation/reload mechanisms introduced for this bead.
- `rg "synthetic|isTauri|repoPath|loading" src/routes/+page.svelte`  
  Expected: guard conditions are explicit and readable in the route handler path.

---

## Task 3: Verification + review checkpoint (manual + static)

**Type:** Verification/Review Checkpoint (no new feature logic)

**Files:**
- Review: `src/routes/+page.svelte`
- Review: `src/lib/components/WatcherMonitor.svelte`
- Review: `.beads/artifacts/bd-htm.3/plan.md`

**Checkpoint Steps:**
1. Run frontend verification and static guard assertions.
2. Perform manual end-to-end flow:
   - Open main route with real repo.
   - Start watcher from watcher monitor diagnostics.
   - Edit tracked file.
   - Confirm main graph/status surface refreshes without pressing Open again.
   - Stop watcher and re-edit; confirm no refresh.
3. Review diff scope remains constrained to main route event bridge + scoped refresh guards only.

**Verification Commands + Expected Outcomes:**
- `pnpm check`  
  Expected: passes.
- `git diff --name-only`  
  Expected: only planned bead-scope files changed (primarily `src/routes/+page.svelte`; plus this plan file for planning phase).
- `rg "repo-changed" src/routes/+page.svelte src/lib/components/WatcherMonitor.svelte`  
  Expected: main route owns refresh bridge; watcher monitor remains diagnostic listener surface.

---

## Expected File Change Set (Implementation Phase)

- `src/routes/+page.svelte` (**required**)
- `src/lib/stores/watcher.svelte.ts` (**not expected for this plan unless implementation reveals an unavoidable tiny helper; default is no change**)

## Out-of-Scope Enforcement

- No backend watcher contract changes (`src-tauri/**`).
- No `/spike-d` refresh-first architecture.
- No diff/workspace integration.
- No global/app-wide invalidation mechanisms.
