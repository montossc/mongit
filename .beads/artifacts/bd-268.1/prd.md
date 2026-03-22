# Command Registry and Palette UI

**Bead:** bd-268.1
**Type:** task
**Status:** Refined
**Parent:** bd-268 (Keyboard-First Command System)
**Blocks:** bd-268.2 (Core shortcut bindings)

---

## Problem Statement

mongit has no centralized way to discover and invoke actions. Every operation requires navigating to the correct page and clicking the right button. Power users (our primary audience) expect a CMD+K command palette — the universal pattern in VS Code, Linear, Slack, and GitHub. Without it, mongit fails the "keyboard-first" core principle.

## Scope

### In-Scope

1. **Command Registry** — A typed, extensible registry that defines all available actions with metadata (id, label, category, shortcut hint, availability predicate)
2. **Command Palette UI** — A modal overlay with search input, filtered command list, keyboard navigation, and execution
3. **Global Keyboard Listener** — CMD+K (macOS) opens the palette from anywhere in the app
4. **Initial Command Set** — Register all existing actions as commands (branch ops, navigation, staging, theme toggle)
5. **Root Layout Integration** — Mount the palette at the root layout level so it's accessible from any route

### Out-of-Scope

- Per-command shortcut bindings (bd-268.2)
- Custom user-defined shortcuts
- Command history / recent commands
- Fuzzy matching beyond simple substring
- Context menus or right-click integration

## Proposed Solution

### Architecture

```
src/lib/commands/
├── registry.ts          # Command registry: register, lookup, execute
├── types.ts             # Command, CommandCategory, CommandContext types
└── commands.ts          # Initial command definitions

src/lib/components/
└── CommandPalette.svelte  # Modal overlay UI

src/routes/
└── +layout.svelte       # Mount CommandPalette at root level
```

### Command Registry Design

- Singleton store (Svelte 5 runes) holding a `Map<string, Command>` 
- Each command: `{ id, label, category, shortcutHint?, icon?, enabled: (ctx) => boolean, execute: (ctx) => void | Promise<void> }`
- Categories: `navigation`, `git`, `staging`, `view`, `general`
- Context object provides: `repoPath`, `currentRoute`, `hasRepo`
- Commands register at module load time (static registration)

### Palette UI Design

- Triggered by CMD+K (Meta+K on macOS)
- Escape or clicking backdrop closes
- Search input filters commands by label (case-insensitive substring match)
- Arrow Up/Down navigates, Enter executes, results grouped by category
- Shows shortcut hint badges when available
- Uses existing design tokens: `--z-modal` for z-index, `--color-bg-elevated` for panel, `--elevation-3` for shadow

## Requirements

### Functional

1. **FR-1:** A `CommandRegistry` store exists that can register, deregister, and look up commands by ID
2. **FR-2:** Commands have typed metadata: `id`, `label`, `category`, `shortcutHint?`, `enabled(ctx)`, `execute(ctx)`
3. **FR-3:** CMD+K opens the command palette from any route in the app
4. **FR-4:** The palette shows all enabled commands, filtered by search input
5. **FR-5:** Arrow keys navigate the filtered list; Enter executes the highlighted command
6. **FR-6:** Escape or backdrop click closes the palette
7. **FR-7:** Commands are grouped by category in the results list
8. **FR-8:** Initial command set includes: navigate to Summary, navigate to Changes, create branch, switch branch, fetch, pull, push, toggle theme, open repo
9. **FR-9:** Commands with `enabled()` returning false are hidden from the palette

### Non-Functional

1. **NFR-1:** Palette opens in < 100ms (no lazy loading delay)
2. **NFR-2:** Search filtering is synchronous (no debounce needed for < 100 commands)
3. **NFR-3:** Full keyboard accessibility (tab trap inside modal, focus returns on close)
4. **NFR-4:** Respects system dark/light theme via existing design tokens

## Success Criteria

- [ ] CMD+K opens a centered modal overlay with search input
- [ ] Typing filters commands by label (case-insensitive)
- [ ] Arrow keys navigate, Enter executes, Escape closes
- [ ] At least 9 commands registered covering navigation, git ops, and view actions
- [ ] Palette is accessible from every route (mounted at root layout)
- [ ] `pnpm check` passes with 0 errors
- [ ] Commands execute their intended actions (navigate, invoke IPC, toggle state)

## Technical Context

### Existing Infrastructure

- **Stores:** `repoStore` (active repo path, status), `changesStore`, `diffStore`, `themeStore` (dark/light toggle), `watcherStore`
- **IPC Commands:** `create_branch`, `switch_branch`, `delete_branch`, `fetch`, `pull`, `push`, `open_repo`, `get_repo_status`
- **Design Tokens:** Full set available including `--z-modal: 40`, `--z-overlay: 30`, `--elevation-3`, `--transition-fast`
- **Root Layout:** Currently minimal (just CSS import + children render) — needs CommandPalette added
- **Keyboard Handling:** Greenfield — no existing keyboard listeners anywhere

### Key Constraints

- Palette must not interfere with CodeMirror keyboard shortcuts (CM6 captures keys when focused)
- Use `window.addEventListener('keydown', ...)` for global CMD+K, not Svelte `on:keydown`
- Must handle cmd+k vs ctrl+k (macOS only for V1, so just Meta+K)
- Modal focus trap: when open, Tab should cycle within palette, not escape to underlying page

## Affected Files

### New Files
- `src/lib/commands/types.ts` — Command types and interfaces
- `src/lib/commands/registry.ts` — Command registry store
- `src/lib/commands/commands.ts` — Initial command definitions  
- `src/lib/components/CommandPalette.svelte` — Palette UI component

### Modified Files
- `src/routes/+layout.svelte` — Mount CommandPalette

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| CMD+K conflicts with browser shortcuts | Low | Tauri desktop app, no browser shortcuts |
| CodeMirror captures keyboard events | Medium | Check `event.target` — skip if inside `.cm-editor` |
| Focus management on open/close | Medium | Use `requestAnimationFrame` for reliable focus |

## Open Questions

None — scope is well-defined for a first implementation.

---

## Metadata

**Parent:** bd-268
**Blocks:** bd-268.2
