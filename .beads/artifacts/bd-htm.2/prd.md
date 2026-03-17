# PRD: Rust watcher service and debounce lifecycle

**Bead:** bd-htm.2  
**Created:** 2026-03-17  
**Status:** Approved

## Bead Metadata

```yaml
depends_on: []
parallel: false
conflicts_with: []
blocks:
  - bd-htm.3
estimated_hours: 3
requirements_score:
  total: 92
  breakdown:
    business_value: 27
    functional_requirements: 23
    user_experience: 17
    technical_constraints: 15
    scope_and_priorities: 10
  status: passed
  rounds_used: 1
  deferred_questions: 0
```

---

## Problem Statement

### What problem are we solving?

mongit already has a basic Rust watcher and a spike monitor UI, but the current implementation has only the minimum lifecycle guarantees: it starts, emits coarse events, and can be dropped. Before frontend refresh wiring can be trusted, the backend watcher must prove that start, replace, stop, filtering, and debounced event emission behave predictably under real editing activity; otherwise downstream work will build targeted refresh logic on top of a watcher that can silently over-emit, under-emit, or behave inconsistently during repo switches.

### Why now?

Spike D splits watcher responsibilities across child beads. `bd-htm.2` needs to lock in the backend watcher contract now so `bd-htm.3` can consume a stable coarse event source instead of mixing frontend refresh logic with watcher hardening.

### Who is affected?

- **Primary users:** solo power developers who need repository change detection to feel trustworthy before staging and conflict flows exist
- **Secondary users:** downstream implementation beads such as `bd-htm.3`, `bd-20d`, and `bd-3gr` that depend on stable watcher behavior

---

## Scope

### In-Scope

- Harden Rust watcher lifecycle behavior for start, replace, and stop flows
- Preserve and validate the existing debounce window around filesystem bursts
- Preserve and validate path filtering rules for working tree and relevant `.git` paths
- Normalize coarse backend event emission semantics for success and error cases
- Add or improve Rust-side tests that prove lifecycle and debounce behavior
- Use the existing watcher monitor as a manual validation surface without expanding its responsibilities

### Out-of-Scope

- Frontend listener, store, or targeted refresh logic owned by `bd-htm.3`
- Enriching `repo-changed` with per-file payloads for UI routing
- Diff fetching, changed-file list rendering, or repo-backed diff selection
- Multi-repo watching or broader app-shell lifecycle redesign
- Staging controls, conflict actions, or production changes workspace UX

---

## Proposed Solution

### Overview

Keep the Rust watcher as the sole owner of repository-change detection and harden it into a trustworthy coarse event source. The implementation should preserve the current `Debouncer<RecommendedWatcher, RecommendedCache>` model managed in Tauri state, but tighten lifecycle guarantees: invalid paths fail fast, replacing a watcher cleanly drops the previous instance, stopping fully disables further emissions, and relevant file-system bursts coalesce into a single `repo-changed` signal after the debounce window. Rust tests should verify these guarantees using temp-repo fixtures so later frontend work can assume the watcher contract is stable.

### User Flow (if user-facing)

1. Developer opens `/spike-d` and starts watching a repository.
2. The backend watcher begins observing the repo, filters noisy paths, and emits a coarse `repo-changed` event after relevant edits settle.
3. Developer stops or switches watching and the old watcher no longer emits events.

---

## Requirements

### Functional Requirements

#### R1: Stable watcher lifecycle ownership

The backend watcher must own start, replace, and stop behavior in a way that does not require frontend cleanup tricks to stay correct.

**Scenarios:**

- **WHEN** `watch_repo` is called with a valid repository path **THEN** a watcher starts and begins observing that path recursively
- **WHEN** `watch_repo` is called again for a different path **THEN** the previous watcher is replaced cleanly and only the latest watcher remains active
- **WHEN** `stop_watching` is called **THEN** the managed watcher is dropped and no new backend events are emitted from the previous watch session

#### R2: Trusted debounce behavior

The backend must coalesce rapid edit bursts into stable coarse events so downstream UI code does not need to compensate for noisy watcher behavior.

**Scenarios:**

- **WHEN** several relevant file changes happen within the debounce window **THEN** they resolve to a single coarse change notification for that burst
- **WHEN** unrelated noisy changes occur in suppressed paths like `target/`, `node_modules/`, `.git/objects/`, or `.git/logs/` **THEN** they do not trigger `repo-changed`
- **WHEN** meaningful repository metadata changes occur in `.git/index`, `.git/HEAD`, or `.git/refs/*` **THEN** they still trigger `repo-changed`

#### R3: Normalized backend event contract

The watcher must emit a predictable coarse backend contract that later frontend work can subscribe to without redefining semantics.

**Scenarios:**

- **WHEN** relevant changes are observed after debouncing **THEN** the backend emits `repo-changed` with the current coarse no-payload contract
- **WHEN** the debouncer reports watcher errors **THEN** the backend emits a watcher-error signal consistently rather than panicking
- **WHEN** no relevant paths are involved in an event batch **THEN** no coarse repo-change event is emitted

#### R4: Rust-side verification coverage

The watcher hardening work must leave behind automated tests that prove lifecycle and filtering behavior instead of relying only on manual spike usage.

**Scenarios:**

- **WHEN** lifecycle behavior changes later **THEN** Rust tests fail if replacement, stop, or filtering semantics regress
- **WHEN** debounce-related changes are introduced **THEN** test coverage makes over-emission or suppressed legitimate events visible

### Non-Functional Requirements

