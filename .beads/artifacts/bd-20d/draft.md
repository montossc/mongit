# Changes Workspace and Partial Staging

**Bead:** bd-20d
**Type:** feature
**Created:** 2026-03-16
**Status:** Draft

---

## Problem Statement

This task delivers one of the project's clearest value propositions: shaping commit contents visually without patch-mode friction. The workspace should make file-, hunk-, and line-level intent explicit.

## Scope

### In-Scope

_To be determined during /refine_

### Out-of-Scope

_To be determined during /refine_

## Proposed Solution

_High-level approach to be sketched during /refine_

## Requirements

1. Modified files are accurately represented.
2. Hunk staging/unstaging is reliable.
3. Line-level flows exist for precision work.
4. Errors are clear when patch application fails.

## Success Criteria

1. Modified files are accurately represented.
2. Hunk staging/unstaging is reliable.
3. Line-level flows exist for precision work.
4. Errors are clear when patch application fails.

## Technical Context

_To be filled during /refine — requires codebase research_

## Affected Files

_To be identified during /refine_

## Tasks

_To be decomposed during /refine_

## Risks

_To be assessed during /refine_

## Open Questions

This workflow should feel safer and more legible than raw git add -p.

---

## Metadata

**Parent:** bd-7gm
**Blocks:** bd-6ew, bd-htm

## Notes from Creation

- Reasoning note: partial staging is one of the strongest product hooks for the target persona. Preserve correctness and clarity over flashy interaction design; users will notice subtle staging mistakes immediately.
