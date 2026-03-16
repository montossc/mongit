# Foundation Completion: finish the remaining technical spikes and baseline platform work that unblock the rest of mongit

**Bead:** bd-134
**Type:** epic
**Created:** 2026-03-16
**Status:** Draft

---

## Problem Statement

Purpose: close the remaining foundation spikes and stabilize the baseline architecture before broad feature expansion. This epic exists because the roadmap still lists commit-graph validation, diff/watcher integration, and design-system completion as prerequisites for confident MVP work. Future agents should treat this epic as platform work: improve primitives, prove risky assumptions, and document what good looks like for future slices.

## Scope

### In-Scope

_To be determined during /refine_

### Out-of-Scope

_To be determined during /refine_

## Proposed Solution

_High-level approach to be sketched during /refine_

## Requirements

1. Commit graph foundation is measurably validated on large histories.
2. Diff viewer and watcher backbone are reliable enough to power staging/conflict workflows.
3. Design tokens and base primitives are stable enough for repeated reuse.
4. Future agents can verify foundation health quickly with documented guardrails.

## Success Criteria

1. Commit graph foundation is measurably validated on large histories.
2. Diff viewer and watcher backbone are reliable enough to power staging/conflict workflows.
3. Design tokens and base primitives are stable enough for repeated reuse.
4. Future agents can verify foundation health quickly with documented guardrails.

## Technical Context

_To be filled during /refine — requires codebase research_

## Affected Files

_To be identified during /refine_

## Tasks

_To be decomposed during /refine_

## Risks

_To be assessed during /refine_

## Open Questions

Out of scope: full MVP polish, advanced history tooling, AI features, and platform expansion. The intent is to turn spikes into trustworthy building blocks.

---

## Metadata


## Notes from Creation

- Background for future self: current code already has a real Tauri/Svelte/Rust base, a working graph surface, watcher plumbing, and substantial backend Git operations. This epic exists to convert those promising pieces into validated platform primitives. The strategic reason is leverage: every later slice gets cheaper and less risky if graph, diff/watcher, and design tokens are trustworthy first.
