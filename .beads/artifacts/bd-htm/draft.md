# Spike D: Diff Viewer and File Watching Backbone

**Bead:** bd-htm
**Type:** feature
**Created:** 2026-03-16
**Status:** Draft

---

## Problem Statement

This task establishes the backbone for repository-change awareness. The diff surface and watcher/event path must be trustworthy because staging, conflict resolution, and ambient status all depend on them.

## Scope

### In-Scope

_To be determined during /refine_

### Out-of-Scope

_To be determined during /refine_

## Proposed Solution

_High-level approach to be sketched during /refine_

## Requirements

1. Diff component renders repository change data reliably.
2. Watcher lifecycle is stable and debounced correctly.
3. Frontend refresh behavior is scoped and predictable.
4. File changes do not require blunt full-app reload patterns.

## Success Criteria

1. Diff component renders repository change data reliably.
2. Watcher lifecycle is stable and debounced correctly.
3. Frontend refresh behavior is scoped and predictable.
4. File changes do not require blunt full-app reload patterns.

## Technical Context

_To be filled during /refine — requires codebase research_

## Affected Files

_To be identified during /refine_

## Tasks

_To be decomposed during /refine_

## Risks

_To be assessed during /refine_

## Open Questions

Treat watcher correctness and lifecycle management as first-order concerns, not incidental plumbing.

---

## Metadata

**Parent:** bd-134

## Notes from Creation

- Reasoning note: watcher correctness is product correctness. If file changes are missed or surfaced inconsistently, staging/conflict UI will feel untrustworthy no matter how polished the components look.
