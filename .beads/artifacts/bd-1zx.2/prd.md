# PRD: Theme Switching Baseline (system/light/dark)

**Bead:** bd-1zx.2
**Parent:** bd-1zx (Design System Tokens Baseline)
**Depends on:** bd-1zx.1 (Token contract in app.css) — CLOSED
**Type:** task
**Priority:** P1

---

## Problem Statement

The token contract (bd-1zx.1) defines dark theme values in `:root`. There is no mechanism to switch to light mode or follow the system preference. Without theme switching, the app is locked to dark mode and cannot meet the MASTER.md requirement for system-following `prefers-color-scheme` behavior.

## Scope

### In-Scope

- Light theme token values defined as CSS custom properties
- System preference detection via `prefers-color-scheme`
- Early bootstrap script in `app.html` to prevent FOUC (flash of unstyled content)
- localStorage persistence for user theme preference
- Runtime theme switching utility (Svelte 5 runes-based)
- `data-theme` attribute on `<html>` element as the theme selector

### Out-of-Scope

- UI controls for theme switching (will be part of settings/preferences UI)
- Custom themes beyond system/light/dark
- Per-component theme overrides
- High-contrast or accessibility-specific themes

## Technical Context

**Current state:**
- `src/app.css` `:root` — 93 dark-theme tokens (colors, spacing, typography, layout, etc.)
- `src/app.html` — bare SvelteKit template, no bootstrap scripts
- No theme store or switching mechanism exists

**Architecture decision (from earlier refinement):**
- Theme selector: `data-theme` attribute on `<html>` (`system` | `light` | `dark`)
- Dark = default in `:root`; light values override via `[data-theme="light"]`
- `system` mode uses `prefers-color-scheme: light` media query to conditionally apply light values
- Early bootstrap in `<head>` reads localStorage before paint to avoid FOUC
- Svelte 5 `$state` rune for reactive theme store

## Requirements

### R1: Light Theme Token Values

Define light theme overrides for all color tokens that differ between dark and light modes. Values must match `design-system/mongit/MASTER.md` § Light Theme (lines 101-115).

**Tokens to override (from MASTER.md):**

| Token | Dark Value | Light Value |
|-------|-----------|-------------|
| `--color-bg` | `#0F1117` | `#FFFFFF` |
| `--color-bg-surface` | `#1A1D27` | `#F8F9FA` |
| `--color-bg-elevated` | `#242833` | `#FFFFFF` |
| `--color-bg-hover` | `#2A2E3A` | `#F1F3F5` |
| `--color-bg-active` | `#323744` | `#E9ECEF` |
| `--color-border` | `#2E3340` | `#DEE2E6` |
| `--color-border-subtle` | `#232730` | `#E9ECEF` |
| `--color-text-primary` | `#E8EAED` | `#1A1D27` |
| `--color-text-secondary` | `#8B8FA3` | `#6C757D` |
| `--color-text-muted` | `#565B6E` | `#ADB5BD` |

Additionally, light theme needs adjusted elevation shadows (lighter opacity) and scrollbar colors.

**Affected file:** `src/app.css`

### R2: Theme Selector Architecture

Use `data-theme` attribute on `<html>` element with three modes:
- `system` — follows OS preference via `prefers-color-scheme`
- `light` — forces light theme
- `dark` — forces dark theme (default `:root` values apply)

CSS structure:
```css
:root { /* dark tokens (existing) */ }
[data-theme="light"],
[data-theme="system"] { /* light overrides via media query */ }
```

For `system` mode, use `@media (prefers-color-scheme: light)` nested inside `[data-theme="system"]` selector.

**Affected file:** `src/app.css`

### R3: FOUC Prevention Bootstrap

Add inline `<script>` in `<head>` of `app.html` that:
1. Reads `mongit-theme` from localStorage
2. Defaults to `system` if no preference stored
3. Sets `data-theme` attribute on `<html>` before first paint
4. Must execute synchronously (no `defer` or `async`)

**Affected file:** `src/app.html`

### R4: Theme Store (Svelte 5 Runes)

Create `src/lib/stores/theme.svelte.ts` with:
- `theme` — reactive `$state<'system' | 'light' | 'dark'>` (reads from localStorage, defaults to `system`)
- `resolvedTheme` — `$derived<'light' | 'dark'>` (resolves `system` using `matchMedia`)
- `setTheme(mode)` — updates state, persists to localStorage, updates `data-theme` on `<html>`
- Listen for `prefers-color-scheme` changes when in `system` mode

**Affected file:** `src/lib/stores/theme.svelte.ts` (new)

### R5: System Preference Listener

When theme is `system`, the store must listen for OS-level changes via `matchMedia('(prefers-color-scheme: dark)').addEventListener('change', ...)` and update `resolvedTheme` reactively.

**Affected file:** `src/lib/stores/theme.svelte.ts`

## Affected Files

| File | Action | Description |
|------|--------|-------------|
| `src/app.css` | Modify | Add `[data-theme="light"]` and `[data-theme="system"]` selectors with light overrides |
| `src/app.html` | Modify | Add synchronous theme bootstrap script in `<head>` |
| `src/lib/stores/theme.svelte.ts` | Create | Theme store with Svelte 5 runes |

## Success Criteria

1. App renders in dark mode by default (existing behavior preserved)
2. Setting `data-theme="light"` on `<html>` switches all color tokens to light values
3. Setting `data-theme="system"` follows OS preference
4. Theme preference persists across page reloads via localStorage
5. No FOUC — theme is applied before first paint
6. `resolvedTheme` reactive value correctly tracks OS changes in `system` mode

## Verify

```bash
pnpm check    # 0 errors (svelte-check)
pnpm build    # Vite build succeeds
```

Manual verification:
- Inspect `src/app.css` for `[data-theme="light"]` selector with all 10 color overrides
- Inspect `src/app.html` for synchronous bootstrap script
- Inspect `src/lib/stores/theme.svelte.ts` for exported theme/resolvedTheme/setTheme

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| FOUC if bootstrap script is async | Medium | Script is inline, synchronous, in `<head>` before `%sveltekit.head%` |
| Light token values don't match MASTER.md | Low | Cross-reference each value against MASTER.md table |
| `matchMedia` not available in SSR | Low | SSR is disabled (`ssr=false`), all code runs in browser |

---

## Metadata

**Parent:** bd-1zx
**Depends on:** bd-1zx.1 (closed)
