# Git Engine Architecture: Hybrid libgit2 + Bundled Git

**Date:** March 13, 2026
**Purpose:** Document the recommended Git engine strategy for a standalone desktop Git client, with detailed tradeoffs and gotchas.

---

## Recommendation

**Hybrid approach: libgit2 for read/query operations + bundled Git binary for write/complex operations.**

This is the industry-proven pattern. GitHub Desktop (via `dugite`) does exactly this. VS Code's git extension, Sourcetree, and Xcode also use libgit2 for reads.

---

## Architecture

```
┌────────────────────────────────────────────────────────────┐
│                     Desktop UI Layer                       │
│             (React/Svelte in Tauri webview)                │
└────────────────────────┬───────────────────────────────────┘
                         │
┌────────────────────────▼───────────────────────────────────┐
│            GitRepository Abstraction Interface             │
│  (single surface: status, diff, commit, rebase, push ...)  │
└──────┬──────────────────────────────┬──────────────────────┘
       │                              │
       │  READ / QUERY PATH           │  WRITE / MUTATION PATH
       │  (fast, in-process)          │  (safe, compatible)
       │                              │
┌──────▼───────────┐        ┌─────────▼────────────────────┐
│   libgit2 (git2) │        │   Bundled Git Binary          │
│                  │        │   (bundled, not system git)   │
│  • status        │        │                               │
│  • diff          │        │  • commit (hooks fire)        │
│  • revwalk/log   │        │  • rebase (interactive too)   │
│  • index/staging │        │  • cherry-pick / revert       │
│  • blame         │        │  • push / fetch with creds    │
│  • partial stage │        │  • GPG / SSH signing          │
│    (hunk-level)  │        │  • merge                      │
│  • conflict data │        │  • credential helpers         │
└──────────────────┘        └──────────────────────────────┘
```

---

## Tradeoff Table

| Dimension | Shell Out (bundled git) | libgit2 | gitoxide (gix) | JGit | isomorphic-git |
|-----------|------------------------|---------|----------------|------|----------------|
| **Compatibility** | ✅ Perfect | ⚠️ High but edge cases | ⚠️ High; tracked gaps | ✅ High | ❌ Low |
| **Hooks** | ✅ Automatic | ❌ Must exec manually | ⚠️ Partial | ⚠️ Manual | ❌ None |
| **GPG/SSH Signing** | ✅ Delegates naturally | ❌ No built-in | ❌ None | ⚠️ Via process | ❌ None |
| **Credential Helpers** | ✅ Automatic | ⚠️ Manual wiring | ⚠️ Partial | ⚠️ Partial | ❌ None |
| **Rebase/Sequencer** | ✅ Full interactive | ✅ API exists | ❌ Not implemented | ✅ Full | ❌ Limited |
| **Cherry-Pick/Revert** | ✅ | ✅ API exists | ⚠️ No high-level API | ✅ | ❌ |
| **Merge (3-way)** | ✅ | ✅ Full API | ✅ Initial dev | ✅ | ⚠️ Basic |
| **Partial Staging** | ⚠️ Via git apply | ✅ Direct index API | ⚠️ Low-level only | ✅ DirCache | ❌ |
| **Conflict Resolution** | ⚠️ Parse files | ✅ Structured iterator | ⚠️ Evolving API | ✅ | ❌ |
| **Large Repos** | ✅ Native optimizations | ⚠️ Some limits | ⚠️ Some limits | ✅ But memory-heavy | ❌ No partial clone |
| **Thread Safety** | ✅ Independent processes | ⚠️ Per-object locking | ✅ Rust ownership | ✅ | ✅ (single-thread) |
| **Binary Size** | ~30-50MB | ~1-3MB | ~5-15MB | ~40MB+ JVM | ~2MB JS |
| **Rust Bindings** | Subprocess | `git2` crate | `gix` crate | N/A | N/A |
| **Production Readiness** | ✅ Decades | ✅ GitHub, VS Code, Xcode | ⚠️ Reads only | ✅ IntelliJ, Gerrit | ⚠️ Limited |
| **License** | GPL-2.0 (binary OK) | GPL-2.0 + linking exception | MIT/Apache-2.0 | EDL (BSD-like) | MIT |

