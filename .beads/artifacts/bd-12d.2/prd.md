# PRD: Canvas renderer and viewport virtualization

**Bead:** bd-12d.2
**Parent:** bd-12d
**Type:** task
**Priority:** P0

```yaml
requirements_score:
  total: 92
  breakdown:
    business_value: 27
    functional_requirements: 23
    user_experience: 17
    technical_constraints: 15
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

The parent Spike B already has a working Canvas graph stack — `renderGraph()` in `src/lib/graph/render.ts`, the sticky-canvas container in `src/lib/graph/GraphCanvas.svelte`, and the `/spike-b` validation surface — but renderer scalability remains the highest unresolved risk inside the graph engine. The current implementation already performs visible-range row culling and segment filtering, yet this child bead exists because the renderer/viewport contract still needs to be specified as its own slice rather than left implicit inside the parent spike.

This task must ensure the Canvas layer keeps per-frame work bounded to the visible region, preserves crisp HiDPI output, and stays aligned with hit testing and overlay metrics while prioritizing **smooth scrolling on large histories**. It should strengthen the renderer and viewport behavior without pulling route integration, layout determinism, or final benchmark evidence capture back into scope.

## Scope

### In-Scope

- Harden the Canvas rendering path in `src/lib/graph/render.ts`
- Define viewport/virtualization expectations for visible rows and cross-viewport edges
- Preserve the single-rAF render queue and sticky-canvas scroll model in `src/lib/graph/GraphCanvas.svelte`
- Keep render geometry and interaction geometry aligned across `render.ts` and `hitTest.ts`
- Use the existing zero-dependency regression harness to validate renderer-adjacent invariants
- Surface renderer-relevant metrics through the existing FPS overlay only where needed to support smooth-scroll validation
- Keep the graph renderer reusable for downstream graph productization work

### Out-of-Scope

- Route extraction or toolbar redesign in `src/routes/spike-b/+page.svelte`
- Graph layout determinism changes owned by `bd-12d.1`
- Full real-repository benchmark evidence capture owned by parent closeout / `bd-12d.3`
- New rendering technology (WebGL, DOM graph rendering, offscreen canvas architecture redesign)
- Adding a frontend test framework or any new dependency without explicit approval
- Reworking commit-detail or broader graph UX outside renderer/alignment needs

## Proposed Solution

Build on the existing renderer rather than redesigning it:

1. Keep `renderGraph()` in `src/lib/graph/render.ts` as the canonical drawing entry point for `LayoutResult` data.
2. Treat `getVisibleRange()` and `segmentIntersectsVisibleRows()` as the viewport contract: off-screen rows should be skipped, but long segments crossing the viewport must still draw.
3. Prioritize **smooth scrolling** by keeping rendering bounded to visible work and by preserving the single queued `requestAnimationFrame` path in `GraphCanvas.svelte`.
4. Keep draw geometry, hit-test geometry, and overlay metrics derived from shared renderer constants so renderer changes do not desynchronize interaction behavior.
5. Use the existing zero-dependency script harness (`scripts/determinism-check.ts`) as the primary automated proof for viewport math, edge culling, and hit-test alignment; leave final large-repo evidence capture to the parent bead / `bd-12d.3`.

## Requirements

### R1: Visible-Range Rendering Contract

Canvas drawing limits work to the visible region while still rendering graph structures that cross the viewport boundary.

**Acceptance:**
- Visible-range calculations remain stable for top, middle, and bottom scroll positions
- Long segments crossing the viewport are not dropped just because both endpoints are off-screen
- Off-screen rows are not rendered unnecessarily
- Rapid scroll does not introduce blank flashes caused by viewport culling errors

**Affected files:** `src/lib/graph/render.ts`, `scripts/determinism-check.ts`

### R2: Smooth-Scroll Canvas Runtime

The graph canvas prioritizes smooth scrolling on large histories by minimizing redraw churn and preserving a single authoritative render queue.

**Acceptance:**
- Scroll updates continue to flow through one queued `requestAnimationFrame` path
- Hover changes do not trigger unstable redraw storms
- Sticky-canvas + spacer behavior remains intact during large-history scrolling
- Synthetic large-history validation on `/spike-b` remains practically smooth

**Affected files:** `src/lib/graph/GraphCanvas.svelte`, `src/lib/graph/render.ts`, `src/lib/graph/FpsOverlay.svelte`

### R3: Render / Hit-Test Geometry Alignment

Interaction targeting stays aligned with what the renderer draws after any viewport or spacing refinements.

**Acceptance:**
- Node, row, and ref hit zones still match rendered geometry
- Ref-label / text-area spacing does not drift from hit-test assumptions
- Selection and context targeting remain correct for visible rows after renderer changes
- Keyboard navigation and selected-row highlighting continue to reflect the visible canvas state

**Affected files:** `src/lib/graph/render.ts`, `src/lib/graph/GraphCanvas.svelte`, `src/lib/graph/hitTest.ts`

### R4: Reusable Renderer Foundation

Renderer hardening remains reusable for downstream graph work instead of becoming `/spike-b`-specific logic.

**Acceptance:**
- Graph modules continue consuming `LayoutResult` and shared constants rather than route-local assumptions
- Overlay additions remain renderer-focused, not a route-level redesign
- No spike-only state leaks from `/spike-b` into shared graph modules
- `bd-12d.3` can build on this bead for performance evidence capture without undoing renderer choices

**Affected files:** `src/lib/graph/render.ts`, `src/lib/graph/GraphCanvas.svelte`, `src/lib/graph/FpsOverlay.svelte`, `src/lib/graph/hitTest.ts`

## Success Criteria

1. Renderer-adjacent regression coverage proves viewport math and edge-culling behavior remain correct
   - Verify: `npx tsx scripts/determinism-check.ts`
2. Renderer/canvas changes keep the frontend type-safe
   - Verify: `pnpm check`
3. The `/spike-b` graph surface scrolls a large synthetic history without blank flashes or obvious redraw instability
   - Verify: manual validation on `/spike-b` with synthetic 10k+ commits and FPS overlay visible
4. Hit testing, selection, and context targeting remain aligned with rendered output after renderer hardening
   - Verify: manual validation on `/spike-b` for click, hover, and right-click targeting
5. This child bead leaves the graph renderer more reusable and better bounded without absorbing parent-bead benchmarking scope
   - Verify: PRD scope + affected files remain renderer-focused

## Verify

```bash
npx tsx scripts/determinism-check.ts
pnpm check
```

Manual verification:
- Open `/spike-b`
- Generate a synthetic history of at least 10,000 commits
- Scroll continuously with the FPS overlay visible
- Confirm no blank flashes or missing cross-viewport edges
- Validate click, hover, and right-click targeting against visible commits

## Technical Context

### Existing patterns

- `src/lib/graph/render.ts:80-125` — `renderGraph()` is the canonical render entry point and already derives visible rows before drawing
- `src/lib/graph/render.ts:190-206` — `getVisibleRange(totalRows, scrollTop, canvasHeight)` provides the current viewport window with buffer rows
- `src/lib/graph/render.ts:212-219` — `segmentIntersectsVisibleRows()` already models the critical “crosses viewport even if endpoints are outside” rule
- `src/lib/graph/GraphCanvas.svelte:54-61` — `queueRender()` enforces a single queued `requestAnimationFrame` path
- `src/lib/graph/GraphCanvas.svelte:43-52` — DPR scaling and sticky canvas sizing are already handled in the component layer
- `src/lib/graph/GraphCanvas.svelte:109-167` — mouse hit resolution and selection/context-menu routing already depend on `hitTest()` alignment
- `src/lib/graph/GraphCanvas.svelte:178-257` — keyboard navigation is already wired and must remain intact while renderer work changes
- `src/lib/graph/FpsOverlay.svelte:29-35` — visible rows, scroll row, lane count, and layout time are already exposed as derived metrics
- `src/routes/spike-b/+page.svelte:195-208` — the spike route already composes `GraphCanvas`, `FpsOverlay`, and `CommitDetail` as the validation surface
- `scripts/determinism-check.ts` already covers visible-range, edge-culling, and hit-test regressions without requiring a new test dependency

### Constraints

- Canvas 2D is already the chosen rendering path; this bead validates and hardens it rather than revisiting the decision
- No frontend test harness exists in `package.json`; validation should stay script-first plus manual `/spike-b` checks
- The deterministic layout/data contract is owned by `bd-12d.1`; this child bead must consume it cleanly rather than redefining layout semantics
- Shared spacing constants in `render.ts` are the source of truth for render and hit-test geometry
- Smooth scrolling is the top priority for this PRD slice, but not at the cost of missing edges or broken interaction targeting

### Research findings

- The renderer already has the right architectural seams; the remaining risk is scale-proofing, not inventing a new render stack
- Existing viewport logic in `render.ts` is the most important contract surface for this bead because it determines both visible work and cross-viewport correctness
- `GraphCanvas.svelte` already protects against duplicate RAF scheduling, so renderer work should preserve that invariant instead of introducing alternate draw triggers
- The likely hotspots are visible-range work, segment filtering, redraw churn from hover/scroll, and geometry drift between render and hit-test logic
- Recent graph fixes in `bd-15p` and parent `bd-12d` show that edge culling and keyboard/interaction regressions tend to surface at renderer boundaries

## Affected Files

| File | Action | Description |
|------|--------|-------------|
| `src/lib/graph/render.ts` | Modify | Harden viewport-aware drawing, segment filtering, and renderer performance behavior |
| `src/lib/graph/GraphCanvas.svelte` | Modify | Preserve single-rAF render queue, sticky-canvas behavior, and scroll/hover stability |
| `src/lib/graph/hitTest.ts` | Maybe modify | Keep node/ref/row targeting aligned with renderer geometry if spacing changes |
| `src/lib/graph/FpsOverlay.svelte` | Maybe modify | Surface renderer-relevant metrics needed for smooth-scroll validation without redesign |
| `scripts/determinism-check.ts` | Modify | Keep viewport / edge-culling / hit-test regression checks aligned with shared renderer behavior |

## Tasks

### Task 1: Visible-range renderer hardening [renderer]

The Canvas renderer draws only the necessary visible work while preserving cross-viewport edge correctness.

**Verification:**
- `npx tsx scripts/determinism-check.ts`
- Manual `/spike-b` synthetic 10k+ scroll shows no blank flashes or missing crossing edges
- `pnpm check`

**Metadata:**
```yaml
depends_on: []
parallel: true
files:
  - src/lib/graph/render.ts
  - scripts/determinism-check.ts
