# PRD: Graph data pipeline and lane algorithm

**Bead:** bd-12d.1
**Parent:** bd-12d
**Type:** task
**Priority:** P0

```yaml
requirements_score:
  total: 91
  breakdown:
    business_value: 27
    functional_requirements: 23
    user_experience: 17
    technical_constraints: 15
    scope_and_priorities: 9
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

The parent Spike B bead needs a graph foundation that downstream renderer, hit-testing, and route integration work can trust. Today, the commit graph already has a lane-assignment engine in `src/lib/graph/layout.ts`, shared contracts in `src/lib/graph/types.ts`, and script-based regression coverage, but this child bead exists because the deterministic data pipeline must be specified as its own slice rather than left implicit inside the larger spike.

This task must ensure that identical commit/ref input always produces the same graph nodes, lanes, colors, and segments, while keeping the pipeline reusable for later commit-graph productization work. It must also define the validation boundary clearly: this child bead owns **script-first determinism proof** and shared pipeline hardening, while route-level manual benchmarking and real 10k+ repo validation remain with the parent bead and later waves.

## Scope

### In-Scope

- Harden the graph data pipeline in `src/lib/graph/layout.ts`
- Keep `src/lib/graph/types.ts` aligned with deterministic pipeline needs
- Define deterministic expectations for:
  - lane assignment
  - segment topology
  - lane/color stability for identical input
  - ref attachment ordering as consumed by the layout result
- Use existing zero-dependency script-based regression coverage to prove determinism
- Consolidate benchmark/regression logic around the shared pipeline implementation so script output reflects real layout behavior
- Keep the pipeline pure and reusable for downstream renderer and interaction work

### Out-of-Scope

- Route extraction to `/spike-b`
- Canvas rendering, viewport culling, or FPS overlay work
- Hit-testing and interaction behavior changes except where pipeline contracts force alignment
- Manual validation on a real 10k+ repository
- Adding a frontend test framework or new dependency
- Replacing the greedy first-parent lane strategy with a new graph architecture

## Proposed Solution

Build on the existing graph pipeline rather than redesigning it:

1. Keep `assignLanes()` as the canonical pipeline entry point from raw `CommitData[]` and `RefData[]` to `LayoutResult`.
2. Make determinism explicit by treating lane reuse, color assignment, and ref ordering as contract-level behavior for identical input.
3. Keep types minimal and shared so renderer and hit-test consumers rely on stable contracts rather than spike-only state.
4. Use the existing zero-dependency script harnesses (`scripts/determinism-check.ts`, `scripts/perf-bench.ts`) as the primary regression proof for this child bead.
5. Ensure any benchmark or regression script consumes the shared layout implementation instead of a copied version, so future measurements cannot drift from production behavior.

## Requirements

### R1: Deterministic Layout Contract

The same commit/ref input must always produce the same `LayoutResult` shape.

**Acceptance:**
- Repeated runs on the same input produce the same lane assignments
- Repeated runs on the same input produce the same segment topology
- Lane count and segment count remain stable across repeated runs
- Color assignment remains stable for identical input ordering

**Affected files:** `src/lib/graph/layout.ts`, `src/lib/graph/types.ts`, `scripts/determinism-check.ts`

### R2: Script-First Regression Coverage

This child bead proves correctness primarily through zero-dependency scripts rather than a new test framework or browser harness.

**Acceptance:**
- Deterministic fixture coverage exists for linear history, branch fan-out, merge fan-in, and multi-parent merge shapes
- Regression output fails loudly when determinism breaks
- Script output is repeatable enough to use as a pre-flight check for downstream graph work
- Validation responsibility for this bead stays script-first; manual real-repo validation is explicitly deferred to parent spike verification

**Affected files:** `scripts/determinism-check.ts`, `scripts/perf-bench.ts`

### R3: Shared Pipeline, No Drift

Benchmark and regression tooling must reflect the actual implementation used by the app.

**Acceptance:**
- `scripts/perf-bench.ts` does not maintain a divergent copy of the lane-assignment algorithm
- Shared types and layout logic are imported from `src/lib/graph/*` where practical
- Script-based evidence cannot silently benchmark a different algorithm than the one used by the renderer

**Affected files:** `scripts/perf-bench.ts`, `src/lib/graph/layout.ts`, `src/lib/graph/types.ts`

### R4: Reusable Data Foundation

The pipeline work remains usable by renderer, hit-testing, and downstream productization without rework.

**Acceptance:**
- `LayoutResult` remains the stable handoff contract to renderer consumers
- No route-specific or spike-only state leaks into pipeline types
- The pipeline stays pure relative to caller-owned commit/ref inputs
- Downstream beads can depend on this child bead for deterministic graph semantics

**Affected files:** `src/lib/graph/types.ts`, `src/lib/graph/layout.ts`

## Success Criteria

1. Determinism checks pass repeatedly for a fixed topology fixture
   - Verify: `npx tsx scripts/determinism-check.ts`
2. Shared benchmark/regression scripts consume the real layout implementation instead of copied logic
   - Verify: inspect `scripts/perf-bench.ts` imports and run `npx tsx scripts/perf-bench.ts`
3. Pipeline/type changes keep the frontend type-safe
   - Verify: `pnpm check`
4. This child bead leaves later graph beads with a stable pipeline contract and no unresolved scope ambiguity about manual real-repo validation
   - Verify: PRD scope + acceptance criteria reflect script-first ownership for this bead

## Verify

```bash
npx tsx scripts/determinism-check.ts
npx tsx scripts/perf-bench.ts
pnpm check
```

## Technical Context

### Existing patterns

- `src/lib/graph/layout.ts:29-190` — pure `assignLanes(commits, refs, config?)` pipeline that returns `LayoutResult`
- `src/lib/graph/layout.ts:197-252` — synthetic history generation already colocated with graph pipeline utilities
- `src/lib/graph/types.ts:1-70` — shared immutable graph contracts used across layout, render, and hit-testing
- `scripts/determinism-check.ts:27-385` — zero-dependency fixed-topology regression harness with normalized comparisons
- `scripts/perf-bench.ts:59-267` — benchmark runner for scaling evidence, currently part of the same validation surface

### Constraints

- No new frontend test framework or dependency additions without explicit approval
- This child bead should not absorb parent-bead responsibilities for `/spike-b` route validation or manual real-repo benchmarking
- The pipeline must remain pure relative to caller-owned input arrays
- Downstream renderer and hit-testing code depend on stable `LayoutResult` semantics

### Research findings

- The main determinism hot spots are `laneOf`, `activeLanes`, color-counter progression, and ref ordering inside the pipeline
- `scripts/perf-bench.ts` was identified as a drift risk when it mirrors layout logic instead of consuming the shared implementation
- Recent graph fixes in `bd-15p` and `bd-12d` show correctness/performance issues tend to surface at graph boundaries, so the data contract must be explicit before later waves build on it
- Script-first validation is sufficient for this child bead; manual real-history validation belongs to the parent spike’s final verification flow

## Affected Files

| File | Action | Description |
|------|--------|-------------|
| `src/lib/graph/layout.ts` | Modify | Harden deterministic lane assignment and document the contract |
| `src/lib/graph/types.ts` | Maybe modify | Extend shared contracts only if deterministic assertions require it |
| `scripts/determinism-check.ts` | Modify | Keep fixed-topology regression coverage aligned to the shared pipeline |
| `scripts/perf-bench.ts` | Modify | Benchmark the shared implementation and avoid drift |

## Tasks

### Task 1: Deterministic lane assignment contract [foundation]

The graph pipeline produces stable lane, color, and segment output for identical commit/ref input.

**Verification:**
- `npx tsx scripts/determinism-check.ts` passes repeatedly
- Identical input reports stable lane count and segment count across multiple runs
- `pnpm check` passes

**Metadata:**
```yaml
depends_on: []
parallel: true
files:
  - src/lib/graph/layout.ts
  - src/lib/graph/types.ts
  - scripts/determinism-check.ts
```

### Task 2: Shared benchmark alignment [validation]

Benchmark and regression scripts measure the same layout implementation used by the app rather than a copied variant.

**Verification:**
- `npx tsx scripts/perf-bench.ts` runs against imported shared graph logic
- No duplicated lane-assignment implementation remains in the benchmark path
- `pnpm check` passes after script alignment

**Metadata:**
```yaml
depends_on: ["Deterministic lane assignment contract"]
parallel: false
files:
  - scripts/perf-bench.ts
  - src/lib/graph/layout.ts
  - src/lib/graph/types.ts
```

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Lane reuse stays accidentally order-dependent | High | Lock determinism expectations into script fixtures and repeated-run comparisons |
| Ref ordering changes silently alter segment/color output | Medium | Treat ref ordering as part of deterministic output and cover it in regression snapshots |
| Benchmark script drifts from app behavior | High | Require shared imports from `src/lib/graph/*` instead of copied logic |
| Scope bleed into renderer or route work | Medium | Keep PRD script-first and pipeline-only; defer manual route validation to parent bead |

## Open Questions

None — validation boundary was resolved during refinement: this child bead is **script-first**, while real-repo/manual validation remains with the parent spike.

---

## Metadata

**Parent:** bd-12d
**Blocks:** bd-12d.2
