# mongit

Standalone macOS Git client — Tauri 2.0 + Svelte 5 + Rust.

## Tech Stack

| Layer    | Tech                              | Version    |
| -------- | --------------------------------- | ---------- |
| Desktop  | Tauri 2.0 (WKWebView)            | 2.x        |
| Frontend | Svelte 5 + SvelteKit (runes)     | 5.x / 2.x |
| Backend  | Rust                              | 1.94+      |
| Git      | git2 (vendored-libgit2)          | 0.20       |
| Watcher  | notify + notify-debouncer-full    | 8 / 0.5    |
| Editor   | CodeMirror 6 (planned)           | —          |
| Build    | Vite 7 + Cargo                   | 7.x        |
| Pkg mgr  | pnpm                             | 10.x       |

## Structure

```
src/               → SvelteKit frontend (adapter-static, ssr=false)
src-tauri/src/     → Rust backend (commands.rs = IPC handlers)
docs/research/     → Technical research (8 docs)
docs/plans/        → Product plan + spike plans
```

## Commands

```bash
pnpm check           # svelte-check (0 errors required before commit)
pnpm build           # Vite build → build/
cargo check          # Rust typecheck (run in src-tauri/)
cargo build          # Rust build (run in src-tauri/)
pnpm tauri dev       # Full dev server (frontend + Rust, port 1420)
pnpm tauri build     # Production build (.app bundle)
```

## IPC Pattern (Tauri 2.0)

```rust
// src-tauri/src/commands.rs — define command
#[tauri::command]
pub fn get_repo_status(path: String) -> Result<RepoStatus, String> {
    let repo = git2::Repository::open(&path).map_err(|e| format!("{}", e))?;
    // ...
}
```

```typescript
// Frontend — call command
const status = await invoke('get_repo_status', { path: '/repo' });
```

## Key Constraints

- git2 `Repository` is **not Send+Sync** — use `Arc<Mutex<>>` or open per-call
- `notify` watcher must stay alive in Tauri `State<>`
- CSP: `style-src 'unsafe-inline'` required for CodeMirror 6
- SvelteKit: `ssr=false`, `prerender=true` (static adapter for Tauri)
- Large diffs: use `convertFileSrc()`, not JSON serialization

## Boundaries

- **Always:** `pnpm check` + `cargo check` before commits
- **Ask first:** new dependencies, .opencode/ changes, bead state changes
- **Never:** force push main, commit secrets/.env, modify build/ or target/
