# JetBrains IntelliJ IDEA — Version Control Integration UX Analysis

**Date:** March 13, 2026
**Source:** https://www.jetbrains.com/help/idea/version-control-integration.html (and sub-pages)
**Purpose:** Extract the core UX patterns, screens, and interaction design decisions that make JetBrains' VCS integration best-in-class, for use as a reference when designing a competing standalone Git client.

---

## 1. Major User Workflows

### 1.1 Commit Authoring
Full commit flow: track local changes → review diffs → select what to commit (whole files, named changelists, or individual chunks/lines) → write commit message (with history picker and template support) → run pre-commit checks (reformat, analyze code, TODO review, run tests, malicious dependency scan) → commit locally → optionally push immediately. Supports amend, force-push with lease, and partial commits at line granularity.

### 1.2 Branch Lifecycle Management
Create (from HEAD, from selected branch, from specific commit) → switch (with smart checkout that auto-shelves/unshelves dirty state) → compare branches (vs current, vs working tree, two-dot range in log) → merge/rebase/cherry-pick → delete (with unmerged-commit warning and undo notification). Per-branch workspace context (open files, run configs, breakpoints) is saved/restored.

### 1.3 Remote Sync
Fetch (safe, no local change) → Pull (with rebase or merge strategy) → Update Project (all roots, all branches, Ctrl+T) → Push (with tag options, force-push guard on protected branches). Incoming/outgoing commit count indicators on the branch widget.

### 1.4 Conflict Resolution
Triggered automatically on merge/rebase/cherry-pick/stash-apply. Three-pane merge editor (local | result | remote), auto-apply of non-conflicting chunks, line/chunk-level accept/ignore, "Resolve Simple Conflicts" one-click button, editable central pane, post-merge review showing how conflicts were resolved.

### 1.5 History Investigation
Project-level log (all branches), file-level history with rename tracking, directory-level history, selection/line history, snapshot at revision (shows full project tree at any past commit), compare any two commits (multi-file diff), review merge commit resolution (2-pane or 3-pane), git blame with hover popups and inline Code Vision author hints.

### 1.6 History Editing
Amend commit, reword message, interactive rebase (squash/fixup/drop/reorder/stop-to-edit with graph preview before execution), fixup/squash from context menu in log, reset HEAD to commit, revert commit (creates new commit), undo commit.

### 1.7 Work-in-Progress Management
Changelists (IDEA-native, automatic file grouping, unlimited named lists) or Git staging area (opt-in mode), shelf (IDEA-native patch format, selective files, re-applicable), stash (git-native, all uncommitted changes), combined Stash+Shelf tab option.

### 1.8 Cross-Branch Change Application
Cherry-pick entire commit, cherry-pick selected files from a commit, cherry-pick selected changes (file-level hunks), Get from Branch (copy entire file from another branch to current).

---

## 2. Distinctive UX Patterns

### 2.1 Ambient VCS Status Everywhere
Change markers appear in the editor gutter (colored bars: green=new, blue=modified, gray=deleted). File tree nodes are colored (green/blue/gray/red). The branch widget in the title bar shows branch name + incoming (↓) and outgoing (↑) commit counts inline. Status bar shows ongoing operations (rebase in progress, cherry-pick status).

**Design takeaway:** VCS state is a constant background layer, not an interrupting mode-switch.

### 2.2 Inline Commit from the Gutter
Clicking a change marker in the gutter opens a mini-toolbar: write a commit message and commit that single hunk without leaving the editor, optionally amend the last commit.

**Design takeaway:** Collapses the commit workflow to zero mode-switches for small, focused changes.

### 2.3 Dual Staging Models (Changelist vs Git Staging Area)
IDEA defaults to "changelists" (files automatically move into named changelists; staging happens implicitly at commit time with checkboxes). Alternatively, Git staging area mode enables an explicit `git add`-style workflow with per-chunk stage buttons, hollow gutter markers for staged chunks, and a 3-panel interactive staging view (HEAD | staged | working).

**Design takeaway:** Changelists are a more forgiving mental model for developers who don't think in terms of the index. The dual-mode approach serves both audiences.

### 2.4 Partial Commit at Line Granularity
Inside the diff viewer opened from the Commit tool window, each hunk has a checkbox. Right-click on a single line offers "Split Chunks and Include Selected Lines into Commit." The gutter of the diff supports hover-to-toggle per-line inclusion.

**Design takeaway:** Goes deeper than `git add -p`. Allows surgical commits without CLI, solving the "I changed too much in one file" problem.

### 2.5 The Commit Tool Window as a Configurable Surface
The Commit panel (Alt+0) can be: (a) a persistent vertical sidebar, (b) a floating non-modal window, (c) a floating always-on-top window, (d) toggled to show commit controls only on demand, or (e) dissolved back into the Git Log tab as a Local Changes subtab.

**Design takeaway:** Respects different working styles — some want a constant staged-changes monitor, others want commit as a deliberate mode.

