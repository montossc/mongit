# PRD: Repo Home and Navigation Shell

**Bead:** bd-6ew
**Parent:** bd-7gm (MVP Core Git Client)
**Depends on:** bd-1zx (Design System Tokens Baseline) — CLOSED
**Type:** feature
**Priority:** P0

```yaml
requirements_score:
  total: 94
  breakdown:
    business_value: 28
    functional_requirements: 24
    user_experience: 18
    technical_constraints: 14
    scope_and_priorities: 10
  status: passed
  rounds_used: 2
  deferred_questions: 0
```

---

## Problem Statement

mongit currently has backend Git capabilities, theme infrastructure, and reusable UI primitives, but it does not yet have a real product entry surface. The app still behaves like a spike/demo rather than a durable desktop Git client. Users need a practical way to open a local repository, return to recent repositories, and immediately understand repository state without hunting through the interface.

This feature creates the first real application shell: a dedicated repo home surface plus a repo workspace shell that downstream features can plug into.

## Scope

### In-Scope

- Open a local repository through:
  - native file picker
  - pasted/manual path entry
  - drag-and-drop onto the home surface
- Persist recent repositories in **Tauri app data** (not localStorage)
- Show recent repositories on the home screen
- Represent stale/missing repository paths visibly and allow users to remove or retry them
- Split app structure into:
  - **home route** (`/`)
  - **repo workspace shell route** (`/repo`)
- Show immediate repo orientation info after open:
  - current branch
  - changed files count
  - staged files count
  - repo path/name
- Establish a repo context store for frontend state and shell routing

### Out-of-Scope

- Branch switcher UX (separate bead)
- Clone/init repository flows
- Changes workspace (bd-20d)
- Commit graph productization beyond shell handoff (bd-145)
- Command palette integration (bd-268)
- Advanced remote/ahead-behind sync indicators
- Multi-window or multi-repo simultaneous workspaces

## Proposed Solution

Split the current single-page spike structure into two product surfaces:

1. **Home (`/`)**
   - Open repository actions (picker, paste, drag-drop)
   - Recent repos list with click-to-open
   - Empty-state guidance for first-time users
   - Drag/drop target for the entire home surface
   - Retry/remove stale repo entries

2. **Repo workspace shell (`/repo`)**
   - Persistent top-level repo context header
   - Immediate summary surface: current branch, changed/staged file counts, repo identity
   - Handoff point for future graph, changes, and authoring views as nested routes

Use a dedicated `repo.svelte.ts` store as the frontend source of truth for:
- current repo path
- recent repos list
- current repo status (branch, changed, staged)
- loading/error state transitions

Use Rust/Tauri commands to:
- validate/open repo paths
- load/save recent repos in app data directory
- fetch repo status via existing `get_repo_status`

## Requirements

### R1: Repo Open Flow

User can open a local repository from the home screen via:
- Native file picker dialog
- Pasted/typed path with submit
- Drag-and-drop a folder onto the home surface

**Acceptance:**
- Invalid paths are rejected with clear error messaging
- Non-git directories are rejected with a specific "not a git repository" error
- Successful open transitions user into repo workspace shell
- All three entry methods produce the same result

**Affected files:** `src/routes/+page.svelte`, `src/lib/stores/repo.svelte.ts`

### R2: Recent Repos Persistence

Recent repos are persisted in **Tauri app data** directory, not localStorage.

**Acceptance:**
- Recent repos list survives app restarts and browser cache clears
- Each recent item stores:
  - `path` (absolute filesystem path)
  - `name` (display name, derived from directory name)
  - `lastAccessed` (timestamp, updated on each open)
  - `valid` (boolean, checked on home load)
- Recent repos list is capped (10-20 entries, LRU eviction)
- Recent repos can be reopened from home with a single click

**Affected files:** `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src/lib/stores/repo.svelte.ts`

### R3: Stale Repo Handling

