# PRD: Home state summary widgets

**Bead:** bd-6ew.2
**Parent:** bd-6ew (Repo Home and Navigation Shell)
**Depends on:** bd-6ew.1 (Repo selection and persistence), bd-6ew (parent-child)
**Type:** task
**Priority:** P1

```yaml
requirements_score:
  total: 93
  breakdown:
    business_value: 27
    functional_requirements: 24
    user_experience: 18
    technical_constraints: 14
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

`bd-6ew.1` established the product home flow, repo persistence, and the `/repo` workspace shell, but the default repo landing page is still a placeholder with a minimal three-value grid and a generic note. After opening a repository, users still need a clearer orientation surface that answers the first question they have on landing: **“Am I in the right repo?”**

This bead refines the `/repo` landing experience into an orientation-first summary surface. It should make repository identity and working-tree state immediately legible without widening into a dashboard, activity feed, or workflow hub.

## Scope

### In-Scope

- Replace the placeholder `/repo` landing page with an orientation-focused summary surface
- Make the repo landing page immediately communicate:
  - repository name and/or path
  - current branch or detached-head state
  - changed file count
  - staged file count
  - concise working-tree state messaging (for example: clean, has unstaged changes, ready to commit)
- Use existing frontend repo context from `src/lib/stores/repo.svelte.ts`
- Use the existing `get_repo_status` backend command and current repo-shell routing introduced in `bd-6ew.1`
- Keep the summary **static on load** for this bead:
  - data is populated from the existing successful open flow
  - no watcher-driven live refresh is required in this slice
- Reuse existing design tokens and UI primitives where they fit (`Panel`, `Badge`, `Button` only if needed)
- Provide graceful fallback presentation when repo status is temporarily unavailable or branch is detached

### Out-of-Scope

- Recent commit lists, activity timelines, or history widgets
- Changed-file preview lists or embedded diff previews
- Ahead/behind or remote sync indicators
- Stash count, branch switcher UX, or other advanced Git health surfaces
- Watcher-driven auto-refresh of summary state
- “What should I do next?” workflow CTAs or command launcher surfaces
- Any new backend command if the existing repo status contract is sufficient
- Reworking the `/` home route or recent-repo flows owned by `bd-6ew.1`

## Proposed Solution

Replace the current placeholder content in `src/routes/repo/+page.svelte` with a polished, orientation-only landing surface built on the existing `repoStore` state. The summary page should be visually calmer and more intentional than the current grid while still staying lightweight.

The summary should answer three things at a glance:

1. **Which repo am I in?**
   - show repo name prominently
   - show the current repo path or another clear identity cue

2. **Which branch/context am I in?**
   - show current branch clearly
   - if detached, render that state explicitly instead of leaving ambiguity

3. **What is the current working-tree state?**
   - show changed/staged counts
   - synthesize those counts into a concise state message

This bead should stay frontend-only unless implementation discovers a true missing contract. Existing store state already includes:

- `activeRepoName`
- `activeRepoPath`
- `repoStatus.branch`
- `repoStatus.changed_files`
- `repoStatus.staged_files`

### User Flow

1. User opens a repository from `/` through the `bd-6ew.1` flow.
2. App navigates to `/repo` with repo context already loaded in `repoStore`.
3. User immediately sees a summary surface that confirms repo identity, branch context, and clean/dirty state.
4. User can answer “Am I in the right repo?” within 1-2 seconds without clicking deeper into the app.

## Requirements

### Functional Requirements

#### R1. Orientation-first repo landing

The `/repo` default page must present an orientation summary instead of a placeholder note.

**Scenarios:**
- **WHEN** the user lands on `/repo` after opening a repository **THEN** the page shows a deliberate summary surface rather than a temporary placeholder.
- **WHEN** the user views the summary page **THEN** the information hierarchy prioritizes identity and state clarity over dashboard breadth.

#### R2. Repository identity visibility

The summary must clearly identify the active repository.

**Scenarios:**
- **WHEN** a repository is loaded **THEN** the user can see the repo name without additional action.
- **WHEN** a repository is loaded **THEN** the user can also see enough identity detail to distinguish similarly named repositories, such as the absolute path or a clearly scoped path label.

#### R3. Branch-context visibility

The summary must clearly communicate current branch context.

**Scenarios:**
- **WHEN** the repository has a current branch **THEN** that branch is shown prominently.
- **WHEN** the repository is in detached HEAD state **THEN** the summary explicitly communicates that state instead of leaving the branch field blank or ambiguous.

#### R4. Working-tree state summary

The summary must expose actionable repo state immediately using existing status data.

**Scenarios:**
- **WHEN** the repository status is available **THEN** the page shows changed file count and staged file count.
- **WHEN** changed and staged counts are both zero **THEN** the summary communicates a clean working tree.
- **WHEN** changed files exist but staged files do not **THEN** the summary communicates that there are unstaged changes.
- **WHEN** staged files exist **THEN** the summary communicates that staged work is present and ready for commit-related follow-up in later beads.

#### R5. Static-on-load behavior

This bead must keep the summary lifecycle intentionally simple.

**Scenarios:**
- **WHEN** the user arrives on `/repo` after the existing open flow **THEN** the summary reads the already-loaded store state.
- **WHEN** repo-changed watcher events fire later **THEN** this bead does not require the summary to live-refresh automatically.
- **WHEN** the user opens a different repository through the normal open flow **THEN** the next `/repo` landing reflects that repo’s state.

#### R6. Graceful empty/error fallback

The summary page must fail safely if repo status is temporarily absent.

**Scenarios:**
- **WHEN** repo identity exists but `repoStatus` is temporarily null **THEN** the page shows a calm fallback state rather than broken or misleading values.
- **WHEN** branch information is unavailable **THEN** the page communicates `detached` or an equivalent explicit fallback label.

### Non-Functional Requirements

- **Performance:** Summary rendering should rely on already-loaded frontend state from the successful open flow and feel immediate on `/repo` load.
- **Compatibility:** Must fit the current Tauri 2.0 + SvelteKit static-adapter setup with `ssr=false`.
- **Design system:** Prefer existing tokens and primitives in `src/app.css` and `src/lib/components/ui/`; avoid introducing dashboard-specific one-off patterns.
- **Scope discipline:** Do not fetch additional backend data or introduce live-refresh complexity unless implementation proves the current contract is insufficient.
- **UX:** The surface should answer “Am I in the right repo?” before it tries to answer “What changed recently?” or “What should I do next?”

## Success Criteria

- [ ] User lands on `/repo` and immediately sees a real summary surface instead of placeholder copy.
  - Verify: manual app check after opening a repo from `/`
- [ ] User can identify the active repository from name plus supporting identity detail (such as path) within the landing surface.
  - Verify: manual app check with two similarly named repos in different directories
- [ ] User can immediately see current branch (or detached state), changed files count, and staged files count without additional clicks.
  - Verify: manual app check with clean repo, dirty repo, and detached-head repo if available
- [ ] Summary behavior stays static-on-load and does not require watcher-driven live updates in this bead.
  - Verify: implementation uses existing loaded store state and does not introduce watcher subscriptions on `/repo`
- [ ] Project verification remains green after implementation.
  - Verify: `pnpm check`
  - Verify: `pnpm build`
  - Verify: `cargo check` (project baseline gate)

## Technical Context

### Existing Patterns

- `src/routes/repo/+page.svelte:1-31` — current minimal summary baseline with branch/changed/staged grid and placeholder note; this is the direct replacement target
- `src/routes/repo/+layout.svelte:16-48` — repo shell already renders repo name and branch label in the header and guards navigation if no repo is active
- `src/lib/stores/repo.svelte.ts:32-166` — runes-based store already owns `activeRepoPath`, `activeRepoName`, `repoStatus`, loading, and error state
- `src-tauri/src/commands.rs:28-44` — `get_repo_status(path)` already returns `branch`, `changed_files`, and `staged_files`; no new backend contract is currently required
- `src/lib/components/ui/index.ts:1-4` — `Badge`, `Button`, `Input`, and `Panel` are already exported for reuse
- `src/lib/stores/watcher.svelte.ts:9-70` — watcher pattern exists, but this bead explicitly does **not** depend on it for live summary updates

### Key Constraints

- `bd-6ew.1` already owns repo opening, recents persistence, and route handoff; this bead must build on that baseline rather than reopen route-entry work
- Parent PRD `bd-6ew` defines immediate repo orientation as in-scope, but explicitly excludes advanced remote or dashboard-style surfaces
- Recent review fixes in `bd-6ew.1` introduced request-id guards and static-on-open state hydration; this bead should preserve those patterns instead of adding competing async flows
- No TODO/FIXME markers were found in `src/` for this area during refinement
- Institutional memory search was unavailable in this session (`FTS5 not available`), so refinement relied on bead artifacts, recent git history, and direct codebase evidence

### Affected Files

```yaml
files:
  - src/routes/repo/+page.svelte # Replace placeholder repo landing content with orientation-only summary widgets
  - src/routes/repo/+layout.svelte # Optional shell/header alignment if summary hierarchy needs minor support changes
  - src/lib/stores/repo.svelte.ts # Only if small derived helpers or fallback presentation state are needed; avoid widening store scope
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| --- | --- | --- | --- |
| Scope drifts into mini-dashboard behavior | Medium | Medium | Lock the feature around identity + branch + changed/staged counts + concise state messaging only |
| Summary duplicates header info without adding clarity | Medium | Low | Use the page to deepen identity/state understanding, not to mirror the toolbar verbatim |
| Implementation adds watcher/live-refresh complexity prematurely | Medium | Medium | Keep behavior static on load and defer live refresh to a later bead if it becomes product-valuable |
| Repo status absent or detached-head state renders awkwardly | Low | Medium | Define explicit fallback copy and detached-state presentation in the implementation |
| Similar repo names remain ambiguous | Low | Medium | Require supporting identity detail such as path in the summary surface |

## Open Questions

None. Refinement resolved the key scope decisions for this slice:

- orientation-only summary
- static-on-load behavior
- primary user question: “Am I in the right repo?”

## Tasks

### Repo workspace orientation summary surface [ui]

Replace the placeholder `/repo` landing page with an orientation-focused summary surface that confirms repo identity, branch context, and current working-tree state using the existing `repoStore` contract.

**Metadata:**

```yaml
depends_on: []
parallel: false
conflicts_with: []
files:
  - src/routes/repo/+page.svelte
  - src/routes/repo/+layout.svelte
  - src/lib/stores/repo.svelte.ts
```

**Verification:**

- `pnpm check`
- `pnpm build`
- `cargo check`
- Manual app check: open a valid repo from `/` and confirm `/repo` shows repo identity, branch/detached state, changed files count, staged files count, and concise clean/dirty messaging
- Manual app check: open two repos with similar names in different directories and confirm the summary surface distinguishes them clearly

---

## Notes

- This bead is the intentionally narrow follow-up to `bd-6ew.1`: it upgrades the repo landing page from placeholder to product-facing orientation surface.
- Existing backend status data appears sufficient for the current scope; if implementation discovers otherwise, that should be treated as a scoped follow-up decision rather than assumed upfront.
