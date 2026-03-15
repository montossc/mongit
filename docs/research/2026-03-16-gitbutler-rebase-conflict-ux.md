# GitButler Rebase Conflict UX: Research for mongit

**Date:** March 16, 2026
**Purpose:** Document GitButler's "fearless rebase" conflict resolution UX and mechanics as a reference for mongit's Fearless Rebase feature (V1.0).
**Confidence:** HIGH — sourced from official docs, source code, and maintainer discussions.

---

## Summary

GitButler implements **non-blocking rebase conflict resolution** — fundamentally different from git's stop-and-fix approach. Instead of halting at the first conflicting commit, GitButler records it, continues rebasing downstream commits, and lets the user resolve conflicts asynchronously, one commit at a time.

---

## The Full Flow

### 1. Pre-flight Conflict Preview

Before executing any rebase/pull/update, the UI shows a preview panel listing every branch and flagging which ones *will* conflict before any action is taken. The user can cancel before it happens.

> Source: [docs — Rebasing and Conflicts](https://docs.gitbutler.com/features/branch-management/merging)

### 2. Non-blocking Rebase Continuation (Core Insight)

GitButler does NOT halt at each conflicting commit. Instead:
1. Applies what it can automatically
2. Records the conflicted commit with a synthetic tree structure (see below)
3. Continues rebasing all downstream commits on top of the conflicted state

This is the single biggest UX win — lets work continue unblocked while conflicts are resolved asynchronously.

### 3. Conflicted Commit Markers

After the rebase, the branch lane in the UI shows each affected commit with a visual conflict badge. The user can click into them individually.

### 4. Per-Commit Edit Mode

Clicking "Resolve Conflict" on a specific conflicted commit triggers Edit Mode:
- All other parallel branches are stashed/removed from the working directory
- Only the conflicted commit is checked out with standard conflict markers in files
- The UI shows a dedicated "Edit Mode" screen

### 5. Save and Exit / Cancel

- **Save and Exit**: Amends the conflicted commit with the resolved tree, strips the "conflicted" commit header, then automatically rebases all downstream commits on the now-clean commit.
- **Cancel**: Cleanly reverts to normal working directory state, no changes made.

### 6. Operations History (Snapshot Undo)

Before every major operation, GitButler captures a full state snapshot (branch state, uncommitted work, conflict state) into the Git object database. The "Operations History" tab shows all snapshots; any entry can be reverted one-click.

> Source: [docs — Operations History](https://docs.gitbutler.com/features/timeline)

### 7. Upstream Integration Options

When a branch has upstream commits the user hasn't integrated: the UI lets the user choose between "Rebase upstream changes" (equivalent to `git pull --rebase`) or "Interactive integration" (reorder, skip, squash commits before integrating).

> Source: [docs — Upstream Integration](https://docs.gitbutler.com/features/branch-management/upstream-integration)

---

## Internal Conflict Representation

From source code at [`cherry_pick.rs`](https://github.com/gitbutlerapp/gitbutler/blob/master/crates/but-rebase/src/graph_rebase/cherry_pick.rs):

A conflicted commit has a **synthetic root tree** containing:

```
.conflict-files         ← TOML blob: lists all conflicted file paths (ours/theirs/ancestor)
CONFLICT-README.txt     ← human-readable warning about not checking out directly
ours/                   ← tree snapshot: our side of the conflict
theirs/                 ← tree snapshot: their side of the conflict
base/                   ← tree snapshot: common ancestor
autoresolution/         ← tree snapshot: auto-resolved tree WITH conflict markers in files
```

Plus a special commit header field (`HEADERS_CONFLICTED_FIELD`) marking the commit as conflicted.

This makes the conflicted commit **un-checkable** by plain Git (intentional — prevents accidental push of unresolved state).

---

## Patterns mongit Should Copy

| # | Pattern | Description |
|---|---------|-------------|
| 1 | **Pre-flight conflict preview** | Before executing rebase, compute outcome speculatively and show summary: "Branch X will have 2 conflicting commits." Requires dry-run merge. |
| 2 | **Non-blocking rebase** | When a commit conflicts, don't halt. Record it as "conflicted commit" with marker in UI, continue rebasing downstream. |
| 3 | **Per-commit conflict badge** | Mark each conflicted commit visually in commit graph (red badge, warning icon). Resolution is scoped to one commit at a time. |
| 4 | **Edit Mode** | Stash everything else, check out just the conflicted commit with conflict markers. Full-screen overlay indicating Edit Mode. Two exit paths: "Save and Exit" (commits + continues) and "Cancel" (reverts). |
| 5 | **Snapshot before destructive ops** | Write state snapshot before any rebase/amend/resolution. Present as scrollable "Operations History" timeline. One-click revert. |
| 6 | **Upstream divergence detection** | When branch has upstream commits, show diverged commits separately. Give choice: "Rebase on top" or "Interactive integration." |
| 7 | **Conflict tree structure** | Store conflict sides (ours/theirs/base/autoresolution) as named subtrees. Include `.conflict-files` manifest. Allows 3-pane diff reconstruction at any time. |
| 8 | **Conflict-aware chain continuation** | When a conflicted commit is resolved, automatically rebase all downstream commits on top. |

---

## Uncertainties and Gaps

| Gap | Description | Confidence |
|-----|-------------|------------|
| 3-pane UI during Edit Mode | Docs don't describe the exact conflict viewer UI. Unclear if CodeMirror-style or simpler file-list approach. | Medium uncertainty |
| Multiple conflicted commits | If commit A and B (downstream) both conflict, the resolution ordering is inferred (bottom-up), not explicitly documented. | Low uncertainty |
| `FailedToMergeBases` UX | Source code has this outcome for merge commits; UX is not described in docs. | Medium uncertainty |
| Conflict format redesign | Current synthetic tree format is under active revision (Discussion #11564). New format: autoresolution tree + SQLite side-map. | Informational |
| Structured merge (tree-sitter) | GitButler evaluating tree-sitter-based merge drivers (Discussion #12274). Not shipped yet. | Future roadmap |

---

## Sources

| # | Source | Type |
|---|--------|------|
| 1 | [Rebasing and Conflicts docs](https://docs.gitbutler.com/features/branch-management/merging) | Official docs |
| 2 | [Operations History docs](https://docs.gitbutler.com/features/timeline) | Official docs |
| 3 | [Upstream Integration docs](https://docs.gitbutler.com/features/branch-management/upstream-integration) | Official docs |
| 4 | [cherry_pick.rs source](https://github.com/gitbutlerapp/gitbutler/blob/master/crates/but-rebase/src/graph_rebase/cherry_pick.rs) | Source code |
| 5 | [Discussion #11564 — Conflict format](https://github.com/gitbutlerapp/gitbutler/discussions/11564) | Maintainer discussion |
| 6 | [Discussion #12274 — Structured merge](https://github.com/gitbutlerapp/gitbutler/discussions/12274) | Maintainer discussion |