If a recent repo path is missing or invalid:
- It remains visible in the recent list (not silently removed)
- It is visually marked as stale/invalid (muted styling, warning indicator)
- User can retry opening it (in case drive was remounted, etc.)
- User can remove it from the recent list

**Affected files:** `src/routes/+page.svelte`, `src/lib/stores/repo.svelte.ts`, `src-tauri/src/commands.rs`

### R4: Home and Workspace Route Split

The app shell is split so home and active repo workspace are distinct product surfaces.

**Acceptance:**
- Home route (`/`) is no longer coupled to spike/demo controls
- Repo workspace has a dedicated route with its own layout shell (`/repo`)
- Current spike content is preserved in a spike-specific route (e.g., `/spike-b`)
- Repo workspace shell can host future nested routes (graph, changes, history)
- Navigating back to home from workspace is possible

**Affected files:** `src/routes/+page.svelte`, `src/routes/repo/+layout.svelte` (new), `src/routes/repo/+page.svelte` (new)

### R5: Immediate Repo State Summary

When a repo is opened, the workspace shell immediately exposes:
- Current branch name
- Changed (unstaged) files count
- Staged files count
- Repository identity (path and/or display name)

This information must be visible without any additional user action after opening a repo.

**Affected files:** `src/routes/repo/+layout.svelte`, `src/lib/stores/repo.svelte.ts`

### R6: Frontend Repo Context Store

Create a runes-based repo store that manages:
- `currentRepoPath` — the active repo (null when on home)
- `recentRepos` — list from Tauri app data
- `repoStatus` — branch, changed, staged counts
- `openRepo(path)` — validate, load status, add to recents, navigate
- Loading and error state transitions

**Affected files:** `src/lib/stores/repo.svelte.ts` (new)

### R7: Design System Compliance

The home and workspace shell must use existing tokens and UI primitives from:
- `src/app.css` (design tokens)
- `src/lib/components/ui/` (Button, Input, Badge, Panel)

No hardcoded hex colors or pixel sizes. All interactive elements must have focus rings and cursor:pointer per MASTER.md.

**Affected files:** All new UI files

## Success Criteria

1. User can open a repo from picker, pasted path, or drag-drop
2. Recent repos persist across app restarts via Tauri app data
3. Invalid recent repos remain visible and can be removed or retried
4. App has separate home (`/`) and repo workspace (`/repo`) shell structure
5. Opening a repo immediately shows branch + changed/staged file counts
6. Existing verification passes:
   - `pnpm check` — 0 errors
   - `pnpm build` — succeeds
   - `cargo check` — succeeds

## Verify

```bash
pnpm check    # 0 errors (svelte-check)
pnpm build    # Vite build succeeds
cargo check   # Rust typecheck succeeds (in src-tauri/)
```

Manual verification:
- Open app → home surface shows recent repos (or empty state)
- Open a repo via each method → transitions to workspace shell with status
- Close and reopen app → recent repos are preserved
- Add invalid path to recents → shows stale indicator, can remove

## Technical Context

### Relevant existing code

- `src/routes/+page.svelte` (471 lines) — currently mixes spike/demo behavior with repo-entry behavior; should be split
- `src/routes/+layout.svelte` — minimal global layout, imports `app.css`
- `src/routes/+layout.ts` — `ssr = false`, `prerender = true`
- `src/lib/stores/theme.svelte.ts` — demonstrates runes + persistence pattern
- `src/lib/stores/watcher.svelte.ts` — demonstrates Tauri invoke/store pattern
- `src-tauri/src/commands.rs` — already exposes `get_repo_status(path)`, branch ops
- `src-tauri/src/lib.rs` — registers commands in `generate_handler![]`
- `src/lib/components/ui/` — Button, Input, Badge, Panel primitives ready for reuse

### Constraints

- SvelteKit runs with `ssr=false`; browser-only APIs are acceptable with guards
- New Tauri plugins/dependencies require asking user first
- Recent repos persistence must survive browser cache clears → use Tauri app data
- `tauri.conf.json` currently only enables `shell` plugin; file picker dialog may need `dialog` plugin (ask first)
- Repo opening UX must handle stale paths safely (no crashes on missing dirs)

