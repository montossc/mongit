# PRD: Repo Home and Navigation Shell

**Bead:** bd-6ew
**Parent:** bd-7gm (MVP Core Git Client)
**Depends on:** bd-1zx (Design System Tokens Baseline) — CLOSED
**Type:** feature
**Priority:** P0

## Bead Metadata

```yaml
depends_on:
  - bd-1zx
parallel: false
conflicts_with: []
blocks:
  - bd-20d
  - bd-145
  - bd-1sy
  - bd-268
  - bd-d4a
estimated_hours: 10
```

```yaml
requirements_score:
  total: 95
  breakdown:
    business_value: 28
    functional_requirements: 24
    user_experience: 18
    technical_constraints: 15
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

### What problem are we solving?

mongit needs a real product entry surface instead of a spike-style launcher. A user should be able to reopen the app, choose a repository through practical local entry paths, see recent repositories that persist across restarts, and immediately orient themselves inside the active repo without clicking deeper into the interface.

This parent feature defines the **baseline shell** that downstream workflows depend on: home entry, repo context persistence, route split between home and workspace, and first-glance workspace orientation.

### Why now?

This shell is the prerequisite for later MVP workflows such as changes workspace, commit graph productization, packaging, keyboard-first actions, and branch-context restoration. Without a stable home/workspace split and durable repo context, later beads would have to build on ad-hoc spike UI and duplicate entry logic.

### Who is affected?

- **Primary users:** Solo power developers using mongit as a desktop Git client and expecting quick repo entry plus immediate context.
- **Secondary users:** Internal builders of subsequent workspace surfaces, who need a predictable repo shell and shared repo state contract.

---

## Scope

### In-Scope

- Provide a product-grade home route (`/`) for repository entry
- Support practical local repo opening via:
  - native folder picker
  - pasted or manually entered path
  - drag-and-drop onto the home surface
- Persist recent repositories in Tauri app data and reload them across app restarts
- Keep stale recent paths visible and allow retry or removal
- Establish a runes-based frontend repo store for active repo identity, recents, status, loading, and errors
- Split the app into:
  - a dedicated **home** surface (`/`)
  - a dedicated **repo workspace shell** (`/repo`)
- Show immediate workspace orientation after open:
  - repo name and path
  - current branch or detached-head state
  - changed file count
  - staged file count
  - concise working-tree status messaging
- Define this parent bead as the **umbrella baseline** delivered by child slices `bd-6ew.1` and `bd-6ew.2`

### Out-of-Scope

- Changes workspace and partial staging (`bd-20d`)
- Commit graph productization beyond the shell handoff (`bd-145`)
- Command palette and keyboard-first command system (`bd-268`)
- Packaging and release workflow (`bd-1sy`)
- Workspace context per branch and scalable history index (`bd-d4a`)
- Clone/init repository flows
- Branch switcher UX, remote health, or ahead/behind indicators
- Multi-window or multi-repo simultaneous workspace support
- Watcher-driven live repo summary refresh in this parent baseline

---

## Proposed Solution

### Overview

Treat `bd-6ew` as the umbrella shell feature that turns mongit’s entry flow into a durable baseline product surface. The implementation is split across two already-established child slices:

1. **`bd-6ew.1` — Repo selection and persistence**
   - product home route
   - repo open lifecycle
   - recent repos persistence and stale handling
   - repo workspace route shell
2. **`bd-6ew.2` — Home state summary widgets**
   - orientation-first `/repo` landing surface
   - repo identity, branch context, and changed/staged summary messaging

The parent PRD therefore specifies the end-state contract for the shell as a whole, while explicitly deferring richer workspace capabilities to later beads.

### User Flow

1. User launches mongit and lands on the home route.
2. User opens a repository through folder picker, manual path entry, or drag-and-drop.
3. App validates the path, persists the repo in Tauri app data recents, hydrates repo status, and routes into `/repo`.
4. User immediately sees repo identity and working-tree orientation without additional clicks.
5. On later app launches, user can reopen a recent repo directly from the home surface.

---

## Requirements

### Functional Requirements

#### R1. Product-grade repo entry surface

The app must provide a practical home surface for opening local repositories rather than a spike/demo launcher.

**Scenarios:**
- **WHEN** the user lands on `/` **THEN** they can open a repository through folder picker, manual path entry, or drag-and-drop.
- **WHEN** the user opens a valid repo through any supported entry method **THEN** the result is the same open lifecycle and route transition.
- **WHEN** the user attempts to open an invalid path or non-git directory **THEN** the app shows a clear error state instead of failing silently.

#### R2. Recent repositories persist outside browser storage

Recent repositories must be stored in Tauri app data so the list survives app restarts and browser cache clears.

**Scenarios:**
- **WHEN** the user successfully opens a repository **THEN** it is upserted into a recent-repos list with path, display name, last-accessed timestamp, and validity state.
- **WHEN** the user restarts the app **THEN** the recent repo list is reloaded from app data.
- **WHEN** the recents list exceeds its cap **THEN** least-recent entries are evicted predictably.

#### R3. Stale recent repositories fail safely

The shell must preserve user context for stale recent paths while making recovery obvious.

**Scenarios:**
- **WHEN** a recent path no longer exists or is no longer valid **THEN** the entry remains visible and is marked stale.
- **WHEN** the user selects a stale recent entry **THEN** they can retry the open flow rather than losing the history silently.
- **WHEN** the user decides the stale entry is no longer useful **THEN** they can remove it from recents.

#### R4. Distinct home and workspace surfaces

mongit must separate repo entry from active repo work.

**Scenarios:**
- **WHEN** no repository is active **THEN** the user remains on the dedicated home surface.
- **WHEN** a repository is active **THEN** the user is routed into the dedicated `/repo` workspace shell.
- **WHEN** the workspace shell is entered without an active repo path **THEN** the guard returns the user to home.
- **WHEN** the user is in the workspace **THEN** they can navigate back to home.

#### R5. Immediate repo orientation in workspace

The workspace landing surface must answer “Am I in the right repo?” immediately after open.

**Scenarios:**
- **WHEN** the user lands on `/repo` after opening a repository **THEN** they can see repo name and path without additional action.
- **WHEN** repo status is available **THEN** they can see branch or detached state, changed count, staged count, and concise working-tree messaging.
- **WHEN** repo status is temporarily unavailable **THEN** the landing page shows a calm fallback state instead of broken values.

#### R6. Shared frontend repo context store

The frontend must use a single repo-context store as the baseline contract for entry and shell behavior.

**Scenarios:**
- **WHEN** the app loads the home surface **THEN** the store can load recent repositories from backend commands.
- **WHEN** the user opens a repository **THEN** the store validates, persists, hydrates status, and navigates through a single lifecycle.
- **WHEN** multiple open requests race **THEN** stale async responses do not overwrite newer state.

### Non-Functional Requirements

- **Performance:** Repo orientation should feel immediate on `/repo` because it uses already-loaded store state after the open flow.
- **Security:** Only local repository paths should be accepted; invalid paths must return structured errors rather than panics.
- **Accessibility:** Home and workspace shell controls should remain keyboard reachable, and error/loading states should communicate clear meaning.
- **Compatibility:** Must fit the existing Tauri 2.0 + SvelteKit setup with `ssr=false` and design-token-based styling.
- **Persistence:** Recent repos must use Tauri app data rather than browser-only storage.

---

## Success Criteria

- [ ] User can open a repository from the home surface through picker, typed path, or drag-and-drop.
  - Verify: manual app check from `/`
- [ ] Recent repositories persist across app restarts and remain usable from the home surface.
  - Verify: reopen app and confirm recent entries are present
- [ ] Invalid recent repository paths remain visible and support retry or removal.
  - Verify: manual app check with moved or missing repo path
- [ ] Home (`/`) and workspace (`/repo`) are distinct product surfaces with guard/back-navigation behavior.
  - Verify: manual navigation check between `/` and `/repo`
- [ ] Opening a repo immediately exposes repo identity plus branch/detached state and changed/staged counts.
  - Verify: manual app check after successful open
- [ ] Project verification remains green for the shell baseline.
  - Verify: `pnpm check`
  - Verify: `pnpm build`
  - Verify: `cargo check` (run in `src-tauri/`)

---

## Technical Context

### Existing Patterns

- `src/lib/stores/repo.svelte.ts:32-168` — runes-based store pattern with `$state`, getters, async lifecycle methods, and `openRequestId` race guard
- `src/routes/+page.svelte:9-53` — home route initializes recents on mount and sets up drag/drop listener with cleanup guard
- `src/routes/+page.svelte:101-198` — product home surface pattern for open actions, error banner, and recent repos rendering
- `src/routes/repo/+layout.svelte:8-48` — workspace shell guard and header pattern for active repo context
- `src/routes/repo/+page.svelte:16-64` — orientation-first repo summary pattern using repo store state and derived working-tree messaging
- `src-tauri/src/commands.rs:29-44, 182-210` — async Tauri command pattern using `spawn_blocking`, `Result<_, String>`, and repo/recents IPC contracts
- `src-tauri/src/recents.rs` — recents persistence, validation, capped LRU behavior, and atomic-write backend pattern

### Key Files

- `src/routes/+page.svelte` — home entry surface with picker, manual path, drag/drop, error banner, and recent repos UI
- `src/routes/repo/+layout.svelte` — repo workspace shell and navigation guard
- `src/routes/repo/+page.svelte` — default repo landing summary surface
- `src/lib/stores/repo.svelte.ts` — repo context store and open lifecycle contract
- `src-tauri/src/commands.rs` — Tauri commands for repo open/status and recent repo management
- `src-tauri/src/recents.rs` — recent repo persistence and validation logic

### Affected Files

Files this bead defines as part of the shell baseline:

```yaml
files:
  - src/routes/+page.svelte # Product home route for repo entry and recent repos
  - src/routes/repo/+layout.svelte # Repo workspace shell and route guard
  - src/routes/repo/+page.svelte # Orientation-first repo landing summary
  - src/lib/stores/repo.svelte.ts # Shared repo context store and open lifecycle
  - src-tauri/src/commands.rs # IPC for open repo, repo status, and recent repo operations
  - src-tauri/src/recents.rs # Tauri app-data persistence, validation, and LRU recents handling
