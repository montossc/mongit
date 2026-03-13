---
purpose: Tech stack, constraints, and integrations for AI context injection
updated: 2026-03-14
---

# Tech Stack

This file is automatically injected into ALL AI prompts via `opencode.json` instructions[].

## Framework & Language

- **Desktop Framework:** Tauri 2.0 (WKWebView on macOS)
- **Frontend:** Svelte 5 + SvelteKit (adapter-static, SSR disabled)
- **Backend:** Rust (stable 1.94+)
- **Language:** TypeScript (frontend), Rust (backend)

## Key Dependencies

### Frontend (package.json)

- **SvelteKit:** ^2.50.2 with adapter-static ^3.0.0
- **Svelte:** ^5.51.0 (runes mode enabled)
- **Tauri API:** @tauri-apps/api ^2.0.0
- **Tauri CLI:** @tauri-apps/cli ^2.0.0 (devDep)
- **Vite:** ^7.3.1

### Backend (Cargo.toml)

- **Tauri:** 2 (with shell plugin)
- **git2:** 0.20 (vendored-libgit2 feature — no system dependency)
- **notify:** 8 (FSEvents on macOS for file watching)
- **notify-debouncer-full:** 0.5
- **tokio:** 1 (async runtime, full features)
- **serde:** 1 (serialization)
- **thiserror:** 2 (error types)

## Build & Tools

- **Frontend Build:** Vite (via SvelteKit)
- **Backend Build:** Cargo (via Tauri CLI)
- **Dev Server:** `pnpm tauri dev` (port 1420, HMR)
- **Package Manager:** pnpm
- **TypeCheck:** `pnpm check` (svelte-check)
- **Rust Check:** `cargo check` (in src-tauri/)

## Key Constraints

- **Node.js >= 20** required
- **Rust stable** required (1.94+)
- **macOS only** for V1 (WKWebView)
- SvelteKit runs with `ssr=false`, `prerender=true` (static adapter for Tauri)
- Vite dev server on port 1420 (strictPort) for Tauri
- CSP requires `style-src 'unsafe-inline'` for CodeMirror 6
- git2 `Repository` is not Send+Sync — use `Arc<Mutex<>>` or open per-call
- `notify` watcher must be kept alive in Tauri `State<>`

## Architecture Decisions

| Decision                          | Rationale                                                    |
| --------------------------------- | ------------------------------------------------------------ |
| Svelte 5 + SvelteKit over React   | Smaller bundle, GitButler precedent, native-feel performance |
| Custom Canvas 2D for commit graph  | DOM breaks at 1000+ nodes, WebGL overkill                    |
| git2 reads + bundled git writes    | GitHub Desktop pattern; git2 lacks hooks/signing             |
| notify crate for file watching     | Native FSEvents on macOS, emit Tauri events to frontend      |
| CodeMirror 6 for diff/merge        | Virtual scrolling, 100k+ lines, battle-tested                |

## IPC Patterns (Tauri 2.0)

- **Commands:** Async request/response (`#[tauri::command]` → `invoke()`)
- **Events:** One-way broadcast (progress, background updates)
- **Channels:** Streaming (file watcher, long operations)
- **Large diffs:** Use `convertFileSrc()` for file-backed references (avoid JSON serialization)

## Release Profile (Cargo.toml)

```toml
[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
```

---

_Update this file when tech stack or constraints change._
_AI will capture architecture, conventions, and gotchas via the `observation` tool as it works._
