# Formalize Hybrid and Agentic Search Runtime - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Introduce a first-class turn-oriented search contract that makes agentic behavior expressible as data, including turn state, controller inputs, and explicit emission modes. | board: VF1VhxmqI |
| MG-02 | Deliver a reusable local multi-turn controller and execution path that decomposes complex queries, iterates retrieval against the current hybrid substrate, manages bounded context, and terminates deterministically on local corpora. | board: VF1Vhy2qJ |
| MG-03 | Add inspectable traces and evaluation coverage for agentic behavior, including turn progression, context-retention actions, and comparative quality/latency evidence for multi-hop retrieval. | board: VF1VhyGqK |

## Constraints

- MUST preserve the zero-friction local contract: no daemon, no external database, no mandatory remote service.
- MUST keep the current single-turn hybrid search path working while the agentic runtime is introduced.
- MUST make agentic control explicit and replayable; hidden mutable background state is not an acceptable substitute for turn records or traces.
- MUST reuse the existing hybrid retrieval substrate and cache pipeline rather than introducing a separate agent-only search stack.
- MUST keep answer generation separate from search orchestration; the search runtime should surface supporting evidence, traces, and emissions rather than collapse into a monolithic answering agent.
- MUST prefer local model execution by default and treat any external service dependency as optional, not foundational.
- SHOULD make plans authoritative data; runtime behavior should become less dependent on implicit overrides and more dependent on inspectable controller state.
- MUST NOT describe Sift as a formally agentic search tool in the shipped runtime until the turn loop, traceability, and evaluation coverage exist in code.

## Halting Rules

- DO NOT halt while any MG-* goal is missing an implemented code path or explicit verification evidence.
- HALT when Sift can execute both single-turn and multi-turn local search through a shared substrate with inspectable traces and documented evaluation coverage.
- YIELD to human if the public API, emission contract, or controller termination semantics require a product decision that cannot be resolved from ADRs and repository principles alone.
- YIELD to human if local-model limitations make the intended controller behavior impractical without relaxing the local-first or zero-friction constraints.
