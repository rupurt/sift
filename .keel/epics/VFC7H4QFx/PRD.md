# Bounded Linear Autonomous Planner - Product Requirements

> Sift has a deterministic controller, but it still depends on caller-supplied turns. The next product step is a bounded autonomous planner that can invent and sequence those turns itself while staying linear-first and locally inspectable.

## Problem Statement

Sift cannot autonomously decompose a root task into self-generated turns over the existing hybrid substrate; callers must currently supply every turn explicitly.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Heuristic baseline planner | Complex tasks can trigger self-generated retrieval turns without user-authored turn lists | A bounded heuristic planner runs end to end on local corpora |
| GOAL-02 | Swappable planning strategies | Heuristic and model-driven planning can be selected under one planner contract | The runtime supports explicit planner strategy selection |
| GOAL-03 | Bounded linear autonomy | The planner continues, prunes, and stops predictably while reusing the existing substrate | Planner execution is deterministic enough to test and replay |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Agent Workflow Builder | Developer wiring Sift into local coding or research agents. | Needs planner-driven retrieval without hand-authoring every turn. |
| Power User | User exploring complex local search tasks. | Needs better retrieval on multi-hop tasks while staying local-first. |

## Scope

### In Scope

- [SCOPE-01] A bounded linear planner that generates, executes, and sequences autonomous turns from a root task.
- [SCOPE-02] A heuristic baseline planner policy.
- [SCOPE-03] A model-driven planner policy slot under the same contract.
- [SCOPE-04] Retained-evidence reuse, pruning, and explicit stop conditions over the current hybrid and artifact substrate.

### Out of Scope

- [SCOPE-05] Branching or tree-search execution.
- [SCOPE-06] Hosted orchestration, daemons, or remote-first planning.
- [SCOPE-07] Final answer synthesis as the primary output.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must generate autonomous retrieval turns from a root task and current retained evidence without requiring caller-supplied turn lists. | GOAL-01 | must | This is the core missing planner capability. |
| FR-02 | The system must ship a heuristic planner baseline that can be used without model-dependent planning. | GOAL-01 | must | The mission explicitly wants a heuristic baseline strategy. |
| FR-03 | The system must support explicit planner strategy selection so a model-driven planner can reuse the same runtime contract. | GOAL-02 | must | Model-driven planning should be additive, not a parallel path. |
| FR-04 | The planner must remain bounded and linear, with explicit continuation, pruning, and stop behavior over the current retrieval and artifact substrate. | GOAL-03 | must | The mission stops at linear planning and must stay inspectable. |
| FR-05 | The autonomous planner must reuse the existing hybrid retrieval/controller substrate instead of introducing a planner-specific search stack. | GOAL-03 | must | Reuse keeps the mission aligned with prior runtime and artifact work. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The planner must preserve the zero-daemon, local-first contract even when model-driven planning is selected. | GOAL-02 | must | Planning strategy choice cannot undermine the product thesis. |
| NFR-02 | The current single-turn and planned-controller experiences must not regress when autonomous planning is not selected. | GOAL-03 | must | Planner work must remain additive until proven. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Heuristic planner | End-to-end local corpus proof and deterministic tests | Story-level verification artifacts linked during execution |
| Strategy selection | Unit/integration tests across heuristic and model-driven policy choices | Story-level verification artifacts linked during execution |
| Bounded behavior | Trace review and stop-condition tests | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A useful first autonomous planner can be linear and bounded rather than branching. | The planner may underperform enough to force a wider mission. | Validate in the evaluation epic. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How strong must the heuristic baseline be before model-driven planning is worth enabling by default? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Sift can autonomously generate and execute bounded linear retrieval turns from a root task on local corpora.
- [ ] A heuristic planner baseline and a model-driven planner strategy share one planner contract.
- [ ] The planner reuses the current hybrid and artifact substrate rather than a forked stack.
<!-- END SUCCESS_CRITERIA -->
