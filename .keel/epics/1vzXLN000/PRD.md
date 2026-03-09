# Recover Hybrid Retrieval Viability - Product Requirements

> The current structure-aware true-hybrid design is functionally correct but will
not reach the product latency/quality targets without changing the retrieval
architecture. The benchmark evidence likely justifies planning an explicit
indexing or precomputation step that was previously out of scope.

## Problem Statement

The completed true-hybrid voyage replaced shortlist reranking with BM25 plus
full-corpus vector retrieval over structure-aware segments. The resulting CLI
works end to end, but the recorded SciFact sample evidence shows two critical
problems:

- hybrid latency is orders of magnitude above the 200 ms target;
- hybrid quality did not beat BM25 on the sampled evaluation slice.

The repo now needs a recovery direction that preserves the local single-binary
product thesis while addressing the proven performance and quality gap.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Validate bearing recommendation in delivery flow | Adoption signal | Initial rollout complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Product/Delivery Owner | Coordinates planning and execution | Reliable strategic direction |

## Scope

### In Scope

- [SCOPE-01] Plan the next recovery slice for hybrid retrieval viability based
  on the completed benchmark evidence.

### Out of Scope

- [SCOPE-02] Unrelated platform-wide refactors outside the hybrid-retrieval
  recovery problem.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Implement the core user workflow identified in bearing research. | GOAL-01 | must | Converts research recommendation into executable product capability. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Ensure deterministic behavior and operational visibility for the delivered workflow. | GOAL-01 | must | Keeps delivery safe and auditable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Bearing findings reflect current user needs | Scope may need re-planning | Re-check feedback during first voyage |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which rollout constraints should gate broader adoption? | Product | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Identify which current constraints are now proven too expensive in
      practice and should be reconsidered.
- [ ] Recommend the next recovery direction with explicit tradeoffs across
      latency, quality, complexity, and product constraints.
<!-- END SUCCESS_CRITERIA -->


---

*This PRD was seeded from bearing `1vzXLN000`. See `bearings/1vzXLN000/` for original research.*
