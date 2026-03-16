# PRD: Token Contract in `src/app.css`

**Bead:** bd-1zx.1
**Parent:** bd-1zx (Design System Tokens)
**Type:** task
**Status:** Ready for implementation

---

## Problem Statement

`src/app.css` currently defines color, spacing, font-stack, radius, elevation, z-index, and transition tokens — but is missing the typography scale, merge/conflict colors, focus/accessibility tokens, component sizing, and layout dimensions specified in `design-system/mongit/MASTER.md`. Without these, downstream beads (bd-1zx.2 theme switching, bd-1zx.3 primitives) cannot reference a complete vocabulary.

## Scope

### In-Scope

Expand the CSS custom property contract in `src/app.css` `:root` to include:

1. **Typography scale** — 9 levels (heading-lg, heading-md, heading-sm, body, body-sm, caption, mono, mono-sm, mono-xs) with font-size, font-weight, and line-height tokens per level
2. **Merge/conflict colors** — `--color-conflict-ours`, `--color-conflict-theirs`, `--color-conflict-base` (from MASTER.md §2)
3. **Focus/accessibility tokens** — `--focus-ring-width`, `--focus-ring-color`, `--focus-ring-offset` (from MASTER.md §6 + §9)
4. **Component sizing tokens** — button heights (compact/default/prominent), input height, row heights (compact/default/commit), badge height
5. **Layout dimension tokens** — sidebar width, detail panel width, title bar height, status bar height

### Out-of-Scope

- Light theme values (bd-1zx.2)
- Svelte component files (bd-1zx.3)
- CodeMirror-specific overrides (future bead)
- `prefers-color-scheme` media queries (bd-1zx.2)
- Any changes outside `:root` block in `src/app.css`

## Requirements

### R1: Typography Scale Tokens

Add tokens per MASTER.md §3 Type Scale table:

```css
/* Typography Scale */
--text-heading-lg-size: 20px;
--text-heading-lg-weight: 600;
--text-heading-lg-leading: 1.3;

--text-heading-md-size: 16px;
--text-heading-md-weight: 600;
--text-heading-md-leading: 1.4;

--text-heading-sm-size: 14px;
--text-heading-sm-weight: 600;
--text-heading-sm-leading: 1.4;

--text-body-size: 13px;
--text-body-weight: 400;
--text-body-leading: 1.5;

--text-body-sm-size: 12px;
--text-body-sm-weight: 400;
--text-body-sm-leading: 1.5;

--text-caption-size: 11px;
--text-caption-weight: 400;
--text-caption-leading: 1.4;

--text-mono-size: 13px;
--text-mono-weight: 400;
--text-mono-leading: 1.6;

--text-mono-sm-size: 12px;
--text-mono-sm-weight: 400;
--text-mono-sm-leading: 1.5;

--text-mono-xs-size: 11px;
--text-mono-xs-weight: 400;
--text-mono-xs-leading: 1.4;
```

### R2: Merge/Conflict Colors

Add from MASTER.md §2 Git-Specific Semantic Colors:

```css
/* Merge/conflict */
--color-conflict-ours: #3ECF8E;
--color-conflict-theirs: #60A5FA;
--color-conflict-base: #FBBF24;
```

### R3: Focus/Accessibility Tokens

Add from MASTER.md §6 and §9:

```css
/* Focus */
--focus-ring-width: 2px;
--focus-ring-color: var(--color-accent);
--focus-ring-offset: 2px;
```

### R4: Component Sizing Tokens

Add from MASTER.md §7 Components:

```css
/* Component sizing */
--size-button-compact: 28px;
--size-button-default: 32px;
--size-button-prominent: 36px;
--size-input: 32px;
--size-row-compact: 28px;
--size-row-default: 32px;
--size-row-commit: 40px;
--size-badge: 20px;
```

### R5: Layout Dimension Tokens

Add from MASTER.md §4 Layout Regions:

```css
/* Layout dimensions */
--layout-titlebar-height: 38px;
--layout-statusbar-height: 24px;
--layout-sidebar-width: 220px;
--layout-sidebar-min: 180px;
--layout-sidebar-max: 320px;
--layout-detail-width: 360px;
--layout-detail-min: 280px;
--layout-detail-max: 600px;
```

## Success Criteria

1. `pnpm check` passes with zero errors
2. `pnpm build` succeeds
3. All tokens from MASTER.md §2 (conflict), §3 (type scale), §4 (layout), §6 (focus), §7 (sizing) have corresponding CSS custom properties in `src/app.css`
4. Existing tokens unchanged (no regressions)
5. Token naming convention is consistent: `--{category}-{name}` pattern

## Affected Files

| File | Change |
|------|--------|
| `src/app.css` | Add ~50 new CSS custom properties to `:root` |

## Tasks

1. Add typography scale tokens after existing Typography section (line ~57)
2. Add merge/conflict color tokens after Git diff section (line ~40)
3. Add focus/accessibility tokens after Transitions section (line ~92)
4. Add component sizing tokens (new section after focus)
5. Add layout dimension tokens (new section after sizing)
6. Verify `pnpm check` + `pnpm build`

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Token naming conflicts with future framework | Low | Prefix convention (`--text-`, `--size-`, `--layout-`) avoids collisions |
| Unused tokens bloat CSS | Low | Tree-shaking not relevant for CSS vars; ~50 tokens < 2KB |

## Open Questions

None — all values sourced directly from MASTER.md.

---

## Metadata

**Parent:** bd-1zx
**Dependencies:** None (first child bead in the family)
**Blocks:** bd-1zx.2 (theme switching needs complete token set), bd-1zx.3 (primitives reference sizing/typography tokens)
