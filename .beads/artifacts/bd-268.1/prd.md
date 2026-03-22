# Command Registry and Palette UI

**Bead:** bd-268.1  
**Created:** 2026-03-22  
**Status:** Draft

## Bead Metadata

```yaml
depends_on: []
parallel: true
conflicts_with: []
blocks: ["bd-268.2"]  # Core shortcut bindings depends on registry
estimated_hours: 4
```

---

## Problem Statement

### What problem are we solving?

The command palette exists as a working prototype but lacks the polish and completeness needed for a production Git client. Key gaps:

1. **Search is too simple** — `String.includes()` fails for partial/out-of-order matches (e.g., typing "ftch" won't find "Fetch from Remote")
2. **No command descriptions** — Users see only labels, not what commands actually do
3. **No recently-used tracking** — Power users repeat the same 5-6 commands; there's no fast path
4. **No error feedback** — When a git command fails, the palette closes silently with no indication
5. **Accessibility gaps** — Multiple a11y warnings suppressed (`a11y_no_static_element_interactions`, missing ARIA roles)
6. **No open/close animation** — Palette appears/disappears abruptly

### Why now?

This is the foundational infrastructure for all keyboard-driven interaction (bd-268). bd-268.2 (Core shortcut bindings) is blocked by this bead. Getting the registry and palette right now prevents rework when adding shortcuts later.

### Who is affected?

- **Primary users:** Solo power developers using keyboard-first workflows
- **Secondary users:** All mongit users who discover CMD+K

---

## Scope

### In-Scope

- Fuzzy search matching (character-level, not just substring)
- Command `description` field added to the Command interface
- Recently-used commands section (persisted to localStorage)
- Error toast/feedback when command execution fails
- Accessibility fixes (ARIA roles, focus trap, screen reader labels)
- Open/close CSS animation
- Expanded command set covering all existing Tauri IPC operations
- Command context enrichment (selected files, branch info)

### Out-of-Scope

- Actual keyboard shortcut bindings (bd-268.2)
- Custom user-defined commands or macros (V1.1+)
- Command-line argument passing (e.g., "create branch [name]") — future
- Plugin/extension command registration API (V2.0+)
- Search result ranking by usage frequency (nice-to-have, defer)

---

## Proposed Solution

### Overview

Harden the existing command palette into a production-quality feature by adding fuzzy search, command descriptions, recently-used tracking, error feedback, proper accessibility, and smooth animations. Expand the command set to cover all available git operations.

### User Flow

1. User presses **CMD+K** → palette opens with smooth animation, input focused
2. "Recently Used" section shows top 5 most-recent commands (if any)
3. User types a query → fuzzy-matched results appear grouped by category
4. User navigates with **Arrow Up/Down**, selects with **Enter**
5. Command executes → palette closes → success or error toast appears
6. User presses **Escape** or clicks backdrop → palette closes with animation

---

## Requirements

### Functional Requirements

#### FR-1: Fuzzy Search

Search must match characters in order but not necessarily contiguous, with match highlighting.

**Scenarios:**

- **WHEN** user types "ftch" **THEN** "Fetch from Remote" appears in results with "f", "t", "c", "h" highlighted
- **WHEN** user types "gb" **THEN** "Go to Branch" or "Git Branch" commands match
- **WHEN** query has no matches **THEN** empty state message shows "No commands matching '{query}'"
- **WHEN** query is empty **THEN** all enabled commands display, grouped by category

#### FR-2: Command Descriptions

Every command has a short description shown below the label in the palette.

**Scenarios:**

- **WHEN** a command has a description **THEN** it renders as secondary text below the label
- **WHEN** a command has no description **THEN** only the label renders (no empty space)

#### FR-3: Recently Used Commands

The palette tracks and displays recently executed commands.

**Scenarios:**

- **WHEN** user opens palette with empty query **THEN** "Recently Used" section appears above category groups (max 5 items)
- **WHEN** user executes a command **THEN** it moves to the top of the recently-used list
- **WHEN** user clears localStorage **THEN** recently-used list is empty (graceful degradation)
- **WHEN** a recently-used command is no longer enabled **THEN** it is hidden from the list

#### FR-4: Error Feedback

Command execution failures are surfaced to the user.

**Scenarios:**

- **WHEN** a command's `execute()` throws **THEN** an error toast appears with the error message
- **WHEN** a command succeeds **THEN** no toast (silent success for non-destructive ops)
- **WHEN** a git operation fails (e.g., push rejected) **THEN** toast shows the git error message

#### FR-5: Expanded Command Set

All existing Tauri IPC operations are available as commands.

**Scenarios:**

- **WHEN** user opens palette in a repo **THEN** commands for fetch, pull, push, create branch, stage, unstage, commit, and all navigation are available
- **WHEN** user is not in a repo **THEN** only "Open Repository" and navigation commands are enabled

#### FR-6: Enriched Command Context

CommandContext provides enough state for commands to make decisions.

**Scenarios:**

- **WHEN** a command checks context **THEN** it has access to `repoPath`, `currentRoute`, `hasRepo`, `currentBranch`, and `hasChanges`

### Non-Functional Requirements

- **Performance:** Palette opens in <50ms. Fuzzy search filters <16ms for 100 commands (single frame)
- **Accessibility:** WCAG 2.1 AA — proper ARIA roles (`dialog`, `combobox`, `listbox`, `option`), focus trap, screen reader announcements
- **Animation:** Open/close transition 150ms (matches `--transition-fast` token)

---

## Success Criteria

- [ ] Fuzzy search matches out-of-order characters (e.g., "ftch" → "Fetch from Remote")
  - Verify: `pnpm check` passes, manual test in `pnpm tauri dev`
- [ ] All commands have descriptions displayed in the palette
  - Verify: Visual inspection in palette UI
- [ ] Recently-used section appears with up to 5 items
  - Verify: Execute 3+ commands, reopen palette, see "Recently Used" section
- [ ] Error toast appears when a command fails
  - Verify: Trigger a git push on a repo with no remote, see error toast
- [ ] No a11y warnings suppressed in CommandPalette.svelte
  - Verify: `pnpm check` reports 0 a11y warnings for CommandPalette.svelte
- [ ] Smooth open/close animation
  - Verify: Visual inspection, 150ms transition
- [ ] `pnpm check` passes with 0 errors
  - Verify: `pnpm check`
- [ ] `cargo check` passes (if Rust changes needed)
  - Verify: `cd src-tauri && cargo check`

---

## Technical Context

### Existing Patterns

- **Command interface:** `src/lib/commands/types.ts` — `Command { id, label, category, shortcutHint?, enabled, execute }`
- **Registry:** `src/lib/commands/registry.svelte.ts` — Svelte 5 runes, Map-based, search returns grouped results
- **Palette UI:** `src/lib/components/CommandPalette.svelte` — 337 lines, modal with backdrop, keyboard navigation
- **Design tokens:** `src/app.css` — CSS variables for colors, spacing, typography, z-index, transitions
- **Store pattern:** `src/lib/stores/*.svelte.ts` — `$state()` + `$derived()` + plain functions

### Key Files

- `src/lib/commands/types.ts` — Add `description` field to Command interface
- `src/lib/commands/registry.svelte.ts` — Add fuzzy search, recently-used tracking
- `src/lib/commands/commands.ts` — Add descriptions and new commands
- `src/lib/components/CommandPalette.svelte` — UI refinements, a11y, animation
- `src/app.css` — Toast styles (if needed)

### Affected Files

```yaml
files:
  - src/lib/commands/types.ts          # Add description field, expand CommandContext
  - src/lib/commands/registry.svelte.ts # Fuzzy search, recently-used tracking
  - src/lib/commands/commands.ts        # Descriptions, new commands
  - src/lib/components/CommandPalette.svelte # UI polish, a11y, animation, error feedback
  - src/lib/components/Toast.svelte     # New: toast notification component
  - src/lib/stores/toast.svelte.ts      # New: toast state management
  - src/app.css                         # Animation keyframes if needed
```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Fuzzy search too slow for large command sets | Low | Medium | Benchmark at 100 commands; simple char-walk algorithm is O(n*m) |
| localStorage quota issues for recent commands | Low | Low | Store only command IDs (not full objects), cap at 20 entries |
| ARIA patterns break existing keyboard nav | Medium | Medium | Test with VoiceOver after changes; keep current keyboard behavior |

---

## Open Questions

| Question | Owner | Due Date | Status |
| --- | --- | --- | --- |
| Should fuzzy match highlight use bold or background color? | NamPT | During impl | Open |
| Should toast auto-dismiss or require manual close? | NamPT | During impl | Open — defaulting to auto-dismiss (3s) |

---

## Tasks

### T1: Add description field to Command types [types]

Command interface includes an optional `description: string` field, and CommandContext is enriched with `currentBranch` and `hasChanges`.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src/lib/commands/types.ts
```

**Verification:**

- `pnpm check` passes
- Command interface has `description?: string`
- CommandContext has `currentBranch` and `hasChanges`

### T2: Implement fuzzy search in registry [search]

Registry search uses character-walk fuzzy matching instead of `String.includes()`, returning match indices for highlighting.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: ["T3"]
files:
  - src/lib/commands/registry.svelte.ts
```

**Verification:**

- `pnpm check` passes
- Search for "ftch" returns "Fetch from Remote"
- Search for "gb" returns commands with "g" and "b" in sequence
- Empty query returns all enabled commands

### T3: Add recently-used tracking to registry [search]

Registry tracks command execution order in localStorage and exposes a `getRecent(ctx, limit)` method.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: ["T2"]
files:
  - src/lib/commands/registry.svelte.ts
```

**Verification:**

- `pnpm check` passes
- After executing commands, `getRecent()` returns them in MRU order
- Disabled commands are filtered from recent results
- localStorage key `mongit:recent-commands` stores command IDs

### T4: Create toast notification system [ui]

Toast component and store for displaying success/error notifications with auto-dismiss.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src/lib/components/Toast.svelte
  - src/lib/stores/toast.svelte.ts
  - src/routes/+layout.svelte
```

**Verification:**

- `pnpm check` passes
- Toast renders with message and variant (success/error)
- Toast auto-dismisses after 3 seconds
- Multiple toasts stack

### T5: Add descriptions and expand command set [commands]

All existing commands have descriptions, and new commands are added for all available Tauri IPC operations.

**Metadata:**

```yaml
depends_on: ["T1"]
parallel: false
conflicts_with: []
files:
  - src/lib/commands/commands.ts
```

**Verification:**

- `pnpm check` passes
- All commands have `description` field
- Commands exist for: fetch, pull, push, create branch, stage all, unstage all, toggle theme, open repo, all navigation routes

### T6: Polish palette UI — a11y, animation, descriptions, recents, error handling [ui]

CommandPalette.svelte updated with ARIA roles, open/close animation, description display, recently-used section, error handling with toast, and fuzzy match highlighting.

**Metadata:**

```yaml
depends_on: ["T1", "T2", "T3", "T4", "T5"]
parallel: false
conflicts_with: []
files:
  - src/lib/components/CommandPalette.svelte
```

**Verification:**

- `pnpm check` passes with 0 errors and 0 a11y warnings in CommandPalette.svelte
- Palette has `role="dialog"`, input has `role="combobox"`, results have `role="listbox"`
- Open/close uses CSS transition (150ms)
- Recently-used section visible when query is empty
- Command descriptions render below labels
- Error toast appears on command failure
- Fuzzy match characters are highlighted

---

## Notes

- The existing implementation (~337 lines in CommandPalette.svelte, ~99 lines in registry, ~133 lines in commands) is a solid foundation. This PRD is focused on hardening, not rewriting.
- bd-268.2 (Core shortcut bindings) will consume the `shortcutHint` field and add actual keybinding infrastructure. This bead only displays hints.
- Toast component is a general utility that will be reused across the app (not just command palette).
