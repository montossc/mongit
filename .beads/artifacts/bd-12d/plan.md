# Spike B — Canvas Commit Graph Engine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `skill({ name: "executing-plans" })` to implement this plan task-by-task.

**Goal:** Turn the existing Canvas 2D commit graph spike into a trustworthy production baseline by moving validation to `/spike-b`, preserving interaction behavior, proving deterministic layout, and capturing explicit performance evidence on a real 10k+ commit repository.

**Architecture:** Build on the existing `src/lib/graph/*` foundation rather than redesigning it. Keep data/layout, render, hit-testing, and detail-panel concerns separated; use the dedicated route as the long-lived validation surface while keeping `/` free for repo-home work.

**Tech Stack:** Svelte 5, SvelteKit, TypeScript, Tauri IPC, Canvas 2D, existing `scripts/perf-bench.ts` benchmark script.

**Discovery Level:** 0 — internal hardening of existing code paths; no new dependency or external API decision is required.

**Context Budget:** ~45-50%. Execute in 4 waves aligned to existing child beads (`bd-12d.1`, `bd-12d.2`, `bd-12d.3`) to keep each execution slice small.

---

## Planning Notes

- `br show bd-12d` confirms the bead is active and has a PRD; `plan.md` did not exist before this plan.
- No `figma-design-spec.json` exists for `bd-12d`, so no `[UI]` / `[Logic]` split is needed.
- Memory tooling was unavailable in this session (`FTS5 not available`, `unable to open database file`), so this plan is based on PRD + codebase evidence + git history.
- Git history shows prior graph work and fixes in `bd-15p`, especially around edge culling, keyboard navigation, and performance hardening:
  - `9f1113a feat(bd-15p): Wave 1 — commit log IPC + graph layout engine`
  - `fe4536f feat(bd-15p): Wave 2 — Canvas 2D renderer + virtual scrolling`
  - `19e89ba feat(bd-15p): Wave 3 — hit testing, context menu, FPS overlay`
  - `55d4088 fix(bd-15p): critical bug fixes + performance optimizations`
  - `faa9be6 fix(bd-15p): code review fixes — edge culling, keyboard nav, security hardening`
- `package.json` has no frontend test runner. Do **not** add Vitest/Jest/Playwright without explicit approval. Use zero-dependency regression scripts + existing verification commands + manual benchmark flows.
- `scripts/perf-bench.ts` currently duplicates layout logic; that duplication is a likely drift risk and should be addressed during implementation planning/execution.

## Must-Haves

**Goal:** A dedicated `/spike-b` validation surface proves the existing commit graph foundation is deterministic, responsive, and reusable at target scale.

### Observable Truths

1. User can open `/spike-b` and validate the graph using either synthetic data or a real repository path.
2. Repeated loads of the same repository history produce the same lane assignment and segment topology.
3. A real 10k+ commit repository scrolls with practically smooth performance and visible metrics.
4. Selection, hover, context menu, keyboard navigation, and commit detail inspection all still work after the route move.
5. The root route is no longer the canonical home of the graph spike surface.
6. The graph modules remain reusable for downstream commit-graph productization rather than becoming route-coupled spike code.

### Required Artifacts

| Artifact | Provides | Path |
|---|---|---|
| Dedicated spike route | Canonical graph validation surface | `src/routes/spike-b/+page.svelte` |
| Root route shell update | Removes spike ownership from `/` | `src/routes/+page.svelte` |
| Deterministic layout engine | Stable lane assignment + segment topology | `src/lib/graph/layout.ts` |
| Graph contracts | Shared layout/render data model | `src/lib/graph/types.ts` |
| Scalable renderer | Visible-range drawing + text/ref rendering | `src/lib/graph/render.ts` |
| Canvas interaction container | Scroll, render queue, selection, keyboard nav | `src/lib/graph/GraphCanvas.svelte` |
| Accurate hit testing | Node/ref/row targeting stays aligned to rendered output | `src/lib/graph/hitTest.ts` |
| Commit inspection panel | Selected-commit detail view remains wired | `src/lib/graph/CommitDetail.svelte` |
| Metrics overlay | FPS, frame time, visible rows, lane/layout metrics | `src/lib/graph/FpsOverlay.svelte` |
| Zero-dependency regression/benchmark harness | Automated deterministic/perf checks without new test deps | `scripts/perf-bench.ts` |
| Benchmark evidence log | Persistent manual validation notes for the bead | `.beads/artifacts/bd-12d/progress.txt` |

