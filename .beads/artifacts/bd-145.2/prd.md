# PRD: Branch and tag overlays for real histories

**Bead:** bd-145.2
**Parent:** bd-145 (Commit Graph Productization)
**Depends on:** bd-145 (parent-child), bd-145.1
**Type:** task
**Priority:** P1

```yaml
requirements_score:
  total: 92
  breakdown:
    business_value: 26
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

mongit now has a production history workspace at `src/routes/repo/history/+page.svelte`, but the branch and tag badges drawn inside the graph still use the spike-era overlay model: every ref badge is rendered inline in insertion order with no density strategy. On real repositories, that makes overlays expand horizontally until they crowd the commit message column, reduce scanability, and make history comprehension worse precisely in the repositories where overlays matter most.

This bead improves practical readability for dense histories without reopening the graph architecture. The goal is to keep branch and tag overlays understandable in day-to-day use by applying a stable prioritization and grouped-overflow strategy inside the existing Canvas renderer, while preserving one-row graph density and the current history-route integration established by `bd-145.1`.

### Why now?

- `bd-145.1` moved the graph into the main app shell, so overlay readability is now a product issue rather than a spike-only limitation.
- The parent feature `bd-145` explicitly includes usable ref overlays as part of graph productization.
- Current rendering in `src/lib/graph/render.ts` draws every ref badge in sequence with no overflow behavior, which is acceptable for synthetic or sparse histories but degrades on real repositories.

### Who is affected?

- **Primary users:** solo power developers using the `/repo/history` workspace to understand branch position and tag context in real repositories
- **Secondary users:** downstream work on `bd-1x9` and later graph UX beads that need a stable overlay contract rather than another round of spike-level badge behavior

## Scope

### In-Scope

- Improve ref badge readability for dense histories inside the existing Canvas graph renderer
- Keep overlays on a single row per commit using prioritized visible refs plus grouped overflow
- Define stable ordering so the most important refs remain visible first on crowded commits
- Keep hit-testing aligned with the visible overlay model so row and ref interactions stay correct
- Add focused verification for dense ref scenarios using existing graph smoke/test patterns and the production `/repo/history` route

### Out-of-Scope

- New graph routes, layout algorithms, or lane assignment redesigns
- DOM/SVG overlay layers, tooltip systems, or multi-line badge wrapping
- New backend IPC commands or changes to `RefData` shape unless a verified blocker is found
- Ref-click actions, filtering, pinning, hiding, or overlay interactions beyond current graph targeting
- Broader history investigation workflows such as blame, compare, or file-history tooling

## Proposed Solution

### Overview

Refine ref overlays inside `src/lib/graph/render.ts` so each commit row shows the highest-priority refs first, then collapses remaining refs into a compact grouped-overflow badge instead of rendering an unbounded sequence of full badges. The renderer should remain Canvas-only and single-row, with deterministic output and no extra layout pass. `src/lib/graph/hitTest.ts` must mirror whatever visible badge geometry the renderer uses so ref hit zones stay trustworthy.

The grouped-overflow strategy chosen during refinement is: keep the most important refs visible, then represent the remainder with a compact summary badge such as `+N` or equivalent grouped text. This preserves the existing row-height model (`ROW_HEIGHT`) and avoids the complexity and density penalties of multi-line wrapping.

### User Flow

1. User opens `/repo/history` for an active repository.
2. The history workspace loads commit graph data exactly as it does today.
3. On commits with few refs, badges render normally and remain easy to read.
4. On commits with many refs, the graph shows stable high-priority refs first and groups overflow rather than letting badges crowd the message column.
5. User can still target visible ref badges correctly, and the graph remains readable across dense histories.

## Requirements

### Functional Requirements

#### R1. Dense ref overlays must remain readable on a single row

The graph must avoid rendering an unbounded horizontal chain of branch/tag badges on dense commits.

**Scenarios:**
- **WHEN** a commit has a small number of refs **THEN** the graph renders the normal visible badges without grouped overflow
- **WHEN** a commit has more refs than can be shown cleanly in the overlay area **THEN** the graph keeps the row single-line and replaces excess refs with a grouped-overflow badge instead of crowding commit text indefinitely
- **WHEN** grouped overflow is shown **THEN** the result communicates that additional refs exist without forcing a second row or a new panel

#### R2. Visible refs must follow a stable readability-first priority order

The renderer must show the most important refs first on crowded commits so users can quickly identify current branch context.

**Scenarios:**
- **WHEN** a commit contains `Head`, local branches, remote branches, and tags **THEN** visible badges are rendered in a deterministic priority order rather than raw backend insertion order alone
- **WHEN** two renders receive the same ordered commit/ref inputs **THEN** the visible badge order and grouped-overflow result are stable across renders
- **WHEN** overflow occurs **THEN** the refs removed from direct display are the lower-priority or later refs, not the highest-value branch context

#### R3. Overlay refinement must preserve current graph route and selection behavior

Improved overlays must attach to the existing graph renderer without regressing the `/repo/history` route introduced in `bd-145.1`.

**Scenarios:**
- **WHEN** the history workspace loads successfully **THEN** the graph still renders through `GraphCanvas.svelte` using the existing `renderGraph()` pipeline
- **WHEN** commit selection changes in `/repo/history` **THEN** overlay refinement does not break graph highlighting or detail-panel synchronization
- **WHEN** no overlay overflow is needed **THEN** graph behavior matches current badge rendering closely enough to avoid regressions on sparse histories

#### R4. Ref hit-testing must stay aligned with visible overlays

Any change to badge visibility or geometry must keep `hitTest()` aligned with what is actually drawn.

**Scenarios:**
- **WHEN** a visible badge is drawn **THEN** its hit zone maps to the same visible badge geometry and ordering
- **WHEN** refs are grouped into overflow **THEN** hit-testing does not pretend that hidden badges are individually targetable at their old positions
- **WHEN** the pointer lands outside visible badges but within the row **THEN** row targeting still works as expected

#### R5. The solution must stay inside the existing graph architecture

This bead is a readability refinement, not a graph-system redesign.

**Scenarios:**
- **WHEN** overlays are improved **THEN** the implementation stays within `render.ts`/`hitTest.ts`-style graph rendering boundaries instead of adding a new overlay subsystem
- **WHEN** the feature is implemented **THEN** it does not require new Tauri commands, route restructuring, or multi-line row layout
- **WHEN** future beads extend graph UX **THEN** they inherit a stable grouped-overflow overlay contract rather than another spike-only rendering rule

### Non-Functional Requirements

- **Performance:** Overlay refinement should avoid extra full-layout work beyond the existing `assignLanes()` pipeline and should keep dense-history redraw cost acceptable for the production graph surface.
- **Security:** The bead must not widen IPC or repository-path trust boundaries; it reuses current loaded graph data only.
- **Accessibility:** The graph remains keyboard reachable through existing `GraphCanvas.svelte` interactions, and readability changes must not rely on hover-only disclosure as the primary signal.
- **Compatibility:** Must work with the current Canvas renderer, `hitTest()` model, `LayoutResult`/`RefData` types, and the `/repo/history` route added in `bd-145.1`.
- **Determinism:** For identical ordered inputs, overlay ordering and grouped-overflow output must be stable across runs.

## Success Criteria

- [ ] Dense commits in `/repo/history` remain visually readable because only priority refs are shown directly and overflow is grouped compactly.
  - Verify: manual app check in `pnpm tauri dev` using a repository with many refs on the same commit
- [ ] Visible ref ordering is stable and keeps the most important refs visible first.
  - Verify: focused graph render/hit-test coverage plus repeat manual checks on the same repository state
- [ ] Visible badge hit targets match the rendered overlay model after overflow is introduced.
  - Verify: manual app check in `pnpm tauri dev` and focused graph coverage where available
- [ ] Project verification remains green after implementation.
  - Verify: `pnpm check`
  - Verify: `pnpm smoke`
  - Verify: `pnpm build`
  - Verify: `cargo check` (run in `src-tauri/`)

## Technical Context

### Existing Patterns

- `src/lib/graph/render.ts:408` - `estimateRefBadgeWidth()` centralizes badge width calculation and already supports measured vs estimated text widths
- `src/lib/graph/render.ts:429` - `drawRefLabels()` currently renders every ref badge inline from left to right with no overflow strategy
- `src/lib/graph/render.ts:567` - `getRefStyle()` already encodes ref-type visual language for local branches, remote branches, tags, and `Head`
- `src/lib/graph/hitTest.ts:49` - ref hit-testing currently assumes every badge is drawn sequentially with the same width-estimation contract
- `src/routes/repo/history/+page.svelte:25` - production history route already fetches commit log + refs and should remain the integration surface for overlay work
- `src/lib/graph/GraphCanvas.svelte:87` - graph rendering still flows through `renderGraph()` with a controlled selection contract from `bd-145.1`
- `src/lib/smoke/graph.test.ts:4` - graph validation currently uses Vitest smoke tests and is the closest existing test pattern for graph-specific regression coverage

### Key Constraints

- `bd-145.1` is complete, so this bead must extend the production history route rather than returning to `/spike-b`.
- Current badge rendering is Canvas-only; adding DOM/SVG overlays would cut against the performance rationale of the graph spike.
- `ROW_HEIGHT` and current graph row geometry assume a single-line row; multi-line ref wrapping would widen scope into layout changes.
- `hitTest.ts` and `render.ts` are coupled through `estimateRefBadgeWidth()` and sequential badge geometry, so overlay changes must update both together.
- The graph renderer already has known pre-existing performance risks on dense histories; this bead should improve readability without introducing a second expensive rendering path.
- No relevant `TODO` or `FIXME` markers were found in `src/lib/graph/` during refinement.

### Affected Files

```yaml
files:
  - src/lib/graph/render.ts # Add priority ordering and grouped-overflow rendering for dense ref badges
  - src/lib/graph/hitTest.ts # Keep ref hit zones aligned with visible badges and grouped overflow
  - src/lib/smoke/graph.test.ts # Add dense-ref regression coverage matching existing graph validation patterns
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Overlay grouping hides the wrong refs and removes critical branch context | Medium | High | Define and test a stable readability-first priority order with highest-value refs shown first |
| Rendered badge geometry and hit-testing drift apart | High | High | Treat `render.ts` and `hitTest.ts` as a paired change set and verify visible badge targeting manually and in focused coverage |
| Dense-history readability work expands into layout or routing redesign | Medium | Medium | Keep the bead scoped to Canvas ref overlay rendering inside the existing `/repo/history` integration |
| Overflow treatment becomes non-deterministic across renders | Low | High | Base ordering on stable ref priority rules and current ordered inputs; avoid random/time-based logic |
| Additional overlay logic causes noticeable redraw regressions | Medium | Medium | Reuse existing width-estimation and render pass structure; verify with `pnpm smoke` and manual history checks |

