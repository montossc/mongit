---
project: mongit
generated: 2026-03-14
style: Premium Developer Tool
reference: Linear, Raycast, Arc, JetBrains
---

# mongit Design System — MASTER

> A standalone macOS Git client. Premium calm, keyboard-first, sharp typography.

> **LOGIC:** When building a specific page, first check `design-system/mongit/pages/[page-name].md`.
> If that file exists, its rules **override** this Master file.
> If not, strictly follow the rules below.

---

## 1. Design Philosophy

| Principle | Implementation |
|-----------|---------------|
| **Ambient status** | VCS state always visible — gutter markers, branch widget, file tree colors |
| **Operation preview** | Show what will happen before every destructive action |
| **Undo everything** | Every mutation reversible with one action |
| **Keyboard-first** | Every action via shortcut or CMD+K command palette |
| **Progressive disclosure** | Clean surface for beginners; depth for experts |
| **Premium calm** | Sharp typography, generous spacing, no visual noise |

**Tone:** Confident but quiet. Like Linear — not flashy, just precise.

**Anti-patterns to avoid:**
- No emoji icons (use SVG: Lucide icons)
- No decorative animations (only functional transitions)
- No layout-shifting hover states (use color/opacity only)
- No arbitrary z-index values (use scale system)
- No flat design without depth (use subtle elevation)

---

## 2. Color Palette

### Dark Theme (Primary)

| Token | Hex | Usage |
|-------|-----|-------|
| `--color-bg` | `#0F1117` | App background (near-black, not pure black) |
| `--color-bg-surface` | `#1A1D27` | Panels, sidebars, cards |
| `--color-bg-elevated` | `#242833` | Dropdowns, popovers, modals |
| `--color-bg-hover` | `#2A2E3A` | List item hover, button hover |
| `--color-bg-active` | `#323744` | Selected/active state |
| `--color-border` | `#2E3340` | Default borders |
| `--color-border-subtle` | `#232730` | Subtle separators |
| `--color-text-primary` | `#E8EAED` | Primary text |
| `--color-text-secondary` | `#8B8FA3` | Secondary labels, descriptions |
| `--color-text-muted` | `#565B6E` | Disabled, placeholder, timestamps |

### Accent & Semantic Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--color-accent` | `#53C1DE` | Brand accent, links, active indicators |
| `--color-accent-hover` | `#7DD3E8` | Accent hover state |
| `--color-accent-muted` | `#53C1DE26` | Accent backgrounds (15% opacity) |
| `--color-success` | `#3ECF8E` | Staged files, push success, added lines |
| `--color-success-muted` | `#3ECF8E1A` | Success background |
| `--color-danger` | `#F87171` | Deleted lines, errors, destructive actions |
| `--color-danger-muted` | `#F871711A` | Error background |
| `--color-warning` | `#FBBF24` | Conflicts, modified files, warnings |
| `--color-warning-muted` | `#FBBF241A` | Warning background |
| `--color-info` | `#60A5FA` | Informational badges, hints |

### Git-Specific Semantic Colors

| Token | Hex | Usage |
|-------|-----|-------|
| `--color-diff-added-bg` | `#3ECF8E15` | Added line background in diff |
| `--color-diff-added-text` | `#3ECF8E` | Added line gutter marker |
| `--color-diff-removed-bg` | `#F8717115` | Removed line background in diff |
| `--color-diff-removed-text` | `#F87171` | Removed line gutter marker |
| `--color-diff-modified-bg` | `#60A5FA15` | Modified line background |
| `--color-diff-hunk-header` | `#53C1DE40` | Hunk header background |
| `--color-conflict-ours` | `#3ECF8E` | Our changes in merge |
| `--color-conflict-theirs` | `#60A5FA` | Their changes in merge |
| `--color-conflict-base` | `#FBBF24` | Base in 3-way merge |

### Branch Graph Colors (10-color cycle)

```
--graph-color-0: #53C1DE   /* cyan — default/main */
--graph-color-1: #3ECF8E   /* green */
--graph-color-2: #F87171   /* red */
--graph-color-3: #FBBF24   /* amber */
--graph-color-4: #A78BFA   /* violet */
--graph-color-5: #FB923C   /* orange */
--graph-color-6: #F472B6   /* pink */
--graph-color-7: #60A5FA   /* blue */
--graph-color-8: #34D399   /* emerald */
--graph-color-9: #E879F9   /* fuchsia */
```

### Light Theme (System-following)

