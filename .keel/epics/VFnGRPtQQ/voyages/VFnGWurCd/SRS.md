# Implement Frontier Coverage Search Semantics - SRS

## Summary

Epic: VFnGRPtQQ
Goal: Support truthful frontier, converging, and sealed search over mounted sectors with explicit coverage signaling and rolling lexical statistics.

## Scope

### In Scope

- [SCOPE-01] Persist or derive frontier ledger state from mounted sectors, dirty sectors, and breadcrumb progress so runtime search knows what portion of the corpus is currently represented.
- [SCOPE-02] Compute truthful `frontier`, `converging`, and `sealed` coverage states during direct-search preparation and rebuild progress.
- [SCOPE-03] Surface coverage mode and rolling sector statistics through direct-search progress snapshots and search responses.
- [SCOPE-04] Keep coverage claims conservative during resume, recovery, and partial rebuild scenarios.

### Out of Scope

- [SCOPE-05] Controller or autonomous runtime adoption.
- [SCOPE-06] New ranking algorithms or frontier-specific retrieval heuristics beyond truthful coverage signaling.
- [SCOPE-07] Replacing the sector or breadcrumb state introduced by earlier voyages.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Define `FrontierLedger` state with rolling sector counts, reuse counters, dirty-sector counts, and active rebuild metadata derived from sector and breadcrumb state. | SCOPE-01 | FR-03 | test |
| SRS-02 | Direct-search preparation computes `frontier`, `converging`, and `sealed` coverage states from mounted sectors and rebuild state, and updates that state as indexing advances. | SCOPE-02 | FR-03 | test |
| SRS-03 | Direct-search progress snapshots and search responses expose coverage mode plus enough sector statistics for callers to distinguish partial results from sealed corpus coverage. | SCOPE-03 | FR-03 | test |
| SRS-04 | Coverage signaling remains conservative during resume, recovery, and dirty-sector rebuilds and never reports `sealed` before all reachable dirty sectors converge. | SCOPE-04 | FR-03 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Coverage claims derive from the existing sector and breadcrumb authorities rather than a second independent file-state tracker. | SCOPE-01, SCOPE-04 | NFR-02 | manual |
| SRS-NFR-02 | Coverage visibility does not require an extra whole-corpus validation pass before first useful progress or results are surfaced. | SCOPE-02, SCOPE-03 | NFR-03 | command |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
