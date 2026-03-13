---
purpose: Project vision, success criteria, and core principles
updated: 2026-03-14
---

# mongit

## Vision

A standalone, free macOS Git client targeting solo power developers. mongit ports JetBrains' best VCS ideas (4-pane log, line-level staging, 3-pane merge, blame-as-navigation) into a lightweight Tauri 2.0 desktop app — filling the empty "deep features + native performance" quadrant that no current client occupies.

## Product Thesis

Solo power developers want a Git client that is simultaneously deep and fast, but every existing option forces a tradeoff:

| Client         | Problem                                   |
| -------------- | ----------------------------------------- |
| GitHub Desktop | Too simple for professionals              |
| GitKraken      | Electron-slow, subscription fatigue       |
| Tower          | Paid subscription, limited depth          |
| Fork           | Good but limited depth                    |
| Sublime Merge  | Fast but no team features, limited staging |
| JetBrains VCS  | Best UX, but locked inside a JVM IDE      |

**Our answer:** JetBrains-grade VCS experience in a standalone, free, native-speed macOS app.

## Success Criteria

- [ ] Users can open a local repo and view commit graph with 10k+ commits at 60fps
- [ ] Line-level and hunk-level staging works for partial commits
- [ ] Commit, amend, push operations work with hooks and signing
- [ ] Merge conflicts resolvable in 3-pane editor
- [ ] All operations keyboard-accessible with command palette (CMD+K)
- [ ] Binary size < 25 MB, RAM baseline < 150 MB, startup < 2 seconds
- [ ] Dark/light theme following system

## Target Users

### Primary

- **Solo power developers** who live in the terminal but want visual git operations
- Git experts who find GitHub Desktop too shallow and GitKraken too bloated

### User Needs

- Visual commit graph that handles real-world repos (10k+ commits, 50+ branches)
- Line-level staging without `git add -p` friction
- Fast, native-feeling UI without Electron overhead
- Keyboard-first workflow with progressive disclosure

## Core Principles

1. **Ambient status** — VCS state is always visible, never hidden behind a panel
2. **Operation preview** — Show what will happen before every destructive action
3. **Undo everything** — Every mutation is reversible with one action
4. **Keyboard-first** — Every action reachable via shortcut or command palette
5. **Progressive disclosure** — Clean surface for beginners; depth for experts
6. **Premium calm** — Sharp typography, generous spacing, no visual noise

## Architecture

```
mongit/
├── src/                    # SvelteKit frontend
│   ├── app.css             # Design tokens + global styles
│   ├── routes/             # SvelteKit pages
│   └── lib/                # Shared components, stores, utils
├── src-tauri/              # Rust backend
│   ├── Cargo.toml          # Dependencies (git2, notify, tauri)
│   ├── tauri.conf.json     # Window, CSP, plugins
│   └── src/
│       ├── main.rs         # Entry point
│       ├── lib.rs          # Tauri builder + plugin setup
│       └── commands.rs     # IPC command handlers
├── docs/
│   ├── research/           # Technical research (8 docs)
│   ├── plans/              # Product plan, spike plans
│   └── handoffs/           # Session handoffs
└── .opencode/              # AI development context
```

## Key Files

| File                          | Purpose                                |
| ----------------------------- | -------------------------------------- |
| `src-tauri/tauri.conf.json`   | App config, window, CSP, plugins       |
| `src-tauri/Cargo.toml`        | Rust dependencies                      |
| `svelte.config.js`            | SvelteKit config (adapter-static)      |
| `vite.config.ts`              | Vite + Tauri dev server                |
| `docs/plans/`                 | Product plan + spike plans             |
| `docs/research/`              | 8 research documents                   |

---

_Update this file when vision, success criteria, or principles change._
