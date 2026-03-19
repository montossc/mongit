# bd-12d.3 — Graph Interaction & Benchmark Evidence Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `skill({ name: "executing-plans" })` to implement this plan task-by-task.

**Goal:** Make `/spike-b` interaction readiness and benchmark evidence explicit, comparable, and durably recorded for future agents.

**Architecture:** Build on existing graph interaction hooks and benchmark surfaces (`GraphCanvas`, `FpsOverlay`, `perf-bench`) without redesigning renderer/layout layers. Capture evidence in a stable format in `.beads/artifacts/bd-12d/progress.txt` and mark bead-task evidence in `.beads/artifacts/bd-12d.3/progress.txt`.

**Tech Stack:** Svelte 5, SvelteKit, TypeScript, existing graph modules, zero-dependency TS scripts, Tauri Rust backend checks.

## Must-Haves

### Observable Truths
1. `/spike-b` provides an explicit interaction-validation surface (selection/hover/context/keyboard/detail).
2. Benchmark output is machine-comparable and tied to shared graph implementation.
3. Evidence is durably recorded in `.beads/artifacts/bd-12d/progress.txt`.

### Required Artifacts
| Artifact | Provides | Path |
|---|---|---|
| Interaction validation state | Explicit manual verification workflow + counters | `src/routes/spike-b/+page.svelte` |
| Overlay metric extension | Validation-friendly runtime visibility | `src/lib/graph/FpsOverlay.svelte` |
| Comparable benchmark output | Stable baseline summary for future diffing | `scripts/perf-bench.ts` |
| Durable evidence ledger | Automated + manual baseline history | `.beads/artifacts/bd-12d/progress.txt` |
| Bead task tracking | Per-task ship evidence | `.beads/artifacts/bd-12d.3/prd.json`, `.beads/artifacts/bd-12d.3/progress.txt` |

### Key Links
| From | To | Via | Risk |
|---|---|---|---|
| `/spike-b` UI | Graph interaction state | `onSelectCommit`, `onContextAction`, keyboard handling | Interaction appears functional but isn't explicitly verifiable |
| FpsOverlay | Validation evidence | props from route/canvas state | Missing counters/metrics reduce comparability |
| `perf-bench.ts` | Future comparisons | structured output | Drift or ad hoc output blocks regression detection |
| Evidence files | Future agents | append-only logs | Lost/unstable formatting reduces trust |

## Dependency Graph
- Task A (interaction validation surface) → creates explicit validation and metric wiring
- Task B (benchmark evidence baseline) → depends on Task A outputs and records durable evidence

Wave 1: Task A
Wave 2: Task B

---

## Task A — Interaction validation surface

**Files:**
- Modify: `src/routes/spike-b/+page.svelte`
- Modify: `src/lib/graph/FpsOverlay.svelte`
- Maybe modify: `src/lib/graph/GraphCanvas.svelte`
- Maybe modify: `src/lib/graph/CommitDetail.svelte`

**Steps:**
1. Add explicit interaction-validation state and checklist in `/spike-b`.
2. Add lightweight interaction counters (selection/context actions + timestamp).
3. Surface counters in route stats and/or `FpsOverlay` (minimal, validation-focused).
4. Keep existing behavior intact (no graph architecture changes).
5. Run `pnpm check`.
6. Commit task changes.

**Verification:**
- `pnpm check`
- Manual validation checklist is visible and actionable on `/spike-b`

---

## Task B — Comparable benchmark evidence baseline

**Files:**
- Modify: `scripts/perf-bench.ts`
- Modify: `.beads/artifacts/bd-12d/progress.txt`
- Modify: `.beads/artifacts/bd-12d.3/prd.json`
- Modify: `.beads/artifacts/bd-12d.3/progress.txt`

**Steps:**
1. Add stable benchmark summary output block in `perf-bench.ts` (machine-readable JSON line/block).
2. Run benchmark and capture fresh output.
3. Append automated evidence + manual-evidence template section to `bd-12d/progress.txt`.
4. Mark `passes=true` for completed tasks in `bd-12d.3/prd.json`.
5. Append per-task evidence with commit hashes to `bd-12d.3/progress.txt`.
6. Run full gates: `npx tsx scripts/perf-bench.ts`, `pnpm check`, `pnpm build`, `cargo check`.
7. Commit task + evidence files.

**Verification:**
- `npx tsx scripts/perf-bench.ts`
- `pnpm check`
- `pnpm build`
- `cargo check`

---

## Review & Compound
- Run deep 5-agent review and auto-fix critical/important findings.
- Re-run full verification after fixes.
- Capture compound learnings via `observation()` and bead comment.
