# mongit Smoke Test Checklist

Manual acceptance tests for validating mongit after a fresh install or build. Work through each section in order — later tests depend on earlier state.

**Version:** 0.1.0
**Platform:** macOS 10.15+

---

## 1. Launch & Home Screen

- [ ] App launches without crash
- [ ] Home screen displays with "mongit" branding
- [ ] Recent repos list is visible (empty on first launch)
- [ ] "Open Repository" button / folder picker is present

## 2. Open Repository

- [ ] Click "Open Repository" — native folder picker opens
- [ ] Select a local Git repository — app navigates to repo workspace
- [ ] Repository name appears in the toolbar
- [ ] Current branch name appears in the toolbar
- [ ] Recent repos list updates to include the opened repo

## 3. Summary Tab

- [ ] Summary tab is selected by default
- [ ] Commit graph renders (if the repo has commits)
- [ ] Branch/tag labels appear on relevant commits
- [ ] Scrolling through the commit graph is smooth

## 4. Changes Tab

- [ ] Click "Changes" tab — changes workspace loads
- [ ] Modified files appear in the left panel with status badges
- [ ] Selecting a file shows its diff hunks in the right panel
- [ ] Diff lines render with correct colors (green for adds, red for deletes)
- [ ] "No changes" message appears when working tree is clean

### 4a. Staging Operations

**Setup:** Make a change to a file in the test repo.

- [ ] Modified file appears with unstaged badge (outline `M`)
- [ ] Click "Stage Hunk" — hunk moves to staged section
- [ ] File badge updates to show staged status (filled `M`)
- [ ] Click "Unstage Hunk" — hunk returns to unstaged section
- [ ] Line selection: click individual `+`/`-` lines to select them
- [ ] "Stage (N)" button appears showing selected line count
- [ ] Stage selected lines — only those lines move to staged

## 5. Branch Operations

- [ ] Create a new branch (via command palette: `CMD+K` → "Create Branch")
- [ ] Switch to the new branch — toolbar updates branch name
- [ ] Fetch from remote (via command palette → "Fetch")
- [ ] Pull from remote (via command palette → "Pull")
- [ ] Push to remote (via command palette → "Push")
- [ ] Delete the test branch (via command palette → "Delete Branch" — if available)

## 6. Commit

**Setup:** Stage some changes first (see 4a).

- [ ] Commit form is visible when changes are staged
- [ ] Type a commit message
- [ ] Click "Commit" — commit is created
- [ ] Changes list clears (or reduces)
- [ ] New commit appears in the commit graph

## 7. Command Palette

- [ ] `CMD+K` opens the command palette modal
- [ ] Search input is focused automatically
- [ ] Type to filter commands — list updates in real-time
- [ ] Commands are grouped by category (Navigation, Git, View, General)
- [ ] Arrow keys navigate through results
- [ ] `Enter` executes the highlighted command
- [ ] `Escape` closes the palette
- [ ] Clicking the backdrop closes the palette

## 8. Keyboard Shortcuts

- [ ] `CMD+K` — opens command palette
- [ ] `CMD+1` — navigates to Summary tab
- [ ] `CMD+2` — navigates to Changes tab
- [ ] `CMD+Shift+T` — toggles dark/light theme
- [ ] `CMD+Shift+F` — triggers fetch
- [ ] `CMD+O` — opens repo folder picker
- [ ] Shortcuts do not fire when typing in a CodeMirror editor

## 9. Theme

- [ ] `CMD+Shift+T` toggles between dark and light modes
- [ ] Dark mode: dark background, light text
- [ ] Light mode: light background, dark text
- [ ] Theme persists after closing and reopening the app
- [ ] System theme mode follows OS preference (if set to system)

## 10. Conflict Resolution (if testable)

**Setup:** Create a merge conflict in the test repo.

- [ ] Conflict banner appears on the Changes page with file count
- [ ] "Resolve (N)" tab appears in the navigation
- [ ] Clicking "Resolve Conflicts" navigates to the resolve workspace
- [ ] Conflicted files are listed with `!` badges
- [ ] Selecting a file opens the 3-pane merge editor
- [ ] "Ours vs Base" pane shows left diff
- [ ] "Theirs vs Base" pane shows right diff
- [ ] "Result" center pane is editable
- [ ] "Accept Ours" / "Accept Theirs" buttons work

## 11. File Watcher

- [ ] Modify a file externally (in a text editor or terminal)
- [ ] Changes page auto-refreshes to show the new modification
- [ ] No manual refresh needed — watcher detects FSEvents

## 12. Window & Navigation

- [ ] Window resizes properly (minimum 900×600)
- [ ] Title bar overlay style works (macOS traffic lights visible)
- [ ] Back button (←) returns to home screen
- [ ] Navigating between tabs preserves state
- [ ] Reopening a recent repo from home screen works

## 13. Performance

- [ ] App startup: under 2 seconds
- [ ] Open a repo with 1000+ commits — graph loads without freezing
- [ ] Scrolling the commit graph stays at 60fps
- [ ] Staging/unstaging responds within 200ms

---

## Test Report Template

```
Date: YYYY-MM-DD
Build: [commit hash or version]
Tester: [name]
Platform: macOS [version], [arch]

Sections passed: [ ] / 13
Sections failed: [ ] / 13
Blockers found: [list or "none"]

Notes:
[Free-form observations]
```