---

## Major Gotchas

### libgit2
1. **Hooks are never automatic.** libgit2 does NOT run pre-commit, post-commit, commit-msg hooks. You must locate and exec them yourself. This is documented behavior.

2. **No native GPG/SSH signing.** `git_commit_create` has no signing parameter. You must call GPG yourself or shell out to `git commit -S`.

3. **Thread safety is object-scoped.** Do NOT share `git_repository`, `git_index`, or mutable objects across threads without external locking. Run all git operations on a dedicated serial queue.

4. **Rebase does not run hooks.** `git_rebase_commit` does not invoke post-commit hooks. Users with commit hooks for linting/signing will see them silently skipped.

5. **Credential helper chain is not automatic.** You must implement the helper lookup logic yourself; libgit2 doesn't automatically chain through `.gitconfig` credential helpers.

### gitoxide
6. **No rebase or sequencer.** As of early 2026, `gix-rebase` and `gix-sequencer` are listed as "idea (just a name placeholder)." Cannot be used for interactive rebase, squash, fixup, or `git pull --rebase`.

7. **Protocol V1 over SSH may hang.** Documented shortcoming. Protocol V2 works fine; V1 still used by some enterprise servers.

8. **Split index writes disabled.** Mutating operations disable split-index, which can surprise power users.

### Shell-out
9. **Always bundle Git.** Never rely on system git. macOS ships ancient Xcode CLT git (2.39). GitHub Desktop bundles per-platform via `dugite`.

10. **Use structured output.** Parse `--porcelain=v2 -z` (NUL-delimited), not human-readable output. One format change historically broke multiple clients.

11. **Conflict data requires parsing.** Unlike libgit2's structured iterator, shelling out gives conflict markers in files. You lose programmatic 3-pane diff without extra parsing.

---

## Implementation Strategy

### Phase 1: MVP (libgit2 reads + git subprocess writes)
- `git2` Rust crate for: status, diff, blame, log (revwalk), index manipulation, conflict data reading
- Bundled git binary for: commit, push, fetch, pull, merge, rebase, cherry-pick, stash
- Single `GitRepository` trait abstracting both paths

### Phase 2: Optimize (gitoxide for hot paths)
- Evaluate `gix` for pack reads, object resolution, and revwalk (2-10× faster than libgit2)
- Keep libgit2 for index manipulation and conflict data
- Keep bundled git for all writes

### Phase 3: Full gitoxide (when ready)
- Monitor `gix-rebase`, `gix-sequencer`, `gix-merge` maturity
- Migrate write operations incrementally as APIs stabilize
- Goal: reduce dependency on subprocess for most operations

---

## Sources

| Source | URL |
|--------|-----|
| libgit2 API samples | https://libgit2.org/docs/guides/101-samples/ |
| libgit2 threading docs | https://github.com/libgit2/libgit2/blob/main/docs/threading.md |
| libgit2 rebase.h | https://github.com/libgit2/libgit2/blob/main/include/git2/rebase.h |
| libgit2 merge.h | https://github.com/libgit2/libgit2/blob/main/include/git2/merge.h |
| gitoxide README | https://github.com/GitoxideLabs/gitoxide/blob/main/README.md |
| gitoxide SHORTCOMINGS.md | https://github.com/GitoxideLabs/gitoxide/blob/main/SHORTCOMINGS.md |
| GitHub Desktop rebase.ts | https://github.com/desktop/desktop/blob/development/app/src/lib/git/rebase.ts |
| gix crate docs | https://docs.rs/gix/latest/gix |