| Token | Light Value | Notes |
|-------|-------------|-------|
| `--color-bg` | `#FFFFFF` | Pure white background |
| `--color-bg-surface` | `#F8F9FA` | Panel backgrounds |
| `--color-bg-elevated` | `#FFFFFF` | Elevated with shadow instead |
| `--color-bg-hover` | `#F1F3F5` | Hover states |
| `--color-bg-active` | `#E9ECEF` | Active/selected |
| `--color-border` | `#DEE2E6` | Visible borders |
| `--color-text-primary` | `#1A1D27` | Dark text (invert of dark bg) |
| `--color-text-secondary` | `#6C757D` | Secondary text (4.5:1 contrast min) |
| `--color-text-muted` | `#ADB5BD` | Muted/disabled |

Light mode accent colors remain the same but increase saturation slightly for visibility.

---

## 3. Typography

### System Fonts (No External Loading)

This is a native desktop app — use system fonts for instant rendering and native feel.

| Token | Value | Usage |
|-------|-------|-------|
| `--font-sans` | `-apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif` | All UI text |
| `--font-mono` | `'SF Mono', 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace` | Code, diffs, commit hashes, file paths |
| `--font-display` | `-apple-system, BlinkMacSystemFont, 'SF Pro Display', system-ui, sans-serif` | Large headings (20px+) |

### Type Scale

| Name | Size | Weight | Line Height | Usage |
|------|------|--------|-------------|-------|
| `heading-lg` | 20px | 600 | 1.3 | Panel titles, modal headers |
| `heading-md` | 16px | 600 | 1.4 | Section headers |
| `heading-sm` | 14px | 600 | 1.4 | Group labels, sidebar headers |
| `body` | 13px | 400 | 1.5 | Default UI text |
| `body-sm` | 12px | 400 | 1.5 | Secondary labels, timestamps |
| `caption` | 11px | 400 | 1.4 | Tooltips, badges, metadata |
| `mono` | 13px | 400 | 1.6 | Code, diffs, paths |
| `mono-sm` | 12px | 400 | 1.5 | Inline code, commit SHAs |
| `mono-xs` | 11px | 400 | 1.4 | Line numbers |

**Note:** 13px base (not 14px or 16px) — matches native macOS app density. Desktop apps use tighter type scales than websites.

---

## 4. Spacing & Layout

### Spacing Scale

| Token | Value | Usage |
|-------|-------|-------|
| `--space-1` | 2px | Inline gaps, icon-to-text |
| `--space-2` | 4px | Tight padding, badge padding |
| `--space-3` | 6px | Small component padding |
| `--space-4` | 8px | Default padding, list item padding |
| `--space-5` | 12px | Card padding, section gaps |
| `--space-6` | 16px | Panel padding, group gaps |
| `--space-8` | 24px | Section padding |
| `--space-10` | 32px | Major section gaps |

### Layout Regions

```
+--------------------------------------------------+
|  Title Bar (overlay, 38px, draggable)            |
+----------+----------------------+----------------+
| Sidebar  |   Main Content       |  Detail Panel  |
| (220px)  |   (flex-1)           |  (360px, opt)  |
|          |                      |                |
| Branches |   Commit Graph       |  Commit Info   |
| Files    |   File List          |  Diff View     |
| Search   |   Staging Area       |  Blame View    |
|          |                      |                |
+----------+----------------------+----------------+
|  Status Bar (24px)                               |
+--------------------------------------------------+
```

| Region | Width | Notes |
|--------|-------|-------|
| Title bar | 100%, 38px height | macOS overlay, traffic lights at left |
| Sidebar | 220px default, resizable (180-320px) | Collapsible with shortcut |
| Main content | flex-1 | Primary workspace |
| Detail panel | 360px default, resizable (280-600px) | Optional, togglable |
| Status bar | 100%, 24px height | Branch, sync status, background jobs |

### Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 4px | Buttons, badges, inputs |
| `--radius-md` | 6px | Cards, panels, dropdowns |
| `--radius-lg` | 8px | Modals, large containers |
| `--radius-full` | 9999px | Pills, avatars, toggle switches |

---

## 5. Elevation & Depth

### Shadows (Dark Theme)

| Level | Shadow | Usage |
|-------|--------|-------|
| `elevation-0` | none | Flat elements, inline |
| `elevation-1` | `0 1px 2px rgba(0,0,0,0.3)` | Cards, surface panels |
| `elevation-2` | `0 4px 12px rgba(0,0,0,0.4)` | Dropdowns, popovers |
| `elevation-3` | `0 8px 24px rgba(0,0,0,0.5)` | Modals, command palette |
| `elevation-4` | `0 16px 48px rgba(0,0,0,0.6)` | Full-screen overlays |

### Z-Index Scale

