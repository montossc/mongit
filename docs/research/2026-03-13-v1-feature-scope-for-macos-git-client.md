# V1 Feature Scope: macOS Git Client

**Date:** March 13, 2026
**Purpose:** Define the exact feature scope for the MVP and V1.1 releases, based on research into JetBrains VCS, competitor landscape, and target persona.

---

## Release Strategy

| Release | Target | Focus |
|---------|--------|-------|
| **MVP (V0.1)** | 3-4 months | Core git operations with premium UX |
| **V1.0** | 6-8 months | Full solo developer workflow |
| **V1.1** | 9-12 months | Power features (work buckets, advanced history) |
| **V2.0** | 12-18 months | AI intelligence, stacked diffs, optional Windows |

---

## MVP (V0.1) — "Commit with Confidence"

### Repo Home
- Open local git repository (drag-and-drop or file picker)
- Recent repos list with quick switch
- Repository status overview (branch, ahead/behind, clean/dirty)

### Commit Graph
- Visual branch topology (DAG rendering in WebGL/Canvas)
- Color-coded branch labels (HEAD, local, remote)
- Click commit → see details + changed files + diff
- Basic filtering: by branch, by author
- Scroll performance: smooth at 10k+ commits

### Local Changes Workspace
- File list showing modified/added/deleted files
- Hunk-level staging (checkbox per hunk in diff view)
- Line-level staging (toggle individual lines)
- Side-by-side diff viewer (CodeMirror 6)
- Unstage hunks/lines
- Discard changes (with confirmation)

### Commit Authoring
- Commit message editor with subject/body split
- Commit history dropdown (recent messages)
- Amend last commit
- Commit + Push in one action

### Branch Operations
- Create branch (from HEAD)
- Switch/checkout branch
- Delete branch (with unmerged warning)
- Branch list panel (local + remote)
- Fetch from remote
- Pull (merge or rebase strategy)
- Push (with upstream tracking)

### Conflict Resolution
- Detect merge/rebase conflicts
- 3-pane merge editor (local | result | remote)
- Per-chunk accept/ignore buttons
- Auto-apply non-conflicting changes
- "Continue rebase/merge" action after resolution

### UX Foundation
- Keyboard shortcuts for all common operations
- Command palette (CMD+K)
- Operation preview (show what will happen before destructive actions)
- Undo notification after destructive operations
- Dark/light theme (follow system)
- Native macOS menu bar integration

---

## V1.0 — "Full Solo Workflow"

Everything in MVP, plus:

### Enhanced Commit Graph
- IntelliSort-style merge display
- Collapse/expand linear branches
- Search commits by message, author, hash
- Multi-branch filtering
- Commit context menu (cherry-pick, revert, rebase from here)

### History Investigation
- File history (commits affecting a specific file)
- Git blame gutter (author + date per line)
- Blame hover popup (commit message, clickable hash)
- "Annotate Previous Revision" navigation
- Compare any two commits (multi-file diff)

### Advanced Branch Operations
- Merge branch into current
- Rebase current onto branch
- Cherry-pick commit(s)
- Interactive rebase (visual editor: pick/squash/fixup/drop/reorder)
- Graph preview before rebase execution

### Stash Management
- Stash current changes
- Stash list with preview
- Apply/pop/drop stash
- Stash with message

### Safety Net (Undo)
- Undo last commit
- Undo last rebase (reflog-backed)
- Undo last merge
- Undo last discard
- Operation history panel

### Tag Management
- Create tag (lightweight and annotated)
- Delete tag
- Push tags
- Tag list in branch panel

---

## V1.1 — "Power-Up"

Everything in V1.0, plus:

### Work Buckets (Reinterpreted Changelists)
- Named groupings of changes (like JetBrains changelists)
- Files assigned to buckets, not the git index
- Commit from a specific bucket
- Move files/hunks between buckets
- Persisted across sessions

### Shelf (Persistent WIP Storage)
- Save work-in-progress to named shelf entries
- Selective shelving (specific files/hunks)
- Unshelve with conflict handling
- Shelf list with diff preview

### Advanced Blame
- "Hide Revision" (suppress noisy commits like reformats)
- Blame for a specific revision range
- Blame performance optimization for large files

### Pre-Commit Checks
- Detect and run `.git/hooks/pre-commit`
- Show check results inline in commit panel
- Allow override (commit despite hook failure)

### Workspace Context per Branch
- Save/restore UI state when switching branches
- Remember: selected files, scroll positions, panel layout
- Automatic on branch checkout

### Log Index / Search
- Full-text search across commit messages
- Search by file path
- Search by date range
- Fast incremental indexing

---

## V2.0 — "Intelligence & Expansion"

### Stacked Diffs
- Visual dependency chain for branch stacks
- Rebase-on-merge management
- Stack status overview

### AI Workflow Intelligence
- Semantic conflict detection
- Diff narration (explain changes in English)
- Interactive rebase advisor
- Commit message generation (optional, focused)

### Windows Support
- Tauri 2.0 WebView2 backend
- Windows-native installer (MSI)
- Platform-specific keyboard shortcuts

### Optional Cloud Features
- Settings sync across machines
- Backup/restore of work buckets and shelves
- Usage analytics (opt-in)

---

## Explicitly Out of Scope (All Versions)

| Feature | Reason |
|---------|--------|
| PR review inside the app | Separate product concern; browser is adequate |
| Issue tracker integration | Not relevant for solo power dev persona |
| Enterprise SSO/SAML | Not the target audience |
| Multi-repo project views | Complex; address in V3+ if demand exists |
| Plugin/extension system | Focus on core quality first |
| Linux support | Small market for paid/premium tools; V3+ |
| SVN/Mercurial support | Git-only product |
| Built-in terminal | Not a productivity multiplier for the target persona |

---

## Tech Stack Summary

| Component | Technology |
|-----------|------------|
| Desktop shell | Tauri 2.0 |
| Backend | Rust |
| Frontend | React or Svelte + TypeScript |
| Git reads | libgit2 via `git2` Rust crate |
| Git writes | Bundled Git binary |
| Diff/editor | CodeMirror 6 |
| Commit graph | WebGL/Canvas |
| File watching | macOS FSEvents via Tauri |
| App data | ~/Library/Application Support/ |
| Auto-update | tauri-plugin-updater |
| Packaging | Tauri bundler (dmg) |

---

## Validation Criteria for MVP

- [ ] Can open a local git repo and see commit graph with 10k+ commits at 60fps
- [ ] Can stage/unstage individual lines and hunks
- [ ] Can commit with message, amend, and push
- [ ] Can create/switch/delete branches
- [ ] Can resolve merge conflicts in 3-pane editor
- [ ] Startup < 2 seconds
- [ ] RAM usage < 150MB with medium repo (10k commits)
- [ ] Binary size < 25MB (dmg)
- [ ] All operations accessible via keyboard shortcuts
- [ ] Command palette works for all actions