### Key Links

| From | To | Via | Risk |
|---|---|---|---|
| `src/routes/spike-b/+page.svelte` | Tauri IPC commands | `invoke('get_commit_log')` + `invoke('get_refs')` | Route loads but cannot hydrate real graph data |
| `src/routes/spike-b/+page.svelte` | `assignLanes()` | Imported layout call for real + synthetic datasets | Same input can render different topology across runs |
| `src/lib/graph/GraphCanvas.svelte` | `src/lib/graph/render.ts` | rAF-driven `renderGraph()` calls | Scroll/hover can trigger redraw churn or blank frames |
| `src/lib/graph/GraphCanvas.svelte` | `src/lib/graph/hitTest.ts` | Mouse coordinates + `scrollTop` + lane count | Context menu / selection can target the wrong commit |
| `selectedId` state | `src/lib/graph/CommitDetail.svelte` | Selection callback + node lookup | Detail panel can desync from highlighted row |
| `src/lib/graph/FpsOverlay.svelte` | route-level benchmark flow | `layout`, `scrollTop`, `canvasHeight` props | Metrics may be visible but not useful for 10k-repo validation |
| `scripts/perf-bench.ts` | shared graph logic | benchmark/regression coverage | Script can drift from real layout behavior and provide false confidence |
| `src/routes/+page.svelte` | `/spike-b` validation surface | route extraction | Root route and spike route can diverge or leave duplicated spike behavior |

## Dependency Graph

### Task Dependencies

Task A — Deterministic graph pipeline baseline  
Needs: nothing  
Creates: stable layout expectations and regression hooks in `src/lib/graph/layout.ts`, `src/lib/graph/types.ts`, `scripts/perf-bench.ts`

Task B — Renderer scaling and viewport hardening  
Needs: Task A  
Creates: stable visible-range rendering and interaction-safe viewport behavior in `src/lib/graph/render.ts`, `src/lib/graph/GraphCanvas.svelte`, `src/lib/graph/hitTest.ts`, `src/lib/graph/FpsOverlay.svelte`

Task C — Dedicated `/spike-b` route integration  
Needs: Task A, Task B  
Creates: dedicated validation surface in `src/routes/spike-b/+page.svelte` and frees `src/routes/+page.svelte`

Task D — Evidence capture and final validation  
Needs: Task C  
Creates: explicit benchmark record in `.beads/artifacts/bd-12d/progress.txt` plus final verification evidence

**Wave 1:** Task A (`bd-12d.1`)  
**Wave 2:** Task B (`bd-12d.2`)  
**Wave 3:** Task C (integration slice across route + graph UI)  
**Wave 4:** Task D (`bd-12d.3` validation closeout)

---

## Tasks

### Task 1: Deterministic graph pipeline baseline

**Maps to:** PRD Task 1 / child bead `bd-12d.1`

**Files:**

- Modify: `src/lib/graph/layout.ts`
- Modify: `src/lib/graph/types.ts`
- Modify: `scripts/perf-bench.ts`

**Intent:** Make repeated layout of identical commit/ref inputs stable and observable, without introducing a new test framework.

**TDD / execution order:**

1. Add a **failing regression scenario** to `scripts/perf-bench.ts` for repeated layout on the same synthetic merge-heavy input; make the script compare lane assignments / segment topology across runs and fail loudly if they differ.
2. Add at least one deterministic fixture shape inside the benchmark script (or a colocated zero-dependency helper imported by it) that stresses:
   - first-parent continuity
   - merge parents
   - branch fan-out/fan-in
   - ref attachment ordering
3. Run the regression script first and confirm it fails before any layout changes.
4. Harden `assignLanes()` in `src/lib/graph/layout.ts` so all iteration order, lane reuse, color assignment, and ref ordering are deterministic for identical inputs.
5. If extra metadata is needed to support deterministic assertions, extend `src/lib/graph/types.ts` minimally rather than leaking spike-only state into route components.
6. Re-run the regression script until the deterministic scenario passes consistently.
7. Re-run `pnpm check` to confirm type safety after the layout/type changes.

