# Refresh Foundational Structural Retrieval Docs - Software Design Description

> Update the foundational documents so they accurately explain structural fuzzy retrieval, strategy selection, and downstream direct-search adoption.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage is a documentation-alignment slice. It does not change runtime behavior; instead, it raises the fidelity of the foundational documents so they explain the shipped structural retrieval substrate, strategy-selection guidance, and downstream direct-search adoption path with fewer gaps and less ambiguity.

## Context & Boundaries

### In Scope

- foundational docs that describe retrieval behavior, architecture, configuration, evaluation, embedding, and release/process expectations
- explicit guidance for structural fuzzy retrieval and `paddles`-style embedding

### Out of Scope

- retrieval code changes
- planner changes
- downstream code edits in `paddles`

```
┌─────────────────────────────────────────────────────────┐
│         Foundational Documentation Alignment            │
│                                                         │
│  retrieval docs + config docs + embedder docs           │
│              -> consistent repo narrative               │
└─────────────────────────────────────────────────────────┘
            ↑                                   ↑
       shipped runtime                     downstream embedders
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `README.md` | internal | User-facing contract and entrypoint navigation | current repo |
| `WORLD.md`, `ARCHITECTURE.md`, `CONFIGURATION.md`, `EVALUATIONS.md`, `RESEARCH.md`, `LIBRARY.md`, `RELEASE.md`, `CONSTITUTION.md` | internal | Foundational references that must agree on the retrieval substrate and boundary | current repo |
| `src/search/domain.rs`, `src/search/adapters/fuzzy.rs`, `src/search/adapters/mod.rs` | internal | Source of truth for the shipped structural retrieval surface | current repo |
| `/home/alex/workspace/spoke-sh/paddles/SEARCH.md` | downstream reference | Existing boundary statement for `paddles` direct retrieval adoption | current workspace |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Scope of docs | Update the foundational set rather than only README/config | The mismatch is repo-wide, not limited to one user-facing file. |
| Boundary framing | Describe `paddles` adoption as richer direct retrieval via explicit plans | Matches the shipped contract and avoids implying planner ownership changed. |
| Fidelity target | Prefer higher-signal explanatory guidance over exhaustive file-by-file churn | Improves usability without bloating the docs. |

## Architecture

The documentation update follows the same dependency order as the repo's decision hierarchy:

1. principles and world model explain what structural retrieval means
2. architecture and configuration explain how the substrate is composed and selected
3. evaluation and library docs explain how to measure and adopt it
4. release/process docs ensure future changes keep that story synchronized

## Components

### Conceptual Docs

Purpose: explain the role of structural retrieval in the system's philosophy and architectural boundaries.

Likely files:
- `WORLD.md`
- `ARCHITECTURE.md`
- `CONSTITUTION.md`

### Operator Docs

Purpose: explain how a repo operator chooses strategies and interprets evaluation or release guidance.

Likely files:
- `README.md`
- `CONFIGURATION.md`
- `EVALUATIONS.md`
- `RELEASE.md`

### Embedder Docs

Purpose: explain how downstream consumers such as `paddles` adopt the richer direct-search surface through explicit plans.

Likely files:
- `LIBRARY.md`
- `RESEARCH.md`

## Interfaces

No new runtime interfaces are introduced. The main interface concern is documentation fidelity for:

- strategy names and preset composition
- public `SearchPlan` helpers
- the direct-search versus planner ownership boundary

## Data Flow

1. Inspect the shipped structural retrieval surface in code.
2. Identify foundational docs that underspecify or inconsistently describe that surface.
3. Update the docs in dependency order so conceptual framing, operator guidance, and embedder guidance agree.
4. Re-read the updated set and verify the board slice against the authored requirements.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Docs remain technically correct but still vague | Manual review finds strategy-selection or downstream-adoption ambiguity | Add clearer guidance and examples | Re-check the affected foundational set together |
| Docs imply a planner boundary change | Review against `paddles` boundary statement shows drift | Rewrite to emphasize explicit direct retrieval plans | Keep planner ownership wording explicit |
| Repo-level docs disagree with each other | Cross-read finds conflicting preset or reranker descriptions | Update the stale file in the same slice | Prefer the shipped code and crate-root surface as authority |
