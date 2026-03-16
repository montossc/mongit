# PRD: Base UI Primitives Aligned to Tokens

**Bead:** bd-1zx.3
**Parent:** bd-1zx (Design System Tokens Baseline)
**Depends on:** bd-1zx.1 (Token contract in app.css) — CLOSED
**Type:** task
**Priority:** P1

---

## Problem Statement

The token contract (bd-1zx.1) and theme switching (bd-1zx.2) are complete, but there are no reusable Svelte components that consume these tokens. The existing `+page.svelte` has ad-hoc inline styles for buttons and inputs (lines 322-375) that partially use tokens but aren't reusable. Without shared primitives, every new view will reinvent styling patterns, leading to inconsistency and drift from the design system.

## Scope

### In-Scope

- **Button** component: 4 variants (primary, secondary, ghost, danger), 3 sizes (compact, default, prominent)
- **Input** component: text input with focus ring, placeholder, disabled state, optional mono font
- **Badge** component: 3 semantic variants (branch, tag, remote) with pill shape
- **Panel** component: layout surface with optional header, border, elevation
- Proper focus ring utility using `--focus-ring-*` tokens
- All components consume design tokens exclusively (no hardcoded values)
- Svelte 5 patterns: `$props()` rune, callback props, `{@render children()}` snippets

### Out-of-Scope

- Complex components (CommandPalette, FileTree, DiffView)
- Layout shells (Sidebar, TitleBar, StatusBar) — separate bead
- Refactoring existing `+page.svelte` to use new components (follow-up work)
- Component tests (separate bead)
- Storybook or component playground

## Technical Context

**Current state:**
- `src/app.css` — 283 lines with full token contract + theme switching
- `src/lib/components/` — 4 spike-specific components (BenchmarkPanel, DiffViewer, MergeEditor, WatcherMonitor), no `ui/` subdirectory
- `src/routes/+page.svelte` — ad-hoc `.btn`, `.btn-primary`, `.btn-secondary`, `.btn-ghost`, `.repo-input` styles (lines 322-375)
- Token contract provides: `--size-button-compact/default/prominent`, `--size-input`, `--size-badge`, `--focus-ring-*`, all color/spacing/radius tokens

**Target directory:** `src/lib/components/ui/`

**Svelte 5 patterns (from MASTER.md §10):**
- Props via `$props()` rune
- Events via callback props (not `createEventDispatcher`)
- Children via `{@render children()}` snippet
- Styles via scoped `<style>` blocks referencing CSS custom properties

## Requirements

### R1: Button Component

Create `src/lib/components/ui/Button.svelte` with:

**Variants (from MASTER.md §7 Buttons):**

| Variant   | Background              | Text                        | Border              |
|-----------|------------------------|-----------------------------|---------------------|
| primary   | `--color-accent`       | white                       | none                |
| secondary | `--color-bg-elevated`  | `--color-text-primary`      | `--color-border`    |
| ghost     | transparent            | `--color-text-secondary`    | none                |
| danger    | `--color-danger`       | white                       | none                |

**Sizes:**

| Size      | Height                       | Padding           |
|-----------|-----------------------------|--------------------|
| compact   | `--size-button-compact` (28px) | 6px 8px          |
| default   | `--size-button-default` (32px) | 8px 12px         |
| prominent | `--size-button-prominent` (36px) | 10px 16px      |

**Props:** `variant` (default: 'secondary'), `size` (default: 'default'), `disabled`, `type` (default: 'button'), `onclick`, and children snippet.

**States:** hover (lighten/darken per variant), disabled (opacity 0.5, cursor not-allowed), focus (2px ring `--color-accent` with 2px offset).

**Affected file:** `src/lib/components/ui/Button.svelte` (new)

### R2: Input Component

Create `src/lib/components/ui/Input.svelte` with:

- Height: `--size-input` (32px)
- Background: `--color-bg-surface`
- Border: 1px solid `--color-border`
- Focus: `--color-accent` border + focus ring (2px, offset)
- Placeholder: `--color-text-muted`
- Font: `--font-sans` 13px default, `--font-mono` when `mono` prop is true
- Disabled: opacity 0.5, cursor not-allowed

**Props:** `value` (bindable), `placeholder`, `type` (default: 'text'), `disabled`, `mono` (boolean), `oninput`, `onkeydown`, standard input attributes via rest props.

**Affected file:** `src/lib/components/ui/Input.svelte` (new)

### R3: Badge Component

Create `src/lib/components/ui/Badge.svelte` with:

- Height: `--size-badge` (20px)
- Padding: 2px 6px
- Font: 11px, weight 500
- Radius: `--radius-full` (pill)

**Semantic variants (from MASTER.md §7 Badges):**