```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Parent scope drifts beyond shipped shell baseline into later workspace capabilities | Medium | Medium | Frame `bd-6ew` explicitly as umbrella baseline and defer richer workflows to blocked follow-up beads |
| Entry flow and workspace shell become inconsistent if repo state is managed in multiple places | Low | High | Keep `repoStore` as the single frontend contract for active repo identity, recents, status, loading, and error state |
| Stale recent repo paths confuse users after directories move | Medium | Medium | Keep stale entries visible with retry/remove affordances instead of silently deleting them |
| Future beads assume more from the shell than this baseline guarantees | Medium | Medium | Document exact baseline guarantees here: entry flow, persistence, route split, and immediate orientation only |
| App-wide muted-text contrast remains below ideal accessibility targets | Medium | Low | Treat token-level contrast improvements as a design-system follow-up rather than widening this shell bead |

---

## Open Questions

| Question | Owner | Due Date | Status |
| --- | --- | --- | --- |
| None | — | — | Resolved |

---

## Tasks

Write tasks in a machine-convertible format for `prd-task` skill.

### Repo selection and persistence baseline [frontend]

Ship the home-route entry flow and shared repo open lifecycle so users can open repos, persist recents, and recover from stale recent paths through a single product surface.

**Metadata:**

```yaml
depends_on: []
parallel: true
conflicts_with: []
files:
  - src/routes/+page.svelte
  - src/lib/stores/repo.svelte.ts
  - src-tauri/src/commands.rs
  - src-tauri/src/recents.rs
