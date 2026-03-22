# Core Shortcut Bindings

**Bead:** bd-268.2  
**Created:** 2026-03-22  
**Status:** Draft

## Bead Metadata

```yaml
depends_on: ["bd-268.1"]  # Command registry and palette UI
parallel: true
conflicts_with: []
blocks: []
estimated_hours: 3
```

---

## Problem Statement

### What problem are we solving?

The command palette (bd-268.1) provides a searchable interface to all commands, but every action requires opening the palette first. Power users need direct keyboard shortcuts for high-frequency operations — navigating between views, triggering git operations, and managing the workspace should be single-keystroke or two-key combos, not a palette search every time.

Currently:
1. **No shortcuts bound** — all 17 commands have `shortcutHint: undefined`
2. **No binding infrastructure** — shortcuts are hardcoded as ad-hoc `window.addEventListener` handlers
3. **No conflict management** — no system to prevent shortcuts from firing inside text inputs or CodeMirror editors

### Why now?

bd-268.1 established the command registry. Shortcuts are the natural next layer — they consume the `shortcutHint` field and the `execute()` pipeline that already exist. Parent bd-268 (Keyboard-First Command System) requires both palette and shortcuts to be complete.

### Who is affected?

- **Primary users:** Solo power developers who expect IDE-grade keyboard workflows
- **Secondary users:** All mongit users who see shortcut hints in the palette

---

## Scope

### In-Scope

- Shortcut binding engine that maps key combos to command IDs
- Shortcut definitions for core commands (navigation, git ops, view)
- `shortcutHint` values populated on all commands with bindings
- Conflict prevention: suppress shortcuts when focus is in text inputs or CodeMirror
- Shortcuts displayed in palette UI (already supported by `shortcutHint` field)

### Out-of-Scope

- User-customizable shortcut remapping (V1.1+)
- Tauri global shortcuts plugin (window-focused shortcuts are sufficient)
- Cross-platform key conventions (macOS-only for V1)
- Shortcut cheat sheet / help overlay (future)

---

## Proposed Solution

### Overview

Create a `ShortcutManager` module that registers key combos mapped to command IDs. On keydown, the manager checks whether the combo matches a binding, verifies the event target isn't a text input or CodeMirror editor, and executes the command through the existing registry. The manager replaces the ad-hoc CMD+K handler in CommandPalette.svelte.

### User Flow

1. User presses **CMD+1** → navigates to Summary view instantly
2. User presses **CMD+Shift+P** → push to remote executes immediately
3. User sees shortcut hints in the palette next to command labels
4. Shortcuts are suppressed when typing in input fields or CodeMirror

---

## Requirements

### Functional Requirements

#### FR-1: Shortcut Binding Engine

A centralized module manages shortcut-to-command mappings and dispatches key events.

**Scenarios:**

- **WHEN** user presses a bound key combo outside text inputs **THEN** the mapped command executes via `commandRegistry.execute()`
- **WHEN** user presses a bound key combo inside a text input or CodeMirror editor **THEN** the shortcut is suppressed (normal typing behavior)
- **WHEN** a command is disabled (e.g., no repo open) **THEN** its shortcut does nothing (no error toast)
- **WHEN** the command palette is open **THEN** global shortcuts are suppressed (palette owns keyboard)

#### FR-2: Core Shortcut Definitions

High-frequency commands have direct keyboard bindings.

**Bindings:**

| Shortcut | Command | Category |
| --- | --- | --- |
| `⌘K` | Toggle command palette | palette |
| `⌘1` | Go to Summary | navigation |
| `⌘2` | Go to Changes | navigation |
| `⌘3` | Go to History | navigation |
| `⌘⇧P` | Push to Remote | git |
| `⌘⇧F` | Fetch from Remote | git |
| `⌘⇧U` | Pull from Remote | git |
| `⌘⇧B` | Create Branch… | git |
| `⌘⇧R` | Refresh Repository Status | git |
| `⌘⇧T` | Toggle Dark/Light Theme | view |
| `⌘O` | Open Repository… | general |

**Scenarios:**

- **WHEN** user presses `⌘1` in a repo **THEN** navigates to `/repo` (Summary)
- **WHEN** user presses `⌘2` in a repo **THEN** navigates to `/repo/changes`
- **WHEN** user presses `⌘3` in a repo **THEN** navigates to `/repo/history`
- **WHEN** user presses `⌘⇧P` with no repo open **THEN** nothing happens (push requires repo)
- **WHEN** user presses `⌘O` **THEN** file picker opens regardless of repo state

#### FR-3: Shortcut Hints in Palette

All bound commands display their shortcut in the palette UI.

**Scenarios:**

