# PRD: Graph interaction and performance benchmark suite

**Bead:** bd-12d.3
**Parent:** bd-12d
**Type:** task
**Priority:** P1

```yaml
requirements_score:
  total: 93
  breakdown:
    business_value: 28
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

The parent Spike B already has the core graph pieces in place: `/spike-b` can load synthetic or real repository data, `GraphCanvas.svelte` supports selection/hover/context-menu/keyboard navigation, `CommitDetail.svelte` renders selected-commit details, `FpsOverlay.svelte` exposes frame-time and visible-row metrics, and `.beads/artifacts/bd-12d/progress.txt` already contains an initial automated benchmark block plus a manual checklist. What remains unresolved is not basic graph rendering, but the **proof surface** that makes the graph baseline trustworthy for future agents and future releases.

This child bead must tighten that proof surface into a comparable baseline: interaction readiness should be explicitly validated on the dedicated `/spike-b` route, automated benchmark output should stay aligned with the real shared graph implementation, and the resulting evidence should be recorded in a durable format that future agents can compare against. The emphasis for this bead is **progress-log baseline evidence**, not a broad telemetry redesign.

## Scope

### In-Scope

- Harden the graph validation flow around existing interaction hooks on `/spike-b`
- Capture explicit interaction-readiness evidence for:
  - selection
  - hover
  - context menu
  - keyboard navigation
  - commit detail inspection
- Expand or align the automated graph benchmark output in `scripts/perf-bench.ts` so it remains useful as a future comparison baseline
- Expose or preserve route-visible metrics needed for manual validation runs on `/spike-b`
- Record automated and manual evidence in `.beads/artifacts/bd-12d/progress.txt` in a durable, comparable format
- Keep validation and evidence generation aligned with the shared graph implementation rather than drift-prone copies

### Out-of-Scope

- Reworking the graph layout algorithm owned by `bd-12d.1`
- Reworking renderer virtualization/culling behavior owned by `bd-12d.2`
- Replacing Canvas 2D with WebGL, DOM rendering, or another graphics architecture
- Adding a frontend test framework or any new dependency without explicit approval
- Building advanced graph UX beyond the current validation surface (search/filter, collapse/expand, history navigation)
- Creating a full analytics/telemetry system beyond what is needed for local validation evidence

## Proposed Solution

Build on the current validation surface rather than inventing a new benchmark system:

1. Keep `/spike-b` as the canonical graph validation surface for both synthetic and real repository runs.
2. Treat the existing interaction hooks in `GraphCanvas.svelte` and `CommitDetail.svelte` as the baseline to validate and minimally instrument, not redesign.
3. Keep `FpsOverlay.svelte` and the `/spike-b` toolbar as the route-visible metrics surface for manual validation, adding only what is necessary to support comparable evidence.
4. Keep `scripts/perf-bench.ts` as the automated benchmark entry point, but ensure its output remains aligned with the shared graph implementation and useful for later comparison.
5. Treat `.beads/artifacts/bd-12d/progress.txt` as the primary evidence ledger: automated benchmark summaries and manual checklist results should live there in a format that future agents can re-run and compare.

## Requirements

### R1: Interaction Readiness Is Explicitly Verifiable

The existing graph interaction primitives must be validated as production-worthy on the dedicated `/spike-b` surface rather than assumed to work because the UI renders.

**Acceptance:**
- Click selection can be validated against the correct commit on `/spike-b`
- Hover behavior can be validated without excessive redraw instability
- Right-click context menu targeting can be validated against the correct commit
- Keyboard navigation (`↑`, `↓`, `Home`, `End`, `PageUp`, `PageDown`, `Enter`, `Escape`) remains part of the explicit validation flow
- Commit detail inspection remains synchronized with the selected commit during validation runs

**Affected files:** `src/routes/spike-b/+page.svelte`, `src/lib/graph/GraphCanvas.svelte`, `src/lib/graph/CommitDetail.svelte`

### R2: Route-Visible Metrics Support Manual Benchmark Runs

The `/spike-b` validation surface must expose enough metrics to make manual benchmark runs useful and repeatable.

**Acceptance:**
- Commit count, lane count, and layout time remain visible during validation runs
- FPS, frame time, visible rows, and scroll row remain observable through the overlay
- Any added metrics stay tightly scoped to validation usefulness, not dashboard sprawl
- Metrics remain derived from real graph state rather than duplicated estimates where shared sources already exist

**Affected files:** `src/routes/spike-b/+page.svelte`, `src/lib/graph/FpsOverlay.svelte`

### R3: Automated Benchmark Output Is Comparable Over Time

The benchmark script must produce evidence that future agents can compare against without guessing what changed between runs.

**Acceptance:**
- `scripts/perf-bench.ts` reports benchmark output in a stable, explicit format
- Automated checks continue to measure the shared graph implementation rather than a divergent copy
- 10k-commit target results remain clearly called out in script output
- Benchmark output is specific enough to compare later regressions or improvements

**Affected files:** `scripts/perf-bench.ts`, `.beads/artifacts/bd-12d/progress.txt`

### R4: Evidence Is Recorded in a Durable Progress-Log Baseline

Spike B’s evidence ledger must become the durable baseline that future agents can consult and extend.

**Acceptance:**
- `.beads/artifacts/bd-12d/progress.txt` contains both automated benchmark evidence and manual validation evidence
- Manual evidence is recorded against an explicit checklist rather than free-form memory only
- The recorded format is stable enough for later re-runs to compare results
- Evidence capture stays scoped to Spike B validation, not a generalized reporting framework

**Affected files:** `.beads/artifacts/bd-12d/progress.txt`, `src/routes/spike-b/+page.svelte`, `src/lib/graph/FpsOverlay.svelte`, `scripts/perf-bench.ts`

### R5: Validation Foundation Remains Reusable

This bead should leave behind a validation/evidence layer that downstream graph work can reuse without undoing core graph structure.

**Acceptance:**
- Validation logic remains attached to the existing graph surface and shared graph modules, not forked into spike-only duplicates
- Interaction/evidence work does not require later graph productization to delete major structural choices
- Shared geometry/rendering contracts from `bd-12d.2` remain authoritative during validation work
- Future agents can compare new benchmark runs against the recorded baseline with minimal ambiguity

**Affected files:** `src/routes/spike-b/+page.svelte`, `src/lib/graph/*.svelte`, `scripts/perf-bench.ts`, `.beads/artifacts/bd-12d/progress.txt`

## Success Criteria

1. User can open `/spike-b` and run an explicit interaction-validation flow against the current graph surface
   - Verify: manual `/spike-b` validation of selection, hover, context menu, keyboard navigation, and commit detail inspection
2. Route-visible metrics are sufficient to support manual benchmark runs on synthetic and real repository data
   - Verify: manual `/spike-b` run with toolbar metrics + FPS overlay visible
3. Automated benchmark output is fresh, explicit, and comparable for later runs
   - Verify: `npx tsx scripts/perf-bench.ts`
4. Spike B evidence is durably recorded in `.beads/artifacts/bd-12d/progress.txt`
   - Verify: inspect progress log after automated + manual validation recording
5. Full project verification still passes after evidence/validation hardening
   - Verify: `pnpm check`, `pnpm build`, `cargo check`

## Verify

```bash
npx tsx scripts/perf-bench.ts
pnpm check
pnpm build
cargo check
```

Manual verification:
- Open `/spike-b`
- Load synthetic data and confirm graph still renders
- Load a real repository with 10k+ commits
- Toggle the FPS overlay and observe route-visible metrics during scrolling
- Validate click, hover, right-click, keyboard navigation, and commit detail behavior
- Record automated and manual results in `.beads/artifacts/bd-12d/progress.txt`

## Technical Context

### Existing patterns

- `src/routes/spike-b/+page.svelte:44-104` — existing real-repo and synthetic data loading flow for the validation surface
- `src/routes/spike-b/+page.svelte:106-136` — parent-level selection and scroll/height wiring already exists
- `src/routes/spike-b/+page.svelte:174-205` — toolbar stats + FPS overlay toggle already expose a basic benchmark surface
- `src/lib/graph/GraphCanvas.svelte:120-176` — selection and context-action callbacks already flow out of the canvas component
- `src/lib/graph/GraphCanvas.svelte:178-257` — keyboard navigation behavior already exists and should be validated rather than redesigned
- `src/lib/graph/CommitDetail.svelte` — selected-commit inspection surface is already wired to the chosen node
- `src/lib/graph/FpsOverlay.svelte:29-35` — current derived metrics include visible rows, scroll row, total commits, lane count, and layout time
- `scripts/perf-bench.ts` — existing automated benchmark surface already measures layout, hit-test cost, and memory targets
- `.beads/artifacts/bd-12d/progress.txt` — existing benchmark log + manual checklist already provide the seed format for the durable baseline

### Constraints

- No frontend test harness exists in `package.json`; validation must rely on zero-dependency scripts, existing project checks, and explicit manual runs
- Real-repo loading on `/spike-b` currently uses `max_count: 10000`, so the primary real-history validation target is 10k commits rather than an arbitrary unbounded dataset
- This bead follows `bd-12d.2`; it should consume the hardened shared renderer/hit-test contracts rather than redefining them
- Evidence emphasis for this bead is **progress-log baseline** first, not a full telemetry product
- Route-visible metrics should stay useful and minimal rather than turning `/spike-b` into a diagnostics dashboard

### Research findings

- The core interaction hooks already exist; the remaining gap is proof, measurability, and durable evidence rather than missing baseline behavior
- `FpsOverlay.svelte` already exposes enough core metrics that this bead should likely extend or preserve the current pattern rather than invent a new overlay architecture
- `.beads/artifacts/bd-12d/progress.txt` already contains benchmark output and a manual checklist, making it the natural baseline artifact for future comparison
- `scripts/perf-bench.ts` is the right automation entry point for this bead, but its output must stay specific and aligned with the shared implementation to avoid false confidence
- The main remaining risk is that future agents could lose comparability if evidence remains ad hoc, incomplete, or split across too many surfaces

## Affected Files

| File | Action | Description |
|------|--------|-------------|
| `src/routes/spike-b/+page.svelte` | Modify | Tighten the validation flow and expose benchmark/use-state signals needed for manual interaction checks |
| `src/lib/graph/FpsOverlay.svelte` | Modify | Preserve or extend route-visible metrics for manual benchmark runs without turning the overlay into a redesign |
| `scripts/perf-bench.ts` | Modify | Produce explicit, comparable automated benchmark evidence aligned with the shared graph implementation |
| `.beads/artifacts/bd-12d/progress.txt` | Modify | Record the durable automated + manual evidence baseline for Spike B |
| `src/lib/graph/GraphCanvas.svelte` | Maybe modify | Support interaction-readiness validation only if route-level evidence needs additional hooks |
| `src/lib/graph/CommitDetail.svelte` | Maybe modify | Preserve or expose selected-commit readiness only if validation flow reveals a gap |

## Tasks

### Task 1: Interaction validation surface [validation]

The `/spike-b` surface makes interaction readiness and route-visible benchmark metrics explicit enough for manual validation runs.

**Verification:**
- Manual `/spike-b` validation confirms selection, hover, right-click, keyboard navigation, and commit detail readiness
- FPS overlay and toolbar metrics remain usable during synthetic and real-repo validation runs
- `pnpm check`

**Metadata:**
```yaml
depends_on: []
parallel: true
files:
  - src/routes/spike-b/+page.svelte
  - src/lib/graph/FpsOverlay.svelte
  - src/lib/graph/GraphCanvas.svelte
  - src/lib/graph/CommitDetail.svelte
```

### Task 2: Comparable benchmark evidence baseline [benchmark]

Automated benchmark output and the Spike B progress log form a durable baseline future agents can compare against.

**Verification:**
- `npx tsx scripts/perf-bench.ts`
- `.beads/artifacts/bd-12d/progress.txt` records automated benchmark output and manual validation results
- `pnpm check`
- `pnpm build`
- `cargo check`

**Metadata:**
```yaml
depends_on: ["Interaction validation surface"]
parallel: false
files:
  - scripts/perf-bench.ts
  - .beads/artifacts/bd-12d/progress.txt
  - src/routes/spike-b/+page.svelte
  - src/lib/graph/FpsOverlay.svelte
```

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Evidence remains ad hoc and future agents cannot compare runs reliably | High | Make `.beads/artifacts/bd-12d/progress.txt` the explicit baseline artifact with stable sections |
| Manual validation proves interaction works, but the proof is not durable | High | Record the results against a checklist rather than relying on conversational memory |
| Benchmark output drifts from the shared graph implementation | High | Keep `scripts/perf-bench.ts` aligned with shared imports and record fresh output in the progress log |
| Route-visible metrics grow into a redesign instead of a validation aid | Medium | Keep additions tightly scoped to benchmark usefulness |
| Scope bleeds back into renderer/layout ownership already covered by `bd-12d.1` and `bd-12d.2` | Medium | Treat this bead as validation/evidence closeout, not a new graph-engine redesign |

## Open Questions

None — the remaining ambiguity was resolved during refinement: this bead should emphasize a **progress-log baseline** future agents can compare against, while keeping route-visible metrics and interaction validation focused on that goal.

---

## Metadata

**Parent:** bd-12d
