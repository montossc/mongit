# MVP Core Git Client

**Bead:** bd-7gm
**Type:** epic
**Created:** 2026-03-16
**Status:** Draft

---

## Problem Statement

Purpose: deliver the first shippable daily-driver version of mongit for solo power developers. This epic should turn the existing backend strength and spike surfaces into a coherent end-user workflow. Future agents should optimize for end-to-end usability rather than isolated technical wins.

## Scope

### In-Scope

Deliver the first shippable daily-driver workflow for solo power developers. This epic should cover repo home/navigation, graph productization, local changes workspace with partial staging, commit authoring and sync, baseline conflict resolution, keyboard-first flows, and packaging so mongit becomes practically usable beyond spikes.

### Out-of-Scope

_To be determined during /refine_

## Proposed Solution

_High-level approach to be sketched during /refine_

## Requirements

1. User can open a repo and navigate core state confidently.
2. User can inspect history, curate local changes, and create/push commits.
3. Common conflicts are resolvable inside the app.
4. Core workflows are accessible from keyboard-first entry points.
5. MVP can be packaged and tested outside the author machine.

## Success Criteria

1. User can open a repo and navigate core state confidently.
2. User can inspect history, curate local changes, and create/push commits.
3. Common conflicts are resolvable inside the app.
4. Core workflows are accessible from keyboard-first entry points.
5. MVP can be packaged and tested outside the author machine.

## Technical Context

_To be filled during /refine — requires codebase research_

## Affected Files

_To be identified during /refine_

## Tasks

_To be decomposed during /refine_

## Risks

_To be assessed during /refine_

## Open Questions

Leverage the existing git resolver, branch operations backend, commit graph spike, and watcher work rather than rebuilding those concerns.

---

## Metadata

**Blocks:** bd-134

## Notes from Creation

- Background for future self: this epic is the first point where mongit stops being a strong technical foundation and starts becoming a coherent product. When making tradeoffs, prefer end-to-end usability over extra platform cleverness. A smaller but actually shippable vertical slice beats another isolated spike.