## Open Questions

None.

## Tasks

### Ref priority and grouped-overflow rendering [ui]

Update the graph renderer so dense commits show a stable priority-ordered subset of refs plus a grouped-overflow badge instead of an unbounded inline sequence.

**Metadata:**
```yaml
depends_on: []
parallel: false
conflicts_with: []
files:
  - src/lib/graph/render.ts
```

**Verification:**
- `pnpm check`
- Manual app check in `pnpm tauri dev` confirms sparse commits still render normal badges
- Manual app check confirms dense commits keep one-row readability with grouped overflow

### Align visible ref hit-testing with overlay geometry [ui]

Keep `hitTest()` synchronized with the new visible overlay model so ref and row targeting stay trustworthy after overflow grouping.

**Metadata:**
```yaml
depends_on: ["Ref priority and grouped-overflow rendering"]
parallel: false
conflicts_with: []
files:
  - src/lib/graph/hitTest.ts
  - src/lib/graph/render.ts
```

**Verification:**
- `pnpm check`
- Manual app check confirms visible badges remain targetable
- Manual app check confirms hidden overflow refs are not individually targetable at stale positions

### Add dense-ref graph regression coverage [testing]

Extend existing graph smoke-style validation with dense-ref scenarios that prove ordering, overflow, and rendering contracts remain stable.