- **WHEN** user opens palette **THEN** commands with shortcuts show the hint (e.g., "⌘1") on the right
- **WHEN** a command has no shortcut **THEN** no hint is displayed (existing behavior)

### Non-Functional Requirements

- **Performance:** Keydown handler must resolve in <1ms (simple Map lookup)
- **Accessibility:** Shortcuts must not conflict with system accessibility shortcuts (VoiceOver uses Ctrl+Option)
- **Maintainability:** Adding a new shortcut requires only adding one entry to a declaration array

---

## Success Criteria

- [ ] All 11 shortcuts listed above work correctly
  - Verify: Manual test in `pnpm tauri dev`
- [ ] Shortcuts are suppressed in text inputs and CodeMirror editors
  - Verify: Type "1" in search input — no navigation
- [ ] Shortcut hints display in palette for all bound commands
  - Verify: Open CMD+K, see hints next to commands
- [ ] No shortcut fires when palette is open
  - Verify: Open palette, press CMD+1 — no navigation
- [ ] `pnpm check` passes with 0 errors
  - Verify: `pnpm check`

---

## Technical Context

### Existing Patterns

- **Command registry:** `src/lib/commands/registry.svelte.ts` — `execute(id, ctx)` returns `Promise<boolean>`
- **Command types:** `src/lib/commands/types.ts` — `shortcutHint?: string` already exists
- **Palette keyboard:** `src/lib/components/CommandPalette.svelte` — ad-hoc CMD+K handler
- **Store pattern:** Svelte 5 runes (`$state`, `$derived`), plain functions

### Key Files

- `src/lib/commands/types.ts` — shortcutHint field (already exists)
- `src/lib/commands/commands.ts` — add shortcutHint values to command definitions
- `src/lib/commands/registry.svelte.ts` — may need minor extensions
- `src/lib/components/CommandPalette.svelte` — migrate CMD+K to shortcut manager
- `src/routes/+layout.svelte` — mount shortcut manager at app level

### Affected Files

```yaml
files:
  - src/lib/commands/shortcuts.ts              # New: shortcut binding engine
  - src/lib/commands/commands.ts               # Add shortcutHint values
  - src/lib/components/CommandPalette.svelte   # Remove ad-hoc CMD+K handler, use shortcut manager
  - src/routes/+layout.svelte                  # Mount shortcut manager
```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Shortcuts conflict with browser/system defaults | Medium | High | Use Shift-modified combos for git ops; test on macOS |
| CodeMirror intercepts shortcuts before our handler | Low | Medium | Check `.cm-editor` ancestor in handler (existing pattern) |
| Shortcuts fire during palette interaction | Low | Medium | Check palette open state before executing |

---

## Open Questions

| Question | Owner | Due Date | Status |
| --- | --- | --- | --- |
| Should ⌘N create a new branch (conflicts with system "new window")? | NamPT | During impl | Open — using ⌘⇧B instead |

---

## Tasks

### T1: Create shortcut binding engine [infrastructure]

A `ShortcutManager` module that maps key combos to command IDs, handles keydown events, and suppresses shortcuts in text inputs/CodeMirror.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src/lib/commands/shortcuts.ts
```

**Verification:**

- `pnpm check` passes
- Module exports `registerShortcuts()` and `destroyShortcuts()`

### T2: Add shortcutHint values and bind shortcuts [commands]

Populate `shortcutHint` on all commands that get keyboard bindings, and define the shortcut-to-command mapping.

**Metadata:**

```yaml
depends_on: ["T1"]
parallel: false
conflicts_with: []
files:
  - src/lib/commands/commands.ts
  - src/lib/commands/shortcuts.ts
```

**Verification:**

- `pnpm check` passes
- All 11 shortcuts defined in mapping
- All corresponding commands have `shortcutHint` values

### T3: Migrate CMD+K and mount shortcut manager [integration]

Move CMD+K handling from CommandPalette.svelte into the shortcut manager. Mount the manager in +layout.svelte. Ensure palette suppresses shortcuts when open.

**Metadata:**

```yaml
depends_on: ["T1", "T2"]
parallel: false
conflicts_with: []
files:
  - src/lib/components/CommandPalette.svelte
  - src/routes/+layout.svelte
```

**Verification:**

- `pnpm check` passes with 0 errors
- CMD+K still opens/closes palette
- No a11y regressions in CommandPalette
- Shortcuts suppressed when palette is open
- Shortcuts suppressed in text inputs and CodeMirror

---

## Notes

- The `shortcutHint` field on Command already exists from bd-268.1. This bead only populates it and adds the binding engine.
- macOS-only for V1: all shortcuts use `metaKey` (CMD). Cross-platform mapping deferred to Windows support.
- The palette already renders `shortcutHint` via `<kbd>` elements — no palette UI changes needed.
