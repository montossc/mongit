# Advanced Branch Operations

**Bead:** bd-2jk
**Type:** feature
**Created:** 2026-03-16
**Status:** Draft

---

## Problem Statement

This task expands branch operations from the current MVP baseline into professional workflows. Merge, rebase, cherry-pick, and guided interactive rebase need strong guardrails because they are powerful but high-risk.

## Scope

### In-Scope

Implement merge, rebase, cherry-pick, and a guided interactive rebase baseline with appropriate previews and safeguards. This task is essential for the product thesis of JetBrains-grade depth in a standalone client.

### Out-of-Scope

_To be determined during /refine_

## Proposed Solution

_High-level approach to be sketched during /refine_

## Requirements

1. Advanced operations exist with meaningful preflight checks.
2. Failure modes are understandable.
3. Interactive rebase has a usable guided surface.

## Success Criteria

1. Advanced operations exist with meaningful preflight checks.
2. Failure modes are understandable.
3. Interactive rebase has a usable guided surface.

## Technical Context

_To be filled during /refine — requires codebase research_

## Affected Files

_To be identified during /refine_

## Tasks

_To be decomposed during /refine_

## Risks

_To be assessed during /refine_

## Open Questions

Build on the existing git resolver and command/error model instead of forking logic.

---

## Metadata

**Parent:** bd-193
**Blocks:** bd-9a4

## Notes from Creation

- Reasoning note: advanced branch operations are where user trust can be won or lost. Previews, guardrails, and error specificity matter as much as the happy path.