| Variant  | Background                      | Text               |
|----------|---------------------------------|--------------------|
| branch   | `--color-accent-muted`          | `--color-accent`   |
| tag      | `--color-warning-muted`         | `--color-warning`  |
| remote   | `rgba(96, 165, 250, 0.15)` (info 15%) | `--color-info` |
| default  | `--color-bg-elevated`           | `--color-text-secondary` |

**Props:** `variant` (default: 'default'), and children snippet.

**Affected file:** `src/lib/components/ui/Badge.svelte` (new)

### R4: Panel Component

Create `src/lib/components/ui/Panel.svelte` with:

- Background: `--color-bg-surface`
- Border: 1px solid `--color-border`
- Radius: `--radius-md`
- Padding: `--space-5` (12px) default
- Optional header slot with bottom border separator
- Optional elevation level (0-4)

**Props:** `elevation` (0-4, default: 0), `padding` (default: true), `header` snippet (optional), `children` snippet.

**Affected file:** `src/lib/components/ui/Panel.svelte` (new)

### R5: Barrel Export

Create `src/lib/components/ui/index.ts` that re-exports all components for clean imports:
```ts
export { default as Button } from './Button.svelte';
export { default as Input } from './Input.svelte';
export { default as Badge } from './Badge.svelte';
export { default as Panel } from './Panel.svelte';
```

**Affected file:** `src/lib/components/ui/index.ts` (new)

## Affected Files

| File | Action | Description |
|------|--------|-------------|
| `src/lib/components/ui/Button.svelte` | Create | Button with 4 variants, 3 sizes, focus ring |
| `src/lib/components/ui/Input.svelte` | Create | Text input with focus ring, mono option |
| `src/lib/components/ui/Badge.svelte` | Create | Semantic badge with 4 variants |
| `src/lib/components/ui/Panel.svelte` | Create | Layout surface with optional header/elevation |
| `src/lib/components/ui/index.ts` | Create | Barrel re-export |

## Success Criteria

1. All 4 components created in `src/lib/components/ui/`
2. All components use design tokens exclusively — no hardcoded color/size values
3. All interactive elements have focus rings using `--focus-ring-*` tokens
4. All clickable elements have `cursor: pointer`
5. Hover states use `var(--transition-fast)` timing
6. Components follow Svelte 5 patterns (`$props()`, callback props, snippets)
7. `pnpm check` passes with 0 errors
8. `pnpm build` succeeds

## Verify

```bash
pnpm check    # 0 errors (svelte-check)
pnpm build    # Vite build succeeds
```

Manual verification:
- Each component file exists and exports a default Svelte component
- Barrel export re-exports all 4 components
- No hardcoded hex colors or pixel sizes (only token references)

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Svelte 5 snippet API for children/header slots | Low | Use `{@render children?.()}` pattern, verified in SvelteKit 2.x |
| Rest props forwarding | Low | Use `{...restProps}` with `$props()` destructuring |
| TypeScript types for variants/sizes | Low | Use union literal types |

## Tasks

### Button Component [functional]

Create `src/lib/components/ui/Button.svelte` with 4 variants (primary, secondary, ghost, danger), 3 sizes (compact, default, prominent), focus ring, hover/disabled states per MASTER.md §7.

**Verification:**
- File exists at `src/lib/components/ui/Button.svelte`
- Component accepts variant, size, disabled, onclick, children props
- All colors reference CSS custom properties
- Focus ring uses `--focus-ring-*` tokens
- pnpm check passes with 0 errors

### Input Component [functional]

Create `src/lib/components/ui/Input.svelte` with focus ring, mono font option, placeholder styling, disabled state per MASTER.md §7.

**Verification:**
- File exists at `src/lib/components/ui/Input.svelte`
- Component accepts value (bindable), placeholder, type, disabled, mono, oninput, onkeydown props
- Focus ring uses `--focus-ring-*` tokens
- pnpm check passes with 0 errors

### Badge Component [functional]

Create `src/lib/components/ui/Badge.svelte` with 4 semantic variants (branch, tag, remote, default) per MASTER.md §7.

**Verification:**
- File exists at `src/lib/components/ui/Badge.svelte`
- Component accepts variant and children props
- Pill shape uses `--radius-full`
- pnpm check passes with 0 errors

### Panel Component [functional]

Create `src/lib/components/ui/Panel.svelte` with surface background, optional header, elevation levels per MASTER.md §5.

**Verification:**
- File exists at `src/lib/components/ui/Panel.svelte`
- Component accepts elevation, padding, header snippet, children snippet
- Surface uses `--color-bg-surface`, border uses `--color-border`
- pnpm check passes with 0 errors

### Barrel Export [functional]

Create `src/lib/components/ui/index.ts` re-exporting all 4 components.

**Verification:**
- File exists at `src/lib/components/ui/index.ts`
- Exports Button, Input, Badge, Panel
- pnpm check passes with 0 errors
- pnpm build succeeds

---

## Metadata

**Parent:** bd-1zx
**Depends on:** bd-1zx.1 (closed), bd-1zx.2 (closed)