| Token | Value | Usage |
|-------|-------|-------|
| `--z-base` | 0 | Default content |
| `--z-sticky` | 10 | Sticky headers, sidebars |
| `--z-dropdown` | 20 | Dropdown menus |
| `--z-overlay` | 30 | Overlay panels |
| `--z-modal` | 40 | Modals, command palette |
| `--z-toast` | 50 | Notifications, toasts |

---

## 6. Interaction & Motion

### Transition Defaults

| Property | Duration | Easing | Usage |
|----------|----------|--------|-------|
| Color/opacity | 150ms | `ease-out` | Hover states, focus rings |
| Transform | 200ms | `cubic-bezier(0.4, 0, 0.2, 1)` | Panel collapse, accordion |
| Layout | 250ms | `cubic-bezier(0.4, 0, 0.2, 1)` | Sidebar resize, panel toggle |

### Functional Animations Only

| Animation | Duration | When |
|-----------|----------|------|
| Fade in | 150ms | Dropdown open, tooltip appear |
| Slide in | 200ms | Panel open, sidebar toggle |
| Skeleton pulse | 1.5s loop | Loading states |
| Spinner | 0.8s loop | Active operations (fetch, push) |
| Count-up | 300ms | Stat counters on load |

**Respect `prefers-reduced-motion`:** Replace all animations with instant state changes.

### Hover & Focus

| Element | Hover | Focus |
|---------|-------|-------|
| Button (primary) | Lighten accent 10% | 2px ring, `--color-accent`, 2px offset |
| Button (ghost) | `--color-bg-hover` background | 2px ring |
| List item | `--color-bg-hover` background | 2px ring inset |
| Link | Underline + lighten | 2px ring |
| Input | `--color-accent` border | 2px ring |

**All clickable elements must have `cursor: pointer`.**

---

## 7. Components

### Buttons

| Variant | Background | Text | Border | Usage |
|---------|-----------|------|--------|-------|
| Primary | `--color-accent` | white | none | Main CTA: Commit, Push |
| Secondary | `--color-bg-elevated` | `--color-text-primary` | `--color-border` | Cancel, secondary actions |
| Ghost | transparent | `--color-text-secondary` | none | Toolbar icons, toggle buttons |
| Danger | `--color-danger` | white | none | Force push, delete branch |

**Size:** 28px height (compact), 32px height (default), 36px height (prominent).
**Padding:** 8px 12px (default), 6px 8px (compact).

### Inputs

- Height: 32px
- Background: `--color-bg-surface`
- Border: `--color-border`, 1px solid
- Focus: `--color-accent` border, 2px ring with offset
- Placeholder: `--color-text-muted`
- Font: `--font-sans` 13px for text, `--font-mono` for code inputs

### Lists & Trees

- Row height: 28px (compact file lists), 32px (default), 40px (commit list)
- Selection: `--color-accent-muted` background + `--color-accent` left border (2px)
- Hover: `--color-bg-hover`
- Indentation: 16px per level (file trees)
- Icons: 16px, `--color-text-muted`, colored for file status

### Badges & Tags

- Height: 20px
- Padding: 2px 6px
- Font: 11px, weight 500
- Radius: `--radius-full`
- Branch badge: `--color-accent-muted` bg, `--color-accent` text
- Tag badge: `--color-warning-muted` bg, `--color-warning` text
- Remote badge: `--color-info` with 15% opacity bg

### Command Palette (CMD+K)

- Overlay: `--z-modal`, `rgba(0,0,0,0.5)` backdrop
- Container: 560px wide, `--color-bg-elevated`, `--radius-lg`, `elevation-3`
- Input: 48px height, no border, large text (15px)
- Results: max 8 visible, 36px per row
- Selected: `--color-bg-active` background
- Shortcut hints: right-aligned, `--font-mono` 11px, `--color-text-muted`

---

## 8. Icons

### Icon Set: Lucide

