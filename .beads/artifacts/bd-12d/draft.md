# Spike B: Canvas Commit Graph Engine

**Bead:** bd-12d
**Type:** feature
**Created:** 2026-03-16
**Status:** Draft

---

## Problem Statement

This task closes the highest-risk visual/technical foundation for mongit. Separate the graph into a deterministic data pipeline, a scalable renderer, and measurable interaction/performance behavior so later history features build on proven primitives rather than ad hoc rendering.

## Scope

### In-Scope

Build and validate the production baseline for mongit's Canvas 2D commit graph. This task exists because the graph is a core differentiator and a known technical risk. Scope includes graph data flow, lane assignment, renderer behavior, interaction primitives, and measurable performance evidence for large histories; it does not include advanced search or branch-collapse UX yet.

### Out-of-Scope

_To be determined during /refine_

## Proposed Solution

_High-level approach to be sketched during /refine_

## Requirements

1. Graph layout is deterministic for real repository history.
2. Canvas rendering stays responsive on large commit sets.
3. Interaction primitives exist for selection, inspection, and future navigation features.
4. Performance evidence is captured rather than assumed.

## Success Criteria

1. Graph layout is deterministic for real repository history.
2. Canvas rendering stays responsive on large commit sets.
3. Interaction primitives exist for selection, inspection, and future navigation features.
4. Performance evidence is captured rather than assumed.

## Technical Context

_To be filled during /refine — requires codebase research_

## Affected Files

_To be identified during /refine_

## Tasks

_To be decomposed during /refine_

## Risks

_To be assessed during /refine_

## Open Questions

Out of scope: advanced search, collapse/expand, or history-analysis features; this task is about a production baseline.

---

## Metadata

**Parent:** bd-134

## Notes from Creation

- Reasoning note: the project has already chosen Canvas 2D over DOM/WebGL for pragmatic reasons. The main unresolved question is no longer “what rendering technology?” but “what proves this approach is production-worthy at the target scale?”
