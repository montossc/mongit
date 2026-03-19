# PRD: Repo selection and persistence

**Bead:** bd-6ew.1
**Parent:** bd-6ew (Repo Home and Navigation Shell)
**Depends on:** bd-6ew (parent-child)
**Type:** task
**Priority:** P0

```yaml
requirements_score:
  total: 93
  breakdown:
    business_value: 27
    functional_requirements: 24
    user_experience: 17
    technical_constraints: 15
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

mongit has backend Git capabilities and a graph spike, but its current app entry point is still a spike surface rather than a durable product workflow. Users need a reliable way to choose a local repository, reopen recently used repositories across app restarts, and recover gracefully when a remembered path becomes stale.

This child bead defines the repository entry contract for the product: getting a valid repository into application state, persisting it in desktop-owned storage, and handing off into the repo workspace route. It intentionally stops short of building the orientation/dashboard widgets owned by `bd-6ew.2`.

## Scope

### In-Scope

- Open a local repository from the home surface via:
  - native folder picker
  - pasted or typed absolute path
  - drag-and-drop of a directory onto the home surface
- Validate that the selected path is an openable Git repository
- Persist recent repositories in **Tauri app data**, not `localStorage`
- Store recent repositories as a 10-item LRU list
- Keep stale recent repositories visible with retry and remove affordances
- Establish frontend repo state sufficient to:
  - track the active repo path
  - load repo status after successful open
  - navigate into `/repo`
- Replace the current root spike controls with a product home surface for repo entry
- Create the `/repo` route shell and default route as the navigation handoff target

### Out-of-Scope

- Summary widgets or dashboard-style orientation surfaces inside the repo workspace (`bd-6ew.2`)
- Branch switcher UX
- Clone/init flows
- Commit graph productization beyond preserving a route handoff target
- Command palette integration
- Multi-window or multi-repo workspace support
- Ahead/behind, remote sync, or richer repo-health indicators

## Proposed Solution

Split the work into a product home route and a repo workspace handoff route.

1. **Home route (`/`)**
   - Replace spike-first controls in `src/routes/+page.svelte` with repo-entry UI.
   - Support three equivalent repo open entry methods: folder picker, manual path entry, and drag-drop.
   - Show the recent repositories list from Tauri app data with stale-entry states.

2. **Repo state layer**
   - Create a runes-based store as the frontend source of truth for the active repository, recent repos, status-loading lifecycle, and open/remove/retry actions.
   - Reuse existing Tauri invoke patterns and existing `get_repo_status` for status hydration after open.

3. **Backend persistence and validation**
   - Add Tauri commands to validate/open repo paths, load/save recent repos, and remove recent repo entries.
   - Persist recents in app data so data survives browser cache clears.

4. **Workspace route handoff (`/repo`)**
   - Create the route shell and default page needed for a successful transition after opening a repo.
   - Keep this bead limited to the route boundary and state handoff; `bd-6ew.2` will add the summary widgets visible after landing.

### User Flow

1. User opens mongit and lands on `/`, which presents repo-entry actions and recent repos.
2. User opens a repository via picker, manual path, drag-drop, or by clicking a recent repo.
3. App validates the path, hydrates repo state, updates the 10-item LRU recents list, and navigates to `/repo`.
4. If a recent repo is stale, the user can see that state, retry opening it, or remove it from recents.

## Requirements

### Functional Requirements

#### R1. Repository open methods

Users must be able to open a local repository using all three entry methods: native folder picker, manual path entry, and drag-drop.

**Scenarios:**
- **WHEN** the user selects a valid Git repository with any supported entry method **THEN** mongit opens that repository using the same validation and state-loading flow.
- **WHEN** the user submits an invalid path or a non-Git directory **THEN** mongit shows a specific error and does not navigate to `/repo`.
- **WHEN** native folder picker support requires the Tauri dialog plugin **THEN** this bead still treats picker support as required MVP scope and documents the plugin approval as an implementation prerequisite rather than dropping the feature.

#### R2. Recent repository persistence

Recent repositories must persist in desktop-owned storage and survive app restarts.

**Scenarios:**
- **WHEN** the user opens a repository successfully **THEN** mongit writes or updates a recent-repo entry in Tauri app data.
- **WHEN** the user reopens an existing recent repository **THEN** its `lastAccessed` value updates and the item moves to the top of the recents list.
- **WHEN** the recents list exceeds 10 items **THEN** mongit evicts the least recently used item.

Recent repo entries must contain:
- `path` — absolute filesystem path
- `name` — display name derived from directory name
- `lastAccessed` — timestamp updated on successful open
- `valid` — current validity state used by the home UI

#### R3. Stale recent repository handling

Stale entries must remain visible and recoverable.

**Scenarios:**
- **WHEN** mongit loads recents and a path no longer resolves to a valid Git repository **THEN** the entry remains visible with a stale/invalid state.
- **WHEN** the user retries a stale entry and the path becomes valid again **THEN** the repo opens successfully and its state updates.
- **WHEN** the user removes a stale entry **THEN** the entry is deleted from persistence and from the displayed recents list.

#### R4. Frontend repo state handoff

The repo store must own the repo-entry lifecycle for this slice.

**Scenarios:**
- **WHEN** a repo opens successfully **THEN** the store updates the active repo path, hydrates repo status, updates recents, and navigates to `/repo`.
- **WHEN** repo open is in flight **THEN** the store exposes loading state for the home UI.
- **WHEN** repo open fails **THEN** the store exposes error state without mutating the active repo selection incorrectly.

#### R5. Route split for product entry

The app must no longer treat `/` as a spike-only surface.

**Scenarios:**
- **WHEN** the app loads **THEN** `/` renders repo-entry UI and recent repos instead of the current graph spike controls.
- **WHEN** a repo opens successfully **THEN** the app navigates to `/repo`.
- **WHEN** the repo workspace route is created **THEN** it provides a stable shell/default page for downstream work, even if summary widgets are added in `bd-6ew.2` later.

### Non-Functional Requirements

- **Performance:** Recent repos loading and validation should feel immediate on app open; avoid blocking the UI longer than necessary for normal local repositories.
- **Security:** Only local filesystem paths are handled; invalid paths must fail safely without panic or undefined state.
- **Compatibility:** Must work within Tauri 2.0 + SvelteKit with `ssr=false` and existing app architecture.
- **Storage:** Recent repos persistence must use Tauri app data, not browser-managed storage.
- **Design system:** New UI must use existing tokens and primitives from `src/app.css` and `src/lib/components/ui/`.

## Success Criteria

- [ ] User can open a repository from the home surface via native picker, manual path, and drag-drop.
  - Verify: manual app check in Tauri build/dev session
- [ ] Successful open updates the active repo state, writes/updates a recent repo entry, and navigates to `/repo`.
  - Verify: manual app check in Tauri build/dev session
- [ ] Recent repositories persist across app restart and are capped at 10 entries with LRU behavior.
  - Verify: manual app restart check + targeted backend test coverage
- [ ] Stale recent repos remain visible and support retry/remove actions.
  - Verify: manual app check with renamed/missing repo path
- [ ] Project verification remains green after implementation.
  - Verify: `pnpm check`
  - Verify: `pnpm build`
  - Verify: `cargo check` (run in `src-tauri/`)

## Technical Context

### Existing Patterns

- `src/lib/stores/theme.svelte.ts:10-79` — local persistence pattern with a storage key, guarded reads, and setter-based sync.
- `src/lib/stores/watcher.svelte.ts:9-70` — runes-based store factory plus `invoke()` error/loading handling.
- `src/routes/+page.svelte:32-117` — current async Tauri-driven repo load pattern on the root route.
- `src-tauri/src/commands.rs:27-178` — async command pattern using `tokio::task::spawn_blocking`, `PathBuf`, and `Result<_, String>`.
- `src-tauri/src/lib.rs:8-29` — command registration location via `tauri::generate_handler![]`.
- `src/lib/components/ui/` — available primitives: `Button.svelte`, `Input.svelte`, `Badge.svelte`, `Panel.svelte`.

### Key Constraints

- `src/routes/+layout.ts:1-3` disables SSR, so browser-only interactions are acceptable with client guards.
- `src/routes/+page.svelte` is currently spike-oriented and large, so root-route replacement should stay focused on entry-flow concerns only.
- `src/routes/repo` does not exist yet; this bead creates the route boundary required by the parent feature.
- Native folder picker may require the Tauri dialog plugin; per project rules, adding a new dependency/plugin requires explicit user approval during implementation.
- `get_repo_status(path)` already exists in `src-tauri/src/commands.rs:27-43` and should be reused rather than duplicated.
- No TODO/FIXME markers were found in `src/` for this domain during refinement.

### Affected Files

```yaml
files:
  - src/routes/+page.svelte # Replace spike-first root surface with repo-entry home UI
  - src/routes/repo/+layout.svelte # New workspace shell route boundary and shared repo handoff frame
  - src/routes/repo/+page.svelte # New default repo landing page / placeholder handoff surface
  - src/lib/stores/repo.svelte.ts # New runes-based repo entry, recents, and navigation state store
  - src-tauri/src/commands.rs # Add repo validation and recent-repo persistence commands
  - src-tauri/src/lib.rs # Register new Tauri commands
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Native picker requires new dialog plugin | Medium | Medium | Keep picker in scope, but treat plugin approval as an implementation prerequisite and preserve manual path + drag-drop parity. |
| Root route refactor accidentally absorbs summary-widget work from `bd-6ew.2` | Medium | Medium | Limit this bead to entry flow, persistence, and route handoff; defer dashboard/orientation surfaces to the sibling bead. |
| Stale repo validation causes confusing silent removal | Low | Medium | Keep invalid entries visible with explicit retry/remove actions and status markers. |
| Persistence format or path handling diverges across frontend/backend | Low | Medium | Define one recent-repo record shape and use backend-owned read/write commands as the source of truth. |
| Scope creep into clone/init or branch UX | Medium | Medium | Keep requirements explicitly focused on opening existing local repos and recents management only. |