- **Performance:** backend debounce remains aligned with the current 300ms window so the parent spike can still target roughly 500ms end-to-end updates
- **Security:** watcher behavior must not panic on invalid paths or unexpected watcher errors; failures return errors or emit consistent diagnostics
- **Accessibility:** no additional accessibility requirements in this backend-focused bead; existing monitor UI remains the manual validation surface only
- **Compatibility:** preserve Tauri-managed watcher state (`Mutex<Option<WatcherHandle>>`) and current `notify_debouncer_full` integration unless research proves a change is necessary

---

## Success Criteria

- [ ] Starting, replacing, and stopping the watcher behaves predictably under test
  - Verify: `cargo test watcher --lib`
- [ ] Debounce and path-filter rules preserve legitimate repo changes while suppressing noisy paths
  - Verify: `cargo test watcher --lib`
- [ ] Backend verification passes for the watcher code path
  - Verify: `cargo check`
- [ ] Manual spike validation confirms the monitor only logs events while a watcher is active
  - Verify: open `/spike-d`, start watching a repo, edit relevant files, stop watching, then confirm new edits no longer add watcher events
- [ ] This bead does not absorb frontend refresh ownership from `bd-htm.3`
  - Verify: inspect affected files and confirm no targeted UI refresh/store wiring is added outside watcher diagnostics

---

## Technical Context

### Existing Patterns

- `src-tauri/src/watcher.rs:11-15` - watcher state stored as `Mutex<Option<WatcherHandle>>` in Tauri-managed state
- `src-tauri/src/watcher.rs:23-49` - current path-filter contract already suppresses noisy directories while allowing important `.git` paths
- `src-tauri/src/watcher.rs:56-112` - current start/stop watcher command flow and 300ms debounce setup
- `src-tauri/src/lib.rs:10-26` - Tauri app builder manages watcher state and registers watcher commands
- `src-tauri/src/git/mod.rs:22-91` - temp-repo test fixture helpers for Rust integration-style testing
- `src/lib/components/WatcherMonitor.svelte:18-22` - existing diagnostic event listener for manual validation only

### Key Files

- `src-tauri/src/watcher.rs` - core watcher lifecycle, filtering, debounce, and tests
- `src-tauri/src/lib.rs` - Tauri-managed watcher state registration
- `src/lib/components/WatcherMonitor.svelte` - manual spike validation surface; should remain diagnostic only
- `src/lib/stores/watcher.svelte.ts` - current monitor-oriented store; not primary ownership for this bead

### Affected Files

Files this bead will modify (for conflict detection):

```yaml
files:
  - src-tauri/src/watcher.rs # Harden lifecycle, debounce semantics, normalized event emission, and Rust tests
  - src-tauri/src/lib.rs # Only if watcher state wiring or lifecycle ownership needs a small supporting adjustment
```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
| ---- | ---------- | ------ | ---------- |
| Scope drifts into frontend refresh ownership from `bd-htm.3` | Medium | High | Keep this bead backend-only; do not add route/store refresh wiring |
| Debounce tests become flaky due to timing assumptions | Medium | High | Prefer condition-based assertions and coarse event-count expectations over brittle sleeps |
| Watcher replacement appears correct but old watcher still leaks events briefly | Medium | High | Add lifecycle tests around replace/stop behavior and validate with manual spike flow |
| Current error path only logs or stringifies failures inconsistently | Medium | Medium | Require normalized watcher-error behavior and test non-panic failure handling |

---

## Open Questions

| Question | Owner | Due Date | Status |
| -------- | ----- | -------- | ------ |
| None | — | — | Resolved |

---

## Tasks

Write tasks in a machine-convertible format for `prd-task` skill.

### Harden watcher lifecycle contract [backend]

Refine `watch_repo` and `stop_watching` so backend watcher ownership, replacement, stop semantics, filtering, and coarse event emission remain predictable under repo switches and edit bursts.

**Metadata:**

```yaml
depends_on: []
parallel: false
conflicts_with: []
files:
  - src-tauri/src/watcher.rs
  - src-tauri/src/lib.rs
```

**Verification:**

- `cargo check`
- Inspect `src-tauri/src/watcher.rs` and confirm watcher lifecycle ownership stays in Rust, not frontend refresh code
- Manually use `/spike-d` watcher monitor and confirm starting, switching, and stopping watching behaves predictably for a real repo

### Add watcher lifecycle and debounce tests [backend]

Extend Rust test coverage so path filtering, replacement, stop behavior, and debounced coarse-event semantics are verified by automated tests.

**Metadata:**

```yaml
depends_on:
  - Harden watcher lifecycle contract
parallel: false
conflicts_with: []
files:
  - src-tauri/src/watcher.rs
```

**Verification:**

- `cargo test watcher --lib`
- Confirm tests cover suppressed paths, allowed `.git` paths, and lifecycle stop/replace behavior
- Confirm no frontend refresh wiring was introduced while adding watcher verification coverage

---

## Dependency Legend

| Field | Purpose | Example |
| ----- | ------- | ------- |
| `depends_on` | Must complete before this task starts | `["Setup database", "Create schema"]` |
| `parallel` | Can run concurrently with other parallel tasks | `true` / `false` |
| `conflicts_with` | Cannot run in parallel (same files) | `["Update config"]` |
| `files` | Files this task modifies (for conflict detection) | `["src/db/schema.ts", "src/db/client.ts"]` |

---

## Notes

- User clarification resolved the main ambiguity: `bd-htm.2` is **backend watcher only**.
- `bd-htm.3` owns frontend subscription and targeted refresh behavior; this bead should stop at coarse backend event guarantees.
- Current watcher debounce is 300ms and should remain compatible with the parent spike’s ~500ms end-to-end target.