### 2.6 Interactive Rebase as a Visual Graph Editor
"Interactively Rebase from Here" opens a dialog listing all commits above the selection point. Each commit can be dragged to reorder, picked, rewound, squashed, fixup'd, or dropped. A graph preview renders the result before execution. Inline mini-editor rewrites commit messages.

**Design takeaway:** Transforms the opaque `git rebase -i` into an approachable visual commit management panel.

### 2.7 Three-Pane Merge Editor with Functional Center
Left (local, read-only), right (remote, read-only), center (editable result — full editor with syntax highlighting, code completion, bracket matching). Conflicts are color-coded. Non-conflicting chunks auto-applied. Simple conflicts get magic-wand one-click resolve.

**Design takeaway:** Allows developers to synthesize resolutions that neither side proposed — crucial for semantic conflicts.

### 2.8 Git Blame as Navigation
Blame gutter shows author + date. Hover opens popup with full commit message and clickable hash → jumps to Log. "Annotate Revision" and "Annotate Previous Revision" walk backward through blame history. "Hide Revision" suppresses noisy commits (e.g., mass reformats). Code Vision inlay hints show author of last change per method/class.

**Design takeaway:** Turns blame from a forensic tool into day-to-day navigation.

### 2.9 Branch Widget as Primary Entry Point
Current branch name in main window header is the main entry point. Clicking opens Branches popup: Recent/Local/Remote/Tags organized hierarchically, prefix-grouping by `/`, favorite branches (star icon), per-branch context menus with every operation.

**Design takeaway:** Centralizes branch discovery at the top of the window, reducing screen real estate cost.

### 2.10 Workspace Context per Branch
When enabled, IDEA saves and restores open files, run configurations, and breakpoints per branch. Branch switching is reversible at the IDE-session level, not just the git level.

**Design takeaway:** Eliminates context-loss friction for developers who context-switch between feature branches.

---

## 3. Product Surfaces a Competing Client Would Need

| # | Screen / Panel | Key Details |
|---|----------------|-------------|
| 1 | **Commit Tool Window** | Staged/unstaged file trees, changelist switcher, message field with history, pre-commit checks, diff preview inline |
| 2 | **Diff Viewer (2-pane)** | Side-by-side or unified, editable right pane, chunk accept/append buttons, whitespace modes, collapse unchanged, sync scrolling |
| 3 | **Three-Pane Merge Editor** | Left=local, center=editable result (full editor), right=remote, per-chunk accept/ignore, auto-apply non-conflicting |
| 4 | **Git Log (Commit Graph)** | Branch topology graph, color-coded labels, filterable by branch/author/date/path, IntelliSort, collapse linear branches, context menus |
| 5 | **Branches Popup / Panel** | Recent, Local, Remote, Tags; prefix-grouped; per-branch context menu; fetch button; search; incoming/outgoing indicators |
| 6 | **Partial Commit / Hunk Selector** | Chunk checkboxes in diff, per-line toggle, "Split Chunks" right-click |
| 7 | **Push Dialog** | Commit list preview, multi-repo grouping, editable remote target, tag options, force push split button |
| 8 | **Interactive Rebase Editor** | Commit list with drag reorder, pick/reword/squash/fixup/drop actions, inline message editor, graph preview |
| 9 | **File History Tab** | Commit list per file, diff preview per revision, rename tracking, Show All Affected Files |
| 10 | **Git Blame Gutter** | Per-line author+date, hover popup, click-to-jump-to-log, annotate previous, hide revision |
| 11 | **Code Vision Author Hints** | Inlay hints above declarations showing last author |
| 12 | **Branch Widget** | Current branch, incoming/outgoing counts, click to open Branches popup |
| 13 | **Shelf / Stash Panel** | Named shelves list, diff preview, unshelve to changelist, conflict handling |
| 14 | **Gutter Change Markers** | Colored bars (new/modified/deleted), click for inline commit/staging |

---

## 4. Sources

| # | URL | Topic |
|---|-----|-------|
| 1 | https://www.jetbrains.com/help/idea/version-control-integration.html | VCS integration overview |
| 2 | https://www.jetbrains.com/help/idea/commit-and-push-changes.html | Commit & push workflows |
| 3 | https://www.jetbrains.com/help/idea/log-tab.html | Git Log tab structure |
| 4 | https://www.jetbrains.com/help/idea/differences-viewer.html | Diff Viewer controls |
| 5 | https://www.jetbrains.com/help/idea/manage-branches.html | Branch management |
| 6 | https://www.jetbrains.com/help/idea/resolve-conflicts.html | Merge editor |
| 7 | https://www.jetbrains.com/help/idea/investigate-changes.html | History, blame, Code Vision |
| 8 | https://www.jetbrains.com/help/idea/shelving-and-unshelving-changes.html | Shelf vs stash |
| 9 | https://www.jetbrains.com/help/idea/apply-changes-from-one-branch-to-another.html | Cherry-pick, rebase |