## Open Questions

None. The remaining implementation-time prerequisite is explicit approval if adding the Tauri dialog plugin becomes necessary for native folder picker support.

## Tasks

### Backend repo validation and recents persistence [backend]

Add Tauri commands that validate repository paths and manage recent-repo records in app data using a single backend-owned schema.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
```

**Verification:**

- `cargo check`
- Backend tests cover valid repo, invalid path, stale repo, recents update, and 10-item LRU eviction behavior
- Successful open can reuse `get_repo_status` without duplicating repo-status logic

### Frontend repo store and open lifecycle [frontend]

Create a runes-based repo store that owns active repo state, recent repos, open/remove/retry actions, and navigation handoff into `/repo`.

**Metadata:**

```yaml
depends_on: ["Backend repo validation and recents persistence"]
parallel: false
conflicts_with: []
files:
  - src/lib/stores/repo.svelte.ts
```

**Verification:**

- `pnpm check`
- Store exposes active repo path, recent repos, repo status, loading state, and error state
- Successful open updates recents and routes into `/repo`

### Home route repo-entry surface [ui]

Replace the current root-route spike controls with a product home surface for repo entry, recents display, stale-entry actions, and drag-drop affordances.

**Metadata:**

```yaml
depends_on: ["Frontend repo store and open lifecycle"]
parallel: false
conflicts_with: []
files:
  - src/routes/+page.svelte
```

**Verification:**

- `pnpm check`
- Home route exposes native picker, manual path entry, drag-drop, and recent-repo reopen affordances
- Invalid path errors and stale-entry states are visible and specific

### Repo route shell handoff [ui]

Create the `/repo` route shell and default page needed for successful navigation after repo open, without implementing the summary widgets owned by `bd-6ew.2`.

**Metadata:**

```yaml
depends_on: ["Frontend repo store and open lifecycle"]
parallel: true
conflicts_with: []
files:
  - src/routes/repo/+layout.svelte
  - src/routes/repo/+page.svelte
```

**Verification:**

- `pnpm check`
- App can navigate from `/` to `/repo` after successful repo open
- Route shell exists as a stable handoff target for `bd-6ew.2`

---

## Notes

- Parent PRD decisions inherited from `bd-6ew`: Tauri app data persistence, visible stale recents, distinct `/` and `/repo` surfaces, and repo-context-store architecture.
- Research phase found no existing `src/routes/repo` directory and no relevant TODO/FIXME markers in `src/`.
- Memory search was attempted but project memory search/read was unavailable in this session (`FTS5 not available`, SQLite open error), so this PRD relies on bead artifacts and codebase evidence instead.
