---
purpose: User identity, preferences, communication style
updated: 2026-03-14
---

# User Profile

## Identity

- Name: NamPT
- Git email: montossc@gmail.com

## Communication Preferences

- Style: Detailed responses preferred
- Explanations welcome when helpful

## Workflow Preferences

- Git commits: Auto-commit after verification passes
- Beads updates: Ask first before modifying task state
- Auto-actions: Commit/push allowed; bead state changes require confirmation

## Technical Preferences

- Frontend: TypeScript, Svelte 5, SvelteKit
- Backend: Rust
- Package manager: pnpm
- Build: Vite (frontend), Cargo (backend)

## Rules to Always Follow

- Run `pnpm check` before any commit (svelte-check)
- Run `cargo check` in src-tauri/ for Rust changes
- Don't modify build/ or src-tauri/target/ directly (built output)
- Ask before adding new dependencies
- Ask before changing .opencode/ structure
- Never commit secrets or .env files
- Never force push to main
