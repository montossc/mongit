# Desktop Architecture Recommendation: Tauri 2.0 + Rust + Svelte/React

**Date:** March 13, 2026
**Purpose:** Document the recommended tech stack for building a standalone Git client, with tradeoff analysis across all viable alternatives.

---

## Recommendation

**Tauri 2.0 + Rust backend + React or Svelte frontend**

Validated by GitButler (19.9k GitHub stars, 127+ releases, commercially backed) which uses exactly this stack: Tauri + Rust + Svelte. Languages: 65% Rust, 18.8% Svelte, 15.3% TypeScript.

---

## Tradeoff Table

| Dimension | Electron | Tauri 2.0 ✅ | Flutter Desktop | Qt6 (C++) | GPUI (Rust) | Native (3x) |
|-----------|----------|-------------|-----------------|-----------|-------------|-------------|
| **Binary size** | 150-300 MB | 5-20 MB | 20-50 MB | 15-40 MB | 5-15 MB | 5-15 MB/platform |
| **RAM baseline** | 250-500 MB | 60-130 MB | 50-120 MB | 30-80 MB | 20-60 MB | 20-60 MB/platform |
| **Startup time** | 3-8 sec | 0.5-2 sec | 0.5-2 sec | 0.5-1 sec | <0.5 sec | <0.5 sec |
| **Graph rendering** | WebGL (good) | WebGL (good) | Custom Canvas (moderate) | OpenGL (excellent) | GPU direct (excellent) | Metal/DX12 (excellent) |
| **Diff/editor views** | Monaco, CM6 (excellent) | Monaco, CM6 (excellent) | Custom only (poor) | Custom (moderate) | Custom Rust (excellent) | Platform editors |
| **OS integration** | Moderate | Strong (v2 plugins) | Moderate | Strong | Strong | Native |
| **Packaging** | electron-forge | Tauri bundler (MSI, dmg, AppImage) | flutter build | Qt Installer | Manual | Platform stores |
| **Auto-update** | electron-updater | tauri-plugin-updater (binary diffs) | DIY | DIY | DIY | Platform-specific |
| **Team velocity** | ████ (web teams) | ███░ (web + Rust) | ██░░ (Dart) | █░░░ (C++) | █░░░ (Rust-only) | ░░░░ (3 teams) |
| **Plugin potential** | ████ (npm) | ███░ (JS + Rust) | ██░░ (pub.dev) | ██░░ (C++) | █░░░ (Rust) | Platform native |
| **Maintainability** | ███░ (Chromium churn) | ████ (Rust memory safe) | ███░ (Google-backed) | ███░ (LGPL) | ██░░ (Zed-tied) | ███░ (divergence) |
| **Premium UI ceiling** | High | High | Moderate | High | Highest | Highest |
| **Cross-platform** | Win/Mac/Linux | Win/Mac/Linux + mobile | All 6 platforms | Win/Mac/Linux | macOS + Linux | Per-platform |
| **Hiring** | Easy (web devs) | Moderate (web + Rust) | Hard (Dart) | Hard (C++) | Very Hard | Very Hard |

---

## Why Tauri 2.0 Wins

### 1. Correct Performance Profile for Developer Tooling
Tauri separates concerns: OS webview renders UI (fast), Rust handles Git operations (blazing fast). `git2` Rust crate (libgit2 bindings) and future `gitoxide` give in-process git access. IPC in Tauri 2.0 supports raw byte payloads, eliminating the bottleneck for large diff transfers.

### 2. Complex Commit Graph
Render in WebGL/Canvas inside the webview — same strategy GitHub.com, Linear, and Miro use for complex graph UIs. Full GPU acceleration through the browser compositing pipeline.

