# Implementation Plan: bd-145.2 — Branch and tag overlays for real histories

## Architecture Summary

The commit graph renders ref badges (branches, tags, HEAD) inline on each row via `drawRefLabels()` in `render.ts`. Currently, ALL refs are drawn sequentially with no overflow strategy. For dense repos, this crowds the commit message column.

**Solution:** Sort refs by readability priority, cap visible badges using available pixel width, and draw a grouped-overflow `+N` badge for hidden refs. Mirror the same logic in `hitTest.ts`.

## Codebase Patterns Discovered

- `CommitNode.refs: RefData[]` — refs attached during `assignLanes()` in backend insertion order
- `drawRefLabels()` (render.ts:429) — iterates `node.refs` left-to-right, draws each badge
- `estimateRefBadgeWidth()` (render.ts:408) — shared badge width calculator (measured or estimated)
- `hitTest()` (hitTest.ts:19) — mirrors render badge positions for click targeting
- `getRefStyle()` (render.ts:567) — visual style per ref type (colors, borders)
- `drawCommitText()` (render.ts:492) — draws hash + message + author/time AFTER ref labels, uses `graphTextStartX` as its start X (does NOT account for ref badge widths shifting text right)
- `ROW_HEIGHT = 32`, `REF_LABEL_GAP = 6`, badge height = 16

**Key insight:** `drawCommitText()` starts at `graphTextStartX` (after graph lanes) — it does NOT shift right based on ref badge widths. Ref badges and commit text currently overlap on dense commits. The overflow strategy will naturally fix this by capping badge width.

## Task Dependency Graph

```
ui-1 → ui-2 → testing-1 → integration-1
```

All tasks are sequential — each depends on the previous.

---

## Task 1: ui-1 — Ref priority + grouped-overflow rendering

**Files:** `src/lib/graph/render.ts`

### 1.1 Add ref priority sorting

Add a `refPriority()` function and a `sortRefsByPriority()` function:

```typescript
// Priority order: Head (0) > LocalBranch (1) > Tag (2) > RemoteBranch (3)
function refPriority(ref: RefData): number {
  switch (ref.ref_type) {
    case 'Head': return 0;
    case 'LocalBranch': return 1;
    case 'Tag': return 2;
    case 'RemoteBranch': return 3;
    default: return 4;
  }
}

/** Sort refs by display priority (highest first), stable within same priority. */
export function sortRefsByPriority(refs: RefData[]): RefData[] {
  if (refs.length <= 1) return refs;
  return [...refs].sort((a, b) => refPriority(a) - refPriority(b));
}
```

Export `sortRefsByPriority` so `hitTest.ts` can reuse it.

### 1.2 Add overflow calculation

Add a `computeVisibleRefs()` function that determines which refs fit and how many overflow:

```typescript
export interface VisibleRefResult {
  visible: RefData[];
  overflowCount: number;
}

/** Max pixel width for ref badges before overflow kicks in. */
const REF_OVERFLOW_MAX_WIDTH = 280;
/** Min refs to always show (even if they exceed width). */
const REF_MIN_VISIBLE = 1;

export function computeVisibleRefs(
  sortedRefs: RefData[],
  measureText?: (text: string, isHead: boolean) => number,
): VisibleRefResult {
  if (sortedRefs.length === 0) return { visible: [], overflowCount: 0 };

  let totalWidth = 0;
  let visibleCount = 0;

  for (let i = 0; i < sortedRefs.length; i++) {
    const badgeWidth = estimateRefBadgeWidth(sortedRefs[i], measureText);
    const gap = i > 0 ? REF_LABEL_GAP : 0;
    const nextTotal = totalWidth + badgeWidth + gap;

    // Always show at least REF_MIN_VISIBLE refs
    if (i < REF_MIN_VISIBLE) {
      totalWidth = nextTotal;
      visibleCount = i + 1;
      continue;
    }

    // Reserve space for overflow badge (+N is ~30px)
    const overflowBadgeWidth = 30 + REF_LABEL_GAP;
    const remaining = sortedRefs.length - (i + 1);
    const needsOverflow = remaining > 0;
    const widthLimit = needsOverflow
      ? REF_OVERFLOW_MAX_WIDTH - overflowBadgeWidth
      : REF_OVERFLOW_MAX_WIDTH;

    if (nextTotal > widthLimit) break;

    totalWidth = nextTotal;
    visibleCount = i + 1;
  }

  return {
    visible: sortedRefs.slice(0, visibleCount),
    overflowCount: sortedRefs.length - visibleCount,
  };
}
```

Export `computeVisibleRefs` so `hitTest.ts` can reuse it.

### 1.3 Update `drawRefLabels()` to use priority + overflow

Replace the current simple loop with:

1. Sort refs via `sortRefsByPriority()`
2. Compute visible set via `computeVisibleRefs()`
3. Draw visible badges as before
4. If `overflowCount > 0`, draw a compact `+N` badge using a muted style

The `+N` badge style: use `textSecondary` colors (muted), no bold, smaller visual weight.

### 1.4 Export new constants

Export `REF_OVERFLOW_MAX_WIDTH` so hitTest can reference the same limits if needed.

### Verification

- `pnpm check` passes
- No regressions on existing graph rendering

---

## Task 2: ui-2 — Align hitTest with visible overlay geometry

**Files:** `src/lib/graph/hitTest.ts`, `src/lib/graph/render.ts`

### 2.1 Import new functions

Import `sortRefsByPriority` and `computeVisibleRefs` from `render.ts`.

### 2.2 Update hit-test ref loop

Replace the current ref iteration (hitTest.ts:49-60) with:

1. Sort refs via `sortRefsByPriority()`
2. Compute visible set via `computeVisibleRefs()`
3. Only create hit targets for visible refs
4. Skip the overflow badge area (no individual ref targeting for hidden refs)

The overflow badge itself does NOT need to be a hit target for this bead (no overflow-click interactions).

### Verification

- `pnpm check` passes
- Hit-test logic matches render geometry

---

## Task 3: testing-1 — Dense-ref regression coverage

**Files:** `src/lib/smoke/graph.test.ts`, `src/lib/graph/render.ts`, `src/lib/graph/hitTest.ts`

### 3.1 Add unit tests for priority sorting

Test `sortRefsByPriority()` with:
- Empty array
- Single ref
- Mixed ref types → verify Head first, then LocalBranch, then Tag, then RemoteBranch
- Stability: same-type refs maintain original order

### 3.2 Add unit tests for overflow computation

Test `computeVisibleRefs()` with:
- Small set that fits → all visible, overflowCount = 0
- Large set that overflows → verify visible subset + correct overflowCount
- Single ref → always visible even if "wide"
- All same-type refs

### 3.3 Add hit-test alignment test

Test that `hitTest()` with dense refs only produces hit targets for visible refs, not overflow ones.

### Verification

- `pnpm smoke` passes (or `npx vitest run`)
- Dense-ref scenarios covered

---

## Task 4: integration-1 — Validate production history readability

**Files:** No code changes expected

### 4.1 Run full verification suite

```bash
pnpm check
pnpm build
cargo check  # in src-tauri/
```

### 4.2 Manual verification (checkpoint)

- Open `pnpm tauri dev`
- Navigate to `/repo/history`
- Verify overlay readability on repo with dense refs
- Verify commit selection + detail sync still works

### Verification

- All automated checks pass
- No regressions in UI behavior
