# Lift Structural Retrieval Documentation Fidelity - Product Requirements

## Problem Statement

The foundational docs mention structural fuzzy retrieval but still under-describe strategy selection, evaluation implications, and downstream direct-search adoption boundaries for embedders such as paddles.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make the foundational docs accurately describe the shipped structural retrieval stack. | Relevant foundational docs explain the retrievers, reranker behavior, and preset composition without stale omissions. | All affected foundational docs updated in one delivery slice |
| GOAL-02 | Make strategy selection and downstream adoption clearer for embedders and operators. | Docs explain when to use `path-hybrid` versus the page-index family and how downstream tools such as `paddles` should adopt richer direct retrieval. | Clear guidance present in README, CONFIGURATION, LIBRARY, and related architectural docs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developers and maintainers working directly in the `sift` repository. | Docs that match the shipped runtime so strategy and architecture discussions stay grounded. |
| Embedder | Downstream users embedding `sift`, including `paddles`. | Clear guidance on which plans to call and what retrieval behavior they get without changing the planning boundary. |

## Scope

### In Scope

- [SCOPE-01] Update the foundational docs that describe retrieval behavior, strategy presets, architecture, evaluation, and embedding guidance.
- [SCOPE-02] Add higher-fidelity explanations for path fuzzy retrieval, segment fuzzy retrieval, structural reranking, and downstream `paddles` adoption.
- [SCOPE-03] Align release and constitutional guidance where needed so repo-level process and principles reflect the shipped retrieval substrate.

### Out of Scope

- [SCOPE-04] New retrieval algorithms, runtime behavior changes, or planner features beyond documentation.
- [SCOPE-05] Downstream code changes inside `paddles`.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The foundational docs SHALL describe the shipped structural retrieval stack, including `path-fuzzy`, `segment-fuzzy`, and the current `PositionAwareReranker` behavior. | GOAL-01 | must | Prevents repo docs from lagging the shipped retrieval substrate. |
| FR-02 | The docs SHALL explain strategy and preset selection, including when `path-hybrid` is useful and what the page-index family adds on top of simpler presets. | GOAL-02 | must | Users need actionable strategy guidance rather than a raw registry dump. |
| FR-03 | The docs SHALL explain the downstream direct-search adoption seam for embedders such as `paddles` without implying that `sift` owns planner behavior. | GOAL-02 | must | Preserves the architecture boundary while making the integration path explicit. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Documentation changes SHALL stay consistent with the current code and public crate-root surface. | GOAL-01 | must | Avoids introducing a new doc/runtime mismatch while fixing the old one. |
| NFR-02 | The delivery SHALL keep the explanation concise enough for operators while still raising fidelity across the foundational set. | GOAL-01, GOAL-02 | should | Improves completeness without turning the docs into low-signal changelogs. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Foundational doc fidelity | Manual review plus repository grep against updated terminology and examples | Story-level verification artifacts linked during execution |
| Runtime/doc consistency | Existing structural retrieval tests still pass after the documentation slice | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current runtime surface for structural retrieval is stable enough to document as the supported baseline. | Excessive runtime churn would force another immediate doc rewrite. | Validate against current source and tests during execution. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Foundational docs may remain technically correct but still underspecify downstream usage. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Operators and embedders can read the foundational docs and understand the shipped structural retrieval stack, when to use the new presets, and how `paddles` should consume the richer direct-search surface.
<!-- END SUCCESS_CRITERIA -->
