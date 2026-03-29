# Graph Evaluation and Supported Surface - Product Requirements

> Sift can evaluate and expose linear autonomous planning, but it still lacks
> graph-aware evaluation and a supported surface for bounded graph search
> through the library and existing CLI agent path.

## Problem Statement

Sift cannot benchmark branching graph search against linear autonomy or expose a
supported graph-aware library and CLI surface through the existing agent entry
points.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Graph evaluation | Graph search can be compared against linear autonomy, planned-controller, and collapsed single-turn baselines | Graph-aware evaluation fixtures and reports exist |
| GOAL-02 | Supported library surface | Embedders can select bounded graph search without relying on internal-only seams | A supported library entry point exists for graph mode |
| GOAL-03 | Supported CLI surface | Users can access graph search through the existing CLI agent entry point | `sift search --agent` can invoke graph mode explicitly |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Library Integrator | Developer embedding Sift into a coding or research workflow. | Needs a supported graph-search contract rather than internal-only seams. |
| Operator | User evaluating whether graph search beats the current linear planner on real tasks. | Needs credible comparative evidence and a supported CLI path. |

## Scope

### In Scope

- [SCOPE-01] Graph-aware evaluation fixtures, metrics, and comparisons against current baselines.
- [SCOPE-02] A supported library surface for bounded graph search.
- [SCOPE-03] CLI support through the existing `sift search --agent` path.

### Out of Scope

- [SCOPE-04] A separate interactive graph shell.
- [SCOPE-05] Hosted evaluation infrastructure or remote telemetry collection.
- [SCOPE-06] Generic answer-synthesis UX above retrieval.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must benchmark graph search against the current linear autonomous planner, planned-controller fixtures, and collapsed single-turn baselines. | GOAL-01 | must | The new layer needs evidence that it is worth shipping. |
| FR-02 | The system must report graph-aware metrics such as branch success, branch efficiency, or frontier expansion cost in a replayable evaluation artifact. | GOAL-01 | must | Graph search needs graph-specific evaluation rather than linear-only metrics. |
| FR-03 | The system must expose a supported library-facing graph search surface that reuses the shipped autonomous runtime patterns. | GOAL-02 | must | Graph search should be embedder-ready, not just repository-internal. |
| FR-04 | The system must expose graph search through the existing `sift search --agent` entry point instead of creating a second autonomous CLI command. | GOAL-03 | must | CLI support should stay additive and avoid command sprawl. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Graph evaluation and surface work must not regress the current linear autonomous or direct search paths. | GOAL-02, GOAL-03 | must | Surface and evaluation additions should remain additive. |
| NFR-02 | Evaluation artifacts must remain deterministic enough for regression review and side-by-side graph versus linear comparison. | GOAL-01 | must | Comparisons are only useful if they are stable enough to track. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Comparative evaluation | Fixture-based graph versus linear versus baseline reports | Story-level verification artifacts linked during execution |
| Library surface | Integration tests and documentation review | Story-level verification artifacts linked during execution |
| CLI surface | CLI tests and end-to-end agent search proofs | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing `sift search --agent` entry point is the right additive home for graph search selection. | CLI surface work may need a different invocation design. | Validate during voyage design and tests. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which graph metrics are the minimum credible set for operator decision-making: branch success, frontier cost, merge yield, or all three? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Graph search can be compared credibly against the current linear planner and single-turn baselines.
- [ ] A supported library surface exists for bounded graph search.
- [ ] The existing CLI agent entry point can invoke graph mode explicitly.
<!-- END SUCCESS_CRITERIA -->
