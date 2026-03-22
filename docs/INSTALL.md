# Installing mongit

Build and install mongit on macOS from source.

## Prerequisites

| Tool           | Version   | Install                                      |
| -------------- | --------- | -------------------------------------------- |
| macOS          | 10.15+    | —                                            |
| Xcode CLI Tools | Latest   | `xcode-select --install`                     |
| Rust           | 1.94+     | [rustup.rs](https://rustup.rs)               |
| Node.js        | 20+       | [nodejs.org](https://nodejs.org)             |
| pnpm           | 10+       | `npm install -g pnpm` or [pnpm.io](https://pnpm.io) |

Verify your environment:

```bash
rustc --version    # 1.94.0 or later
node --version     # v20.x or later
pnpm --version     # 10.x or later
xcode-select -p    # /Library/Developer/CommandLineTools (or Xcode path)
```

## Build from Source

### 1. Clone the repository

```bash
git clone https://github.com/montossc/mongit.git
cd mongit
```

### 2. Install dependencies

```bash
pnpm install
```

This installs all frontend dependencies (Svelte, SvelteKit, CodeMirror, Tauri API). Rust dependencies are fetched automatically by Cargo during the build.

### 3. Build release artifacts

**Option A — Using the build script (recommended):**

```bash
./scripts/build-release.sh
```

This runs version sync checks, `svelte-check`, `cargo check`, then `pnpm tauri build`. To skip verification and build directly:

```bash
./scripts/build-release.sh --skip-checks
```

**Option B — Manual build:**

```bash
pnpm tauri build
```

### 4. Locate artifacts

After a successful build, artifacts are in:

```
src-tauri/target/release/bundle/
├── macos/mongit.app     # Application bundle (~6.6 MB)
└── dmg/mongit_0.1.0_aarch64.dmg   # Disk image (~3 MB)
```

## Install

### From DMG (recommended)

1. Open `mongit_0.1.0_aarch64.dmg` from the build output
2. Drag **mongit.app** to your **Applications** folder
3. Launch from Applications or Spotlight

### From .app directly

1. Copy `mongit.app` from `src-tauri/target/release/bundle/macos/` to `/Applications/`
2. Launch from Applications or Spotlight

### First launch (Gatekeeper)

Since the app is not code-signed (V1), macOS may block it:

1. Try opening the app — macOS shows "cannot be opened because the developer cannot be verified"
2. Go to **System Settings → Privacy & Security**
3. Scroll down to find the message about mongit being blocked
4. Click **Open Anyway**
5. Confirm in the dialog

Alternatively, remove the quarantine attribute:

```bash
xattr -cr /Applications/mongit.app
```

## Development Mode

For iterative development with hot-reload:

```bash
pnpm tauri dev
```

This starts the Vite dev server on port 1420 with HMR and launches the Tauri window. Changes to Svelte files reload instantly; Rust changes trigger a recompile.

## Verification

Run these checks before committing changes:

```bash
pnpm check             # svelte-check (0 errors required)
cd src-tauri && cargo check   # Rust typecheck
cd src-tauri && cargo test    # Rust tests
```

## Version Sync

All three config files must have matching versions:

- `package.json` → `"version"`
- `src-tauri/Cargo.toml` → `version`
- `src-tauri/tauri.conf.json` → `"version"`

Verify with:

```bash
./scripts/check-version-sync.sh
```

## Troubleshooting

### Build fails with "linker not found"

Install Xcode Command Line Tools:

```bash
xcode-select --install
```

### Build fails with Cargo errors

Ensure Rust is up to date:

```bash
rustup update stable
```

### "App is damaged and can't be opened"

Remove quarantine attribute:

```bash
xattr -cr /Applications/mongit.app
```

### Port 1420 already in use (dev mode)

Kill the existing process:

```bash
lsof -ti:1420 | xargs kill
```
