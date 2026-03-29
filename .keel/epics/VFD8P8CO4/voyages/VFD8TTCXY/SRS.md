# Benchmark Graph Search Against Linear Baselines - SRS

## Summary

Epic: VFD8P8CO4
Goal: Measure graph planner performance against linear autonomy, planned-controller, and single-turn baselines.

## Scope

### In Scope

- [SCOPE-01] Graph-aware evaluation fixtures and reports that compare graph search against linear autonomous, planned-controller, and collapsed single-turn baselines.
- [SCOPE-02] Graph-specific metrics such as branch success, frontier expansion cost, merge or prune counts, or branch efficiency.
- [SCOPE-03] Replayable evaluation artifacts suitable for regression review.

### Out of Scope

- [SCOPE-04] CLI surface or library API rollout.
- [SCOPE-05] Hosted evaluation services.
- [SCOPE-06] Final-answer quality judging beyond retrieval and evidence behavior.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must evaluate graph search against linear autonomous planning, planned-controller fixtures, and collapsed single-turn baselines on the same task sets. | SCOPE-01 | FR-01 | manual: comparative report review |
| SRS-02 | Evaluation reports must expose graph-specific metrics such as branch success, frontier expansion cost, or merge and prune outcomes alongside current success and recall metrics. | SCOPE-02 | FR-02 | manual: metrics review |
| SRS-03 | Graph evaluation artifacts must remain replayable and suitable for regression comparison across graph and linear planner revisions. | SCOPE-03 | FR-02 | manual: artifact review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Graph evaluation must not regress the current linear evaluation paths when graph mode is not selected. | SCOPE-01 | NFR-01 | manual: regression review |
| SRS-NFR-02 | Graph evaluation artifacts must remain deterministic enough for meaningful graph-versus-linear regression review. | SCOPE-03 | NFR-02 | manual: deterministic report review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
