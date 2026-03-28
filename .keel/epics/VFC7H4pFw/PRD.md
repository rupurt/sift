# Autonomous Planning Evaluation and Surface - Product Requirements

> Autonomous planning is not real until users can invoke it and compare it against the current alternatives. This epic makes planner-driven search measurable and reachable through supported surfaces.

## Problem Statement

Sift lacks autonomous-planning evaluation, strategy comparisons, and supported invocation surfaces for planner-driven search beyond deterministic fixture replay.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Autonomous evaluation | Autonomous planning quality, stopping, and turn efficiency are measurable in-repo | Reports compare planner-driven runs against existing baselines |
| GOAL-02 | Library-first surface | Embedders can invoke autonomous planning through a supported public surface | A stable library entry point exists for planner-driven search |
| GOAL-03 | CLI reachability | Users can trigger autonomous planning from the shipped executable | `sift search --agent` reuses the supported autonomous runtime |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Evaluator | Developer tuning or comparing planner strategies. | Needs reproducible evidence for decomposition and stop behavior. |
| CLI User | User exploring planner-driven search from the executable. | Needs a supported flag that reuses the same library runtime. |
| Library Embedder | Developer integrating planner-driven retrieval into local tooling. | Needs a supported public autonomous entry point. |

## Scope

### In Scope

- [SCOPE-01] Autonomous-planning evaluation against collapsed single-turn and current planned-controller baselines.
- [SCOPE-02] Supported library-first autonomous-planning surface.
- [SCOPE-03] CLI support through `sift search --agent`.
- [SCOPE-04] Strategy-aware reporting that distinguishes heuristic and model-driven planner runs.

### Out of Scope

- [SCOPE-05] Generic interactive agent shells or hosted orchestration.
- [SCOPE-06] Branching/tree-search CLI UX.
- [SCOPE-07] Grounded answer synthesis evaluation.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must evaluate autonomous planner runs against both collapsed single-turn search and the existing planned-controller path. | GOAL-01 | must | The new planner should be judged against both current alternatives. |
| FR-02 | The system must expose a supported library-first autonomous entry point. | GOAL-02 | must | The mission is library-first by explicit user direction. |
| FR-03 | The executable must support planner-driven search through `sift search --agent`. | GOAL-03 | must | CLI support is in scope once the library surface is real. |
| FR-04 | Reports and traces must record which planner strategy produced an autonomous run. | GOAL-01, GOAL-02 | must | Strategy comparison is part of the mission scope. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The library and CLI autonomous surfaces must share one runtime contract rather than drifting into parallel implementations. | GOAL-02, GOAL-03 | must | Shared semantics keep planner behavior inspectable and maintainable. |
| NFR-02 | Non-agent search behavior and current evaluation commands must remain intact when autonomous planning is not selected. | GOAL-01 | must | Planner rollout must stay additive and low-risk. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Evaluation reports | Fixture/task reports plus comparison review | Story-level verification artifacts linked during execution |
| Library surface | API proof and integration tests | Story-level verification artifacts linked during execution |
| CLI surface | `sift search --agent` proof on local corpora | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Retrieval- and evidence-focused autonomous evaluation is enough for the first mission without adding answer synthesis. | Evaluation may miss user-visible planner weaknesses. | Revisit after the first planner reports exist. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which planner-efficiency metrics best explain when model-driven planning is worth its extra cost? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Autonomous planner runs can be compared against both single-turn and planned-controller baselines.
- [ ] Embedders can invoke autonomous planning through a supported public surface.
- [ ] Users can trigger the same planner runtime through `sift search --agent`.
<!-- END SUCCESS_CRITERIA -->