## Affected Files

| File | Action | Description |
|------|--------|-------------|
| `src/routes/+page.svelte` | Modify/replace | Convert into product home surface |
| `src/routes/repo/+layout.svelte` | Create | Persistent repo workspace shell layout |
| `src/routes/repo/+page.svelte` | Create | Default repo workspace landing surface |
| `src/lib/stores/repo.svelte.ts` | Create | Current repo + recent repos + status store |
| `src-tauri/src/commands.rs` | Modify | Add recent repo persistence + path validation commands |
| `src-tauri/src/lib.rs` | Modify | Register new commands |
| `src/app.css` | Maybe modify | Minimal shell-specific styling if tokens are insufficient |

## Tasks

### Repo Persistence and Validation Backend [backend]

Add backend support for:
- Validating a path is an openable git repository
- Saving recent repos list to Tauri app data directory (JSON)
- Loading recent repos list on startup
- Removing a repo from the recent list

**Verification:**
- `cargo check` passes
- `cargo test` passes for new persistence logic
- Invalid path returns structured error, not panic

**Metadata:**
```yaml
depends_on: []
parallel: true
files:
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
```

### Repo Context Store [frontend]

Create runes-based repo store managing current repo, recents, status, and open action.

**Verification:**
- Store exposes currentRepoPath, recentRepos, repoStatus, openRepo, loading, error
- Store integrates with backend commands via invoke()
- `pnpm check` passes

**Metadata:**
```yaml
depends_on: [backend]
parallel: false
files:
  - src/lib/stores/repo.svelte.ts
```

### Home Surface [ui]

Create the repo home page with:
- Open repository button (triggers file picker)
- Manual path input with submit
- Drag/drop affordance
- Recent repos list
- Stale repo visual states
- Empty state for first-time users

**Verification:**
- Home route renders without spike-only controls
- User can trigger open from all 3 entry paths
- Stale recent repos are visibly differentiated
- Uses design system tokens and UI primitives
- `pnpm check` passes

**Metadata:**
```yaml
depends_on: [store]
parallel: false
files:
  - src/routes/+page.svelte
```

### Workspace Shell [ui]

Create repo workspace layout with:
- Persistent repo context header showing branch + changed/staged counts + repo identity
- Outlet for future nested routes (graph, changes, history)
- Default landing surface

**Verification:**
- Opening a repo routes into workspace shell
- Workspace shows branch + changed/staged counts + repo identity
- `pnpm build` succeeds

**Metadata:**
```yaml
depends_on: [store]
parallel: true
files:
  - src/routes/repo/+layout.svelte
  - src/routes/repo/+page.svelte
```

### Barrel Export [cleanup]

Re-export barrel for any new components created. Preserve current spike in `/spike-b` if needed.

**Verification:**
- `pnpm check` passes
- `pnpm build` succeeds

**Metadata:**
```yaml
depends_on: [home, workspace]
parallel: false
files:
  - src/routes/spike-b/+page.svelte
```

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| File picker requires new Tauri dialog plugin/dependency | Medium | Ask user before adding; keep pasted path as fallback |
| Current `+page.svelte` is spike-heavy and hard to evolve cleanly | Medium | Split home/workspace instead of extending monolith |
| Recent repo entries become stale when directories move | Low | Keep visible with retry/remove actions |
| Scope creep into branch switching / clone/init | Medium | Keep bd-6ew limited to entry flow + shell + immediate summary |
| Drag-drop may need CSP or Tauri permission changes | Low | Test early; fall back to picker + paste if blocked |

## Open Questions

None — all questions resolved during refinement.

---

## Metadata

**Parent:** bd-7gm
**Depends on:** bd-1zx (closed)
**Children:** bd-6ew.1 (repo selection/persistence), bd-6ew.2 (home state summary widgets)
