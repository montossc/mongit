# PRD: Spike B — Canvas Commit Graph Engine

**Bead:** bd-12d
**Parent:** bd-134
**Type:** feature
**Priority:** P0

```yaml
requirements_score:
  total: 93
  breakdown:
    business_value: 27
    functional_requirements: 24
    user_experience: 18
    technical_constraints: 14
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

mongit's commit graph is a core differentiator and one of the highest remaining technical risks. The codebase already contains a substantial graph foundation — IPC commands for commit log/refs, a TypeScript lane assignment engine, a Canvas renderer, hit testing, a detail panel, and an FPS overlay — but that work still behaves like a spike living on the root route rather than a validated production baseline.

This task must convert the existing graph foundation into a trustworthy technical baseline: deterministic layout on real repository history, responsive Canvas rendering at target scale, preserved interaction primitives, and measurable performance evidence from a **real 10k+ commit repository**. It must also move the graph validation surface to a dedicated `/spike-b` route so the app root can evolve into the repo home/workspace shell without losing graph validation tooling.

## Scope

### In-Scope

- Validate and harden the existing graph data pipeline in `src/lib/graph/`
- Prove deterministic lane assignment on real and synthetic histories
- Optimize/confirm Canvas rendering behavior for large histories
- Preserve and validate interaction primitives already present:
  - selection
  - hover
  - context menu
  - keyboard navigation
  - commit detail inspection
- Capture measurable performance evidence for a **real 10k+ commit repo**
- Move the graph spike surface from `/` to **`/spike-b`**
- Keep the graph foundation reusable for downstream graph productization work

### Out-of-Scope

- Advanced graph search/filter UX
- Branch collapse/expand UX
- IntelliSort or history-analysis features beyond current topology baseline
- Inline diff/history cross-navigation
- Production repo-home/workspace shell UX
- Replacing Canvas 2D with another rendering technology
- New frontend test framework or dependency additions without explicit approval

## Proposed Solution

Build on the existing graph implementation rather than redesigning it:

1. Keep the current Rust commit-log/ref IPC path as the source of real graph data.
2. Harden `assignLanes()` and related graph data flow so repeated runs on the same history produce stable output suitable for downstream features.
3. Focus renderer work on the real scaling gaps surfaced by research: viewport-aware segment work, text-measurement cost, ref-label behavior, and high-density merge readability.
4. Preserve the existing interaction model in `GraphCanvas.svelte`, `CommitDetail.svelte`, and `hitTest.ts`, validating that it remains responsive at large history sizes.
5. Move the graph validation UI to `/spike-b`, preserving it as an internal spike surface while freeing `/` for bd-6ew and later workspace flows.
6. Use the existing FPS overlay plus explicit manual benchmark flows to produce evidence, not assumptions, for graph readiness.

## Requirements

### R1: Dedicated Spike B Validation Surface

The graph validation surface lives on a dedicated `/spike-b` route instead of the root route.

**Acceptance:**
- Graph spike UI is accessible at `/spike-b`
- Root route is no longer the long-term home of the graph spike surface
- Route move preserves the current ability to load real repo data and synthetic data
- Graph validation tooling remains available for future work after bd-6ew advances the root/home surface

**Affected files:** `src/routes/+page.svelte`, `src/routes/spike-b/+page.svelte`

### R2: Deterministic Graph Data Pipeline

Graph layout for the same commit/ref input is deterministic and stable enough for downstream graph productization.

**Acceptance:**
- Repeated runs on the same commit/ref input produce the same lane assignments and segment topology
- Real repository histories with branches and merges render without invalid graph structure
- Synthetic histories still work for stress validation
- Layout remains bounded and practical at 10k commits

**Affected files:** `src/lib/graph/layout.ts`, `src/lib/graph/types.ts`

### R3: Scalable Canvas Rendering and Viewport Handling

Canvas rendering remains responsive on large histories by limiting work to what is necessary for the visible region.

**Acceptance:**
- Visible-range rendering avoids unnecessary off-screen work
- No blank flashes or missing edges during rapid scroll
- Retina/HiDPI rendering remains crisp
- Merge and same-lane edges remain visually understandable at target scale

**Affected files:** `src/lib/graph/render.ts`, `src/lib/graph/GraphCanvas.svelte`

### R4: Interaction Primitives Remain Trustworthy

The graph surface provides responsive interaction primitives suitable for later navigation features.

**Acceptance:**
- Click selection works reliably on large histories
- Hover behavior remains stable and does not cause excessive redraw churn
- Context menu still resolves against the correct commit
- Keyboard navigation (`↑`, `↓`, `Home`, `End`, `PageUp`, `PageDown`, `Enter`) remains usable
- Commit detail inspection remains wired to the selected commit

**Affected files:** `src/lib/graph/GraphCanvas.svelte`, `src/lib/graph/hitTest.ts`, `src/lib/graph/CommitDetail.svelte`

### R5: Real 10k+ Repo Performance Evidence

Performance proof for Spike B is based on a **real 10k+ commit repository**, not synthetic-only evidence.

**Acceptance:**
- The graph can load and render a real 10k+ commit repository
- Sustained scrolling remains practically smooth on the benchmark surface
- Layout time, visible rows, lane count, and FPS are observable through the validation surface
- Performance evidence is captured explicitly during verification rather than assumed from code structure

**Affected files:** `src/routes/spike-b/+page.svelte`, `src/lib/graph/FpsOverlay.svelte`, `src/lib/graph/layout.ts`, `src/lib/graph/render.ts`

### R6: Reusable Foundation for Commit Graph Productization

The spike leaves behind a graph engine and validation surface that downstream graph productization can adopt without rework.

**Acceptance:**
- Core graph logic remains separated by concern: types, layout, render, hit testing, canvas component
- No spike-only shortcuts block later commit-graph productization work in bd-145
- Route extraction does not force future graph work to undo major structural choices

**Affected files:** `src/lib/graph/types.ts`, `src/lib/graph/layout.ts`, `src/lib/graph/render.ts`, `src/lib/graph/GraphCanvas.svelte`

### R7: Design-System and UX Baseline Compliance

The graph spike continues to use established tokens and UX patterns rather than ad hoc styling.

**Acceptance:**
- Existing design tokens remain the source of colors, spacing, and typography
- Focusable/interactive elements preserve keyboard accessibility expectations
- Performance overlays and graph controls do not regress the existing visual baseline

**Affected files:** `src/routes/spike-b/+page.svelte`, `src/lib/graph/*.svelte`, `src/app.css` (only if minimal additions are required)

## Success Criteria

1. User can open **`/spike-b`** and validate the graph on both synthetic and real repository data
2. The same real repository history produces stable graph layout across repeated runs
3. A **real 10k+ commit repository** scrolls with practical smoothness and visible FPS evidence
4. Graph interactions remain usable: selection, hover, context menu, keyboard navigation, commit detail inspection
5. Root route is no longer burdened with the graph spike surface, reducing conflict with bd-6ew
6. Verification passes:
   - `pnpm check` — 0 errors
   - `pnpm build` — succeeds
   - `cargo check` — succeeds

## Verify

```bash
pnpm check    # 0 errors (svelte-check)
pnpm build    # Vite build succeeds
cargo check   # Rust typecheck succeeds (in src-tauri/)
```

Manual verification:
- Open `/spike-b`
- Load synthetic data and confirm graph still renders
- Load a real repository with 10k+ commits
- Scroll continuously and observe FPS/metrics overlay
- Repeat load for the same repo and confirm layout remains stable
- Validate click, hover, right-click, keyboard navigation, and commit detail behavior

## Technical Context

### Relevant existing code

- `src/lib/graph/types.ts:1-70` — graph data contracts used by the renderer
- `src/lib/graph/layout.ts:29-191` — current greedy lane-assignment implementation
- `src/lib/graph/layout.ts:197-252` — synthetic history generator for stress validation
- `src/lib/graph/render.ts:63-96` — main render pipeline
- `src/lib/graph/render.ts:209-268` — grouped edge rendering with visible-range checks
- `src/lib/graph/GraphCanvas.svelte:43-82` — DPR scaling, render queue, sticky-canvas pattern
- `src/lib/graph/GraphCanvas.svelte:174-253` — keyboard navigation behavior
- `src/lib/graph/hitTest.ts:32-75` — row/node/ref hit testing
- `src/lib/graph/CommitDetail.svelte:117-188` — selected commit inspection surface
- `src/lib/graph/FpsOverlay.svelte:14-132` — ring-buffer FPS measurement loop
- `src/routes/+page.svelte:44-136` — current graph spike container on the root route
- `src-tauri/src/commands.rs:48-67` — `get_commit_log` and `get_refs` IPC commands already exist

### Constraints

- Canvas 2D is already the chosen rendering path; this spike validates that decision rather than revisiting it
- `git2::Repository` is not `Send + Sync`; existing open-per-call pattern must remain
- No frontend test harness exists in `package.json`; verification should rely on existing checks plus explicit manual benchmark flows unless a later dependency decision is approved
- Root route is under pressure from bd-6ew; keeping the graph spike there increases integration conflict
- Existing graph work already landed in `src/lib/graph/*`; this PRD should harden and validate that implementation, not plan a greenfield rebuild

### Research findings

- Existing graph architecture is already substantial and reusable
- The biggest remaining risks are proof and scaling, not initial architecture choice
- Recent fixes in bd-15p already addressed some graph correctness/performance issues, including edge culling and keyboard-nav hardening
- The most plausible remaining hotspots are segment work at scale, text measurement cost, merge-edge readability, and benchmark evidence gaps

## Affected Files

| File | Action | Description |
|------|--------|-------------|
| `src/routes/+page.svelte` | Modify | Remove graph spike from root route or reduce it to a non-spike shell |
| `src/routes/spike-b/+page.svelte` | Create | Dedicated graph validation surface |
| `src/lib/graph/layout.ts` | Modify | Determinism validation and lane-algorithm hardening |
| `src/lib/graph/types.ts` | Maybe modify | Keep graph contracts aligned with hardening work |
| `src/lib/graph/render.ts` | Modify | Rendering/culling/perf refinements |
| `src/lib/graph/GraphCanvas.svelte` | Modify | Preserve scroll/render/interaction behavior at scale |
| `src/lib/graph/hitTest.ts` | Maybe modify | Maintain accurate hit testing after renderer adjustments |
| `src/lib/graph/CommitDetail.svelte` | Maybe modify | Preserve selected-commit inspection flow |
| `src/lib/graph/FpsOverlay.svelte` | Modify | Support benchmark/validation visibility if needed |
| `src/app.css` | Maybe modify | Minimal graph-spike styling additions only if tokens are insufficient |

## Tasks

### Task 1: Graph data pipeline and determinism [foundation]

Validate and harden the existing graph data pipeline so layout remains deterministic on repeated runs with real and synthetic histories.

**Verification:**
- `pnpm check` passes
- Same input history yields stable lane assignment across repeated runs
- Real merge-heavy history renders without broken topology

**Metadata:**
```yaml
depends_on: []
parallel: true
files:
  - src/lib/graph/layout.ts
  - src/lib/graph/types.ts
```

### Task 2: Renderer scaling and viewport hardening [renderer]

Harden Canvas rendering and viewport behavior for large histories, focusing on visible-range work, redraw stability, and high-density graph readability.

**Verification:**
- `pnpm check` passes
- Rapid scroll shows no blank flashes or missing edges
- HiDPI rendering remains crisp
- FPS and frame-time behavior stay practical on large histories

**Metadata:**
```yaml
depends_on: [foundation]
parallel: false
files:
  - src/lib/graph/render.ts
  - src/lib/graph/GraphCanvas.svelte
  - src/lib/graph/hitTest.ts
  - src/lib/graph/FpsOverlay.svelte
```

### Task 3: Dedicated spike route and graph validation flow [integration]

Move the graph spike to `/spike-b`, preserve interaction behavior, and make the route the canonical validation surface for real-repo and synthetic benchmarks.

**Verification:**
- `/spike-b` loads graph validation UI successfully
- Root route no longer serves as the primary graph spike surface
- Selection, hover, context menu, keyboard navigation, and commit detail behavior still work on `/spike-b`
- Real 10k+ repo benchmark can be performed from the dedicated spike surface

**Metadata:**
```yaml
depends_on: [foundation, renderer]
parallel: false
files:
  - src/routes/+page.svelte
  - src/routes/spike-b/+page.svelte
  - src/lib/graph/GraphCanvas.svelte
  - src/lib/graph/CommitDetail.svelte
  - src/lib/graph/FpsOverlay.svelte
```

### Task 4: Performance evidence capture [validation]

Use the dedicated spike surface to capture explicit evidence that the Canvas graph baseline is production-worthy for the target scale.

**Verification:**
- Real 10k+ repo loads successfully
- Sustained scroll remains practically smooth with observable FPS data
- Layout time, lane count, visible rows, and rendering behavior are recorded during manual validation

**Metadata:**
```yaml
depends_on: [integration]
parallel: false
files:
  - src/routes/spike-b/+page.svelte
  - src/lib/graph/FpsOverlay.svelte
```

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Remaining performance issues only appear on real large repos | High | Require real 10k+ repo validation rather than synthetic-only evidence |
| Renderer still does too much work per frame at scale | High | Focus hardening on visible-range work, measurement cost, and redraw behavior |
| Root-route graph spike conflicts with bd-6ew home/workspace work | High | Move graph validation to `/spike-b` |
| No formal frontend test harness limits automated confidence | Medium | Use deterministic checks plus explicit manual benchmark verification |
| Merge-heavy histories expose lane/edge readability issues | Medium | Validate against real merge-heavy repos and synthetic stress cases |

## Open Questions

None — all questions resolved during refinement.

---

## Metadata

**Parent:** bd-134
**Children:** bd-12d.1 (graph data pipeline and lane algorithm), bd-12d.2 (canvas renderer and viewport virtualization), bd-12d.3 (graph interaction and performance benchmark suite)