```

### Task 2: Canvas loop and interaction alignment [integration]

The graph canvas, hit testing, and FPS overlay stay aligned with renderer geometry while preserving smooth-scroll behavior.

**Verification:**
- `pnpm check`
- Manual `/spike-b` validation confirms click/hover/right-click targeting still matches rendered commits
- FPS overlay reports visible rows and frame-time metrics during synthetic large-history scrolling

**Metadata:**
```yaml
depends_on: ["Visible-range renderer hardening"]
parallel: false
files:
  - src/lib/graph/GraphCanvas.svelte
  - src/lib/graph/hitTest.ts
  - src/lib/graph/FpsOverlay.svelte
  - src/lib/graph/render.ts
```

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Renderer still does too much work per frame on large histories | High | Keep the PRD focused on visible-range work and redraw stability rather than adding new rendering tech |
| Edge culling becomes too aggressive and drops crossing segments | High | Treat cross-viewport edge visibility as an explicit regression contract in the script harness |
| Render geometry and hit-test geometry drift apart | High | Keep shared spacing constants authoritative and verify click/hover/context targeting manually on `/spike-b` |
| Smooth-scroll optimizations regress HiDPI crispness or keyboard usability | Medium | Preserve DPR scaling and existing keyboard semantics as explicit acceptance criteria |
| Scope bleeds into route integration or final benchmark evidence capture | Medium | Keep `/spike-b` route redesign and real-repo evidence ownership out of this child bead |

## Open Questions

None — scope and priority were resolved during refinement: this child bead is **renderer-focused** and should emphasize **smooth scrolling** first while preserving correctness and interaction alignment.

---

## Metadata

**Parent:** bd-12d
**Blocks:** bd-12d.3