**Metadata:**
```yaml
depends_on: ["Ref priority and grouped-overflow rendering", "Align visible ref hit-testing with overlay geometry"]
parallel: false
conflicts_with: []
files:
  - src/lib/smoke/graph.test.ts
  - src/lib/graph/render.ts
  - src/lib/graph/hitTest.ts
```

**Verification:**
- `pnpm smoke`
- Focused assertions cover dense commits with multiple local branches, remote branches, tags, and `Head`

### Validate production history readability [integration]

Prove the refined overlays work in the production `/repo/history` route without regressing graph-detail binding from `bd-145.1`.

**Metadata:**
```yaml
depends_on: ["Add dense-ref graph regression coverage"]
parallel: false
conflicts_with: []
files:
  - src/routes/repo/history/+page.svelte
  - src/lib/graph/GraphCanvas.svelte
  - src/lib/graph/render.ts
```

**Verification:**
- `pnpm check`
- `pnpm build`
- `cargo check`
- Manual app check in `pnpm tauri dev` confirms overlay readability in `/repo/history` and no regression in commit selection/detail sync

---

## Notes

- Readability strategy chosen during refinement: keep one-row overlays and group overflow rather than truncating every badge or wrapping badges onto additional lines.
- The current renderer already preserves caller-provided ref order through `assignLanes()`; this bead adds a readability-first display order at render time without changing backend ref shape.
- This PRD intentionally avoids promising new ref interaction semantics so `bd-145.2` can stay focused on overlay readability.