- **Library:** [Lucide](https://lucide.dev) (fork of Feather, actively maintained)
- **Size:** 16px (inline), 20px (toolbar), 24px (empty states)
- **Stroke:** 1.5px (matches macOS native icon weight)
- **Color:** `currentColor` (inherits from text color)

### Git-Specific Icons

| Concept | Icon Name | Notes |
|---------|-----------|-------|
| Branch | `git-branch` | |
| Commit | `git-commit-horizontal` | |
| Merge | `git-merge` | |
| Pull request | `git-pull-request` | |
| Tag | `tag` | |
| Stash | `archive` | |
| Staged | `check-circle-2` | Green |
| Modified | `circle-dot` | Blue |
| Untracked | `circle-plus` | Green |
| Deleted | `circle-minus` | Red |
| Conflict | `alert-triangle` | Yellow |
| Folder | `folder` / `folder-open` | |
| File | `file` / `file-code` | |

**Never use emoji as icons.** Always SVG from Lucide.

---

## 9. Accessibility

### Contrast Requirements

| Context | Minimum Ratio | Standard |
|---------|--------------|----------|
| Body text on background | 7:1 | WCAG AAA |
| Secondary text | 4.5:1 | WCAG AA |
| Large text (18px+) | 3:1 | WCAG AA |
| Interactive borders | 3:1 | WCAG AA |
| Focus indicators | 3:1 | WCAG AA |

### Keyboard Navigation

- **Tab order** matches visual order (left-to-right, top-to-bottom)
- **Focus ring:** 2px solid `--color-accent`, 2px offset
- **All modals:** trap focus, Escape to close
- **Command palette:** CMD+K opens, Escape closes, arrow keys navigate, Enter selects
- **Tree views:** Arrow keys for navigation, Enter to expand/select, Space to toggle

### Screen Reader

- All interactive elements have accessible names
- Status changes announced via `aria-live="polite"`
- File status uses text labels, not color alone
- Diff view has semantic structure (role="table" or equivalent)

---

## 10. Svelte 5 Implementation Notes

### State Management (Runes)

```svelte
<script lang="ts">
  // Use $state for reactive state
  let selectedFile = $state<string | null>(null);
  let isLoading = $state(false);

  // Use $derived for computed values
  let hasChanges = $derived(stagedFiles.length > 0);

  // Use $effect for side effects (e.g., CodeMirror)
  $effect(() => {
    if (editorContainer && currentDiff) {
      updateEditorContent(currentDiff);
    }
  });
</script>
```

### Scoped Styles

- Use `<style>` blocks (scoped by default in Svelte)
- Use `:global()` sparingly — only for third-party component overrides (CodeMirror)
- Reference CSS custom properties from `app.css` for consistent theming

### Component Patterns

- **Props:** Use `$props()` rune, not `export let`
- **Events:** Use callback props, not `createEventDispatcher` (Svelte 5)
- **Slots:** Use `{@render children()}` snippet pattern
- **Lifecycle:** Use `$effect` for mount/cleanup, not `onMount` when possible

### Tauri IPC Integration

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let status = $state<RepoStatus | null>(null);

  async function refreshStatus(path: string) {
    status = await invoke('get_repo_status', { path });
  }
</script>
```

---

## 11. Pre-Delivery Checklist

### Visual Quality
- [ ] No emoji icons — all SVG from Lucide
- [ ] Consistent icon sizing (16px inline, 20px toolbar)
- [ ] Hover states don't shift layout (color/opacity only)
- [ ] All colors reference CSS custom properties, not hardcoded hex

### Interaction
- [ ] All clickable elements have `cursor: pointer`
- [ ] Hover provides clear visual feedback (150ms transition)
- [ ] Focus rings visible on all interactive elements
- [ ] Keyboard shortcuts work and are discoverable

### Dark/Light Mode
- [ ] Text contrast meets WCAG AA minimum (4.5:1)
- [ ] Borders visible in both modes
- [ ] Semantic colors (success/danger/warning) work in both modes
- [ ] `prefers-color-scheme` media query applied

### Layout
- [ ] Content not hidden behind title bar (38px top padding)
- [ ] Resizable panels have min/max constraints
- [ ] No horizontal scroll at minimum window size (900x600)
- [ ] Sidebar collapsible without breaking layout

### Accessibility
- [ ] `prefers-reduced-motion` respected
- [ ] Focus trap in modals
- [ ] Screen reader labels on icon-only buttons
- [ ] Color is never the only status indicator (add icons/text)

---

## 12. File Organization

```
src/
  app.css                      # Design tokens (this system's variables)
  lib/
    components/
      ui/                      # Generic UI components
        Button.svelte
        Input.svelte
        Badge.svelte
        CommandPalette.svelte
      layout/                  # Layout shells
        Sidebar.svelte
        Panel.svelte
        StatusBar.svelte
        TitleBar.svelte
      git/                     # Git-specific components
        CommitGraph.svelte
        DiffView.svelte
        FileTree.svelte
        StagingArea.svelte
        BranchList.svelte
      editor/                  # CodeMirror wrappers
        DiffEditor.svelte
        MergeEditor.svelte
    stores/                    # Shared state (if needed beyond runes)
    utils/                     # Helpers, formatters
    types/                     # TypeScript interfaces
  routes/
    +page.svelte               # Main app shell
```
