# Introduce Branching Autonomous Search Graph Runtime - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Formalize a graph episode contract that can represent branchable autonomous search as explicit requests, state, decisions, nodes, edges, frontier membership, and replayable traces. | board: VFD8KR44d |
| MG-02 | Deliver a bounded frontier runtime that can execute graph-shaped autonomous search over the existing hybrid retrieval and retained-evidence substrate. | board: VFD8NgvJl |
| MG-03 | Add heuristic and model-driven graph planner strategies that can fork, select, merge, and prune bounded search episodes under one contract. | board: VFD8ORnLV |
| MG-04 | Add graph-aware evaluation and supported invocation surfaces so graph search can be compared against the current linear planner and surfaced through the library and existing CLI agent entry points. | board: VFD8P8CO4 |

## Constraints

- MUST preserve the local-first, zero-daemon, no-external-database contract.
- MUST keep direct search, deterministic planned-controller search, and the shipped linear autonomous planner working while graph search is introduced additively.
- MUST formalize graph episode state and replay semantics before introducing heuristic or model-driven branching policy.
- MUST keep frontier mutations explicit and replayable; hidden branch creation, merge, or prune behavior is not acceptable.
- MUST reuse the existing hybrid retrieval, controller, and retained-artifact substrate rather than introducing a second graph-only search stack.
- MUST ship a heuristic graph planner baseline and keep model-driven graph planning behind the same explicit strategy contract.
- MUST be library-first; CLI support should layer onto the same graph runtime through `sift search --agent`.
- MUST keep final answer synthesis, hosted orchestration, distributed execution, and remote storage out of scope for this mission.
- MUST NOT require a persistent external turn store or a generic public graph DSL in this mission.

## Halting Rules

- DO NOT halt while any MG-* goal is missing an implemented code path or explicit verification evidence.
- HALT when Sift can represent, execute, and evaluate bounded graph-shaped autonomous search over local corpora through supported library and CLI agent surfaces.
- YIELD to human if bounded graph search requires a breaking change to the current crate-root autonomous contract instead of an additive extension.
- YIELD to human if persistent cross-session turn storage becomes necessary to make bounded graph search useful enough to ship.