**Implementation notes:**

- Treat `childrenOf`, `laneOf`, `activeLanes`, and ref-array ordering as the main determinism hot spots.
- Keep the algorithm pure relative to input arrays; do not mutate caller-owned commit/ref objects.
- If `scripts/perf-bench.ts` continues to mirror layout logic, make its assertions consume the shared implementation rather than another copy.

**Verification:**

- Repeated deterministic scenario passes multiple consecutive runs.
- `pnpm check` → 0 errors.
- Same input repository history yields stable lane count and segment count across repeated loads.

---

### Task 2: Renderer scaling and viewport hardening

**Maps to:** PRD Task 2 / child bead `bd-12d.2`

**Files:**

- Modify: `src/lib/graph/render.ts`
- Modify: `src/lib/graph/GraphCanvas.svelte`
- Modify: `src/lib/graph/hitTest.ts`
- Modify: `src/lib/graph/FpsOverlay.svelte`

**Intent:** Keep rendering work bounded to the visible region while preserving crispness and interaction correctness at large history sizes.

**TDD / execution order:**

1. Extend the zero-dependency regression harness so it captures at least these renderer-adjacent expectations:
   - visible-range calculations for top/middle/bottom scroll positions
   - no missing edge segments when a segment crosses the viewport boundary
   - stable row/ref hit zones for visible rows
2. Run the harness before render changes and confirm the new checks fail for the targeted cases.
3. Harden `src/lib/graph/render.ts` first:
   - keep edge culling based on crossing the visible range, not just endpoints
   - reduce repeated text measurement cost for visible rows
   - keep HiDPI drawing crisp
   - preserve merge readability at high density
4. Harden `src/lib/graph/GraphCanvas.svelte` second:
   - preserve sticky-canvas + spacer behavior
   - keep rendering queued through a single rAF path
   - prevent hover-driven redraw churn from causing visible instability
   - keep keyboard navigation semantics intact
5. Update `src/lib/graph/hitTest.ts` if any renderer spacing/ref-label math changes, so hit targets stay aligned with what is drawn.
6. Update `src/lib/graph/FpsOverlay.svelte` only as needed to surface renderer-relevant metrics, not as a separate redesign.
7. Re-run the regression harness, then `pnpm check`.

**Implementation notes:**

- The likely hotspots from the PRD/research are visible-range work, text measurement, ref-label rendering, and redraw stability during rapid scroll.
- Keep hit-testing and render geometry derived from the same spacing constants to avoid drift.
- Do not introduce offscreen canvas, WebGL, or a new rendering library in this task.

**Verification:**

- Targeted regression checks pass.
- `pnpm check` → 0 errors.
- Rapid scroll shows no blank flashes or missing cross-viewport edges.
- FPS overlay remains usable on large synthetic histories.

---

### Task 3: Dedicated `/spike-b` route integration

**Maps to:** PRD Task 3 / parent integration slice

**Files:**

- Modify: `src/routes/+page.svelte`
- Create: `src/routes/spike-b/+page.svelte`
- Modify: `src/lib/graph/GraphCanvas.svelte`
- Modify: `src/lib/graph/CommitDetail.svelte`
- Modify: `src/lib/graph/FpsOverlay.svelte`

**Intent:** Move the spike to `/spike-b` without losing graph validation capabilities or breaking existing interaction flows.

**TDD / execution order:**

1. Define the route-move acceptance checklist first in `.beads/artifacts/bd-12d/progress.txt` so the implementation has a concrete pass/fail list for:
   - synthetic load
   - real repo load
   - selection
   - hover
   - context menu
   - keyboard navigation
   - commit detail inspection
2. Create `src/routes/spike-b/+page.svelte` by moving the current spike responsibilities there; keep the route focused on validation, not future repo-home UX.
3. Reduce `src/routes/+page.svelte` so `/` is no longer the graph spike surface. Keep it intentionally minimal and compatible with upcoming bd-6ew home/workspace work.
4. Verify that selection state, detail-panel wiring, and FPS overlay still work from the new route; apply minimal fixes in `GraphCanvas.svelte`, `CommitDetail.svelte`, and `FpsOverlay.svelte` only when the route extraction exposes coupling.
5. Run `pnpm check` and `pnpm build` after the route move.

