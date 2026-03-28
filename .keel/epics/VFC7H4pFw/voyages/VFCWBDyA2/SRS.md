# Benchmark Autonomous Planning Against Baselines - SRS

## Summary

Epic: VFC7H4pFw
Goal: Measure autonomous planner quality, stop behavior, and turn efficiency against collapsed single-turn and planned-controller baselines.

## Scope

### In Scope

- [SCOPE-01] Autonomous evaluation against collapsed single-turn and planned-controller baselines.
- [SCOPE-02] Strategy-aware reporting for planner quality, stop behavior, and turn efficiency.
- [SCOPE-03] Stable autonomous evaluation artifacts suitable for replay and regression review.

### Out of Scope

- [SCOPE-04] Final answer synthesis evaluation.
- [SCOPE-05] CLI agent UX changes.
- [SCOPE-06] Hosted benchmark orchestration.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must evaluate autonomous planner runs against both collapsed single-turn search and the existing planned-controller path. | SCOPE-01 | FR-01 | manual: evaluation report review |
| SRS-02 | Evaluation reports must distinguish planner strategy, turn count, stop reason, retained-evidence efficiency, and quality/latency tradeoffs for autonomous runs. | SCOPE-02 | FR-04 | manual: metric review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Autonomous evaluation artifacts must remain stable enough to support replay and regression review. | SCOPE-03 | NFR-02 | manual: artifact review |
| SRS-NFR-02 | The autonomous evaluation workflow must stay runnable from the local repository environment without hosted infrastructure. | SCOPE-01, SCOPE-03 | NFR-02 | manual: local run proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