### 3. Battle-Tested Editor Components
Monaco Editor (VS Code's engine) and CodeMirror 6 run perfectly in Tauri's webview for diff views, merge editors, and syntax highlighting with virtual scrolling.

### 4. Binary Size & Update Model
Tauri's updater ships binary diffs. Typical update: 2-5 MB vs Electron's 80-150 MB full replacement.

### 5. Security
Tauri 2.0 was externally audited by Radically Open Security (report publicly available). MIT license. Minimal attack surface compared to bundled Chromium.

### 6. Tauri 2.0 Specific Additions
- Mobile (iOS/Android) support for future expansion
- HMR on all platforms
- Plugin ecosystem: autostart, fs, global-shortcut, store, updater, shell, notifications, single-instance
- 80,000+ GitHub stars

---

## Why Not the Others

### Electron
Ships and maintains an entire copy of Chromium (~130MB) plus Node.js. Baseline 300-500 MB RAM before any work. GitHub Desktop demonstrates Electron's ceiling: adequate but never "premium."

### Flutter Desktop
No mature code editor widget (Monaco/CodeMirror are web-based). No production-grade diff view exists. Desktop plugin ecosystem is thin. Dart is ~0.5% in GitHub surveys — hiring constrained. Custom renderer (Impeller) is animation-focused, not text-heavy-UI-focused.

### Qt6
C++ required. LGPL commercial licensing complexity. QML requires separate expertise. Qt Group licensing has become more restrictive for startups.

### GPUI (Zed's Framework)
Technically most impressive: pure Rust, Metal/DirectX/Vulkan direct rendering, React-like component model. But it's embedded inside Zed's codebase, not an independent library. Windows support in progress. Building a Git client on GPUI means owning every UI primitive yourself. Worth watching for 2027+.

### Native (Swift + WinUI3 + GTK)
Tower (macOS-only) demonstrates native can achieve premium quality. But maintaining three codebases requires three expert teams and 3× CI/CD. Not viable for cross-platform without massive team.

---

## Risks & Mitigations

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| WebView2 inconsistency (Windows) | High | Medium | Automated cross-platform testing; avoid bleeding-edge CSS |
| Rust hiring difficulty | Medium | High | Team split: web devs own UI (TypeScript); 1-2 Rust engineers own Git backend |
| WebkitGTK on Linux is old | Medium | High | Tauri working on CEF for Linux; target Ubuntu 22.04+ |
| IPC overhead for large diffs | Medium | Low | Tauri 2.0 raw byte IPC; `convertFileSrc` for file-backed diffs |
| GPUI becomes mainstream | Low | Low | Isolate Rust business logic from UI layer; migration path exists |
| Complex syntax highlighting at scale | Medium | Medium | CodeMirror 6 virtual rendering handles 100k+ line files |

---

## Architecture Diagram

```
┌─────────────────────────────────────────────┐
│           Tauri Desktop Shell               │
│    (macOS: WKWebView, Win: WebView2)        │
├─────────────────────────────────────────────┤
│          Frontend (TypeScript)              │
│  React/Svelte + CodeMirror 6 + WebGL Graph │
│  ├── Commit Panel                           │
│  ├── Diff Viewer (2-pane)                   │
│  ├── Merge Editor (3-pane)                  │
│  ├── Commit Graph (WebGL/Canvas)            │
│  ├── Branch Manager                         │
│  └── Blame/History Views                    │
├──────────────── IPC ────────────────────────┤
│          Backend (Rust)                     │
│  ├── Git Operations (git2 + bundled git)    │
│  ├── File System Watcher (FSEvents)         │
│  ├── Change Tracking Engine                 │
│  ├── Diff Engine                            │
│  ├── Log Index / Search                     │
│  └── App State Manager                      │
└─────────────────────────────────────────────┘
```

---

## Decision Matrix (Quick Reference)

```
Web-only team, zero Rust?
  → Electron + TypeScript + React (accepted: size/memory penalty)

Best performance + modern DX + proven for Git clients?
  → Tauri 2.0 + Rust + Svelte/React  ✅ RECOMMENDED

Maximum native feel, macOS only?
  → Swift + AppKit/SwiftUI (accepted: macOS-only forever)

Pure-Rust team, 3+ years to invest?
  → GPUI (accepted: no web ecosystem, build everything)

Industrial-grade rendering, non-consumer?
  → Qt6 + C++/QML (accepted: licensing cost, C++ required)
```

---

## Sources

| Source | URL |
|--------|-----|
| GitButler (Tauri/Rust/Svelte proof) | https://github.com/gitbutlerapp/gitbutler |
| GitHub Desktop (Electron baseline) | https://github.com/desktop/desktop |
| Tauri 2.0 Release | https://tauri.app/blog/tauri-20/ |
| Tauri Security Audit | https://github.com/tauri-apps/tauri/blob/dev/audits/ |
| GPUI Framework | https://gpui.rs |
| Flutter Desktop docs | https://docs.flutter.dev/platform-integration/desktop |
| Qt Desktop | https://www.qt.io/product/qt-for-desktop |