```

**Verification:**

- `pnpm check`
- `cargo check`
- Manual app check: open a valid repo from `/` using picker, typed path, and drag-drop
- Manual app check: reopen app and confirm recent repos persist
- Manual app check: invalidate a recent path and confirm retry/remove behavior

### Repo workspace orientation shell [ui]

Ship the `/repo` workspace shell and default landing summary so users immediately see repo identity and working-tree context after opening a repository.

**Metadata:**

```yaml
depends_on:
  - Repo selection and persistence baseline
parallel: false
conflicts_with: []
files:
  - src/routes/repo/+layout.svelte
  - src/routes/repo/+page.svelte
  - src/lib/stores/repo.svelte.ts
```

**Verification:**

- `pnpm check`
- `pnpm build`
- Manual app check: opening a repo routes into `/repo`
- Manual app check: `/repo` shows repo name/path, branch or detached state, changed count, staged count, and working-tree messaging

### Parent shell baseline acceptance [integration]

Confirm the combined shell baseline is a stable handoff point for later beads by verifying route split, persistence, and orientation behavior together.

**Metadata:**

```yaml
depends_on:
  - Repo selection and persistence baseline
  - Repo workspace orientation shell
parallel: false
conflicts_with: []
files:
  - src/routes/+page.svelte
  - src/routes/repo/+layout.svelte
  - src/routes/repo/+page.svelte
  - src/lib/stores/repo.svelte.ts
  - src-tauri/src/commands.rs
  - src-tauri/src/recents.rs
```

**Verification:**

- `pnpm check`
- `pnpm build`
- `cargo check`
- Manual app check: navigate from home to workspace and back without losing shell integrity
- Manual app check: confirm the shell feels like a product baseline, not a spike launcher

---

## Notes

- This parent PRD is intentionally framed as the **umbrella baseline** for the repo home and navigation shell.
- Child slices already mapped to this feature are:
  - `bd-6ew.1` — repo selection and persistence
  - `bd-6ew.2` — home state summary widgets
- Later workspace depth remains intentionally deferred to blocked follow-up beads rather than widened into this parent shell spec.