**Implementation notes:**

- Avoid creating a thin wrapper/re-export component just to move code between routes.
- Keep the spike route self-contained enough for future manual validation runs.
- Preserve keyboard accessibility and focus behavior on the new route.

**Verification:**

- `/spike-b` loads and can render both synthetic and real repository histories.
- `/` is no longer the primary graph spike surface.
- `pnpm check` → 0 errors.
- `pnpm build` → succeeds.

---

### Task 4: Performance evidence capture and closeout

**Maps to:** PRD Task 4 / child bead `bd-12d.3`

**Files:**

- Modify: `scripts/perf-bench.ts`
- Modify: `src/routes/spike-b/+page.svelte`
- Modify: `src/lib/graph/FpsOverlay.svelte`
- Modify: `.beads/artifacts/bd-12d/progress.txt`

**Intent:** Leave behind explicit evidence that the graph baseline is ready for downstream productization, rather than relying on subjective “felt smooth” claims.

**TDD / execution order:**

1. Expand `scripts/perf-bench.ts` so the automated output reports the deterministic/perf thresholds that matter to this bead (layout time, lane count, segment count, hit-test stability, and any renderer-adjacent metrics it can credibly measure without a browser harness).
2. Add or expose just enough route-level metrics on `/spike-b` to support manual benchmark runs on a real 10k+ repo:
   - commit count
   - lane count
   - layout time
   - visible rows
   - FPS / frame time
3. Run the automated benchmark script and record the output summary in `.beads/artifacts/bd-12d/progress.txt`.
4. Run the manual `/spike-b` validation flow on a real 10k+ repository and record results in `.beads/artifacts/bd-12d/progress.txt` using a checklist that includes:
   - repeated load stability
   - sustained scroll smoothness
   - no blank flashes
   - crisp HiDPI rendering
   - selection / hover / context menu / keyboard nav / detail inspection
5. Finish with full project verification.

**Implementation notes:**

- Keep evidence capture concrete and reproducible; record the benchmark repository path or identifying description if policy allows.
- If a real 10k+ repo exposes a new hotspot, fix the hotspot before closing the bead rather than lowering the bar.

**Verification:**

- `pnpm check` → 0 errors.
- `pnpm build` → succeeds.
- `cargo check` in `src-tauri/` → succeeds.
- `.beads/artifacts/bd-12d/progress.txt` contains both automated and manual evidence.

---

## Verification Sequence

Run in this order after implementation:

1. Targeted zero-dependency regression/benchmark script for determinism + layout/perf checks.
2. `pnpm check`
3. `pnpm build`
4. `(cd src-tauri && cargo check)`
5. Manual validation on `/spike-b`:
   - synthetic dataset load
   - real 10k+ repo load
   - repeated load stability
   - continuous scroll with FPS overlay visible
   - click selection
   - hover stability
   - right-click context menu targets correct commit
   - keyboard navigation: `↑`, `↓`, `Home`, `End`, `PageUp`, `PageDown`, `Enter`, `Escape`
   - commit detail inspection remains synchronized with selection

## Risks / Watchpoints

- **Benchmark drift:** `scripts/perf-bench.ts` currently mirrors layout logic; if it stays duplicated, it can hide real regressions.
- **Route extraction drift:** moving the route can accidentally fork behavior between `/` and `/spike-b`.
- **Hit-test misalignment:** any change to ref-label widths or text start positions can silently break selection/context menu targeting.
- **False confidence from synthetic-only runs:** this bead must be closed only after real 10k+ repository validation is recorded.
- **Testing pressure:** because no frontend test harness exists, resist “just add Vitest” unless the user explicitly approves the dependency change.

## Existing Child Beads

- `bd-12d.1` — Graph data pipeline and lane algorithm → covers Task 1
- `bd-12d.2` — Canvas renderer and viewport virtualization → covers Task 2
- `bd-12d.3` — Graph interaction and performance benchmark suite → covers Task 4

No additional child beads are required for this plan.

## Next Command

`/ship bd-12d`
