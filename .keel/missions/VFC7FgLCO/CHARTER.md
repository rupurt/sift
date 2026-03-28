# Introduce Local Autonomous Planning and Decomposition - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Formalize an autonomous planner contract that turns a root task, retained artifacts, and planner state into explicit next-turn proposals, planner decisions, and replayable stop semantics. | board: VFC7H4QFy |
| MG-02 | Deliver a bounded linear autonomous planner that can decompose a root task into self-generated retrieval turns over the existing hybrid and artifact substrate, starting with a heuristic strategy and remaining extensible to model-driven planning. | board: VFC7H4QFx |
| MG-03 | Add autonomous-planning evaluation and supported invocation surfaces so planner-driven search can be compared against both single-turn baselines and the current planned-controller path through library-first and CLI entry points. | board: VFC7H4pFw |

## Constraints

- MUST preserve the local-first, zero-daemon, no-external-database contract.
- MUST stop at bounded linear autonomous planning in this mission while structuring state and decisions so branching or graph search can be added later without rewriting the public contract.
- MUST keep the existing single-turn hybrid path and deterministic planned-controller path working while the autonomous planner is introduced.
- MUST make planner control explicit and replayable; hidden runtime magic is not an acceptable substitute for planner state, planner decisions, or stop reasons.
- MUST reuse the existing hybrid retrieval and context-artifact substrate rather than introducing a separate planner-only retrieval stack.
- MUST ship a heuristic baseline planner strategy and make strategy selection explicit enough that model-driven planning can be added behind the same contract in this mission.
- MUST be library-first; CLI support should layer on the same autonomous runtime rather than inventing a parallel path.
- MUST keep answer synthesis out of scope; this mission is about planning, decomposition, retrieval, and evidence management.
- MUST NOT expose a generic public graph IR or full branching tree-search API as part of this mission.

## Halting Rules

- DO NOT halt while any MG-* goal is missing an implemented code path or explicit verification evidence.
- HALT when Sift can autonomously decompose and execute bounded linear multi-turn local search through a supported library surface, compare it against current baselines, and expose the same behavior through the supported CLI flag.
- YIELD to human if branching or graph search appears necessary to make the first linear planner useful enough to ship.
- YIELD to human if model-driven planning requires relaxing the local-first or zero-friction contract.
