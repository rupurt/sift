# Simplified Context Assembly Runtime And API - Product Requirements

> Once artifacts and adapters exist, the runtime should get smaller, not larger. This epic simplifies context assembly, emissions, and public integration around the artifact substrate.

## Problem Statement

The target architecture is accumulating engine, graph, and emission abstractions before the context substrate exists; Sift needs a smaller runtime surface that assembles bounded context over artifacts and exposes it cleanly to CLI and embedders.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Smaller context-assembly runtime | The runtime assembles bounded context over artifacts through a compact, inspectable contract | An artifact-based context-assembly path exists in code |
| GOAL-02 | Explicit budgets and emissions | Retention decisions and visual/protocol/latent outputs are typed and inspectable rather than CLI-only side effects | The runtime exposes bounded assembly and emission contracts |
| GOAL-03 | Stable public integration | CLI and library consumers can use the simplified substrate without adopting speculative graph APIs | Supported surfaces expose artifact-based assembly and outputs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Embedder | Rust developer integrating Sift into a larger local agent workflow. | Needs a stable API for assembling and emitting context over artifacts. |
| Runtime Maintainer | Developer evolving the current hybrid runtime. | Needs a simpler architecture that the active agentic runtime mission can build on. |
| CLI Power User | User exploring local search workflows beyond file-hit rendering. | Needs outputs and controls that are not trapped in `SearchResponse` as it exists today. |

## Scope

### In Scope

- [SCOPE-01] Define a simplified artifact-based context-assembly request/response contract.
- [SCOPE-02] Make retention and pruning policies explicit enough for bounded context assembly.
- [SCOPE-03] Decouple core execution results from the current CLI-shaped `SearchResponse` so the same substrate can emit visual, protocol, or latent outputs.
- [SCOPE-04] Expose supported CLI and library entry points that use the simplified surface.

### Out of Scope

- [SCOPE-05] Full graph-IR generalization or a public Reactor API beyond what the simplified substrate requires.
- [SCOPE-06] Inventing the final multi-turn controller policy already scoped under the active hybrid-and-agentic runtime mission.
- [SCOPE-07] Hosted answer-generation or service-first orchestration.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must assemble bounded context from artifacts through an explicit request/response contract rather than by implicit prompt shaping or file-only result construction. | GOAL-01 | must | A codex-style substrate needs first-class context assembly. |
| FR-02 | The system must expose explicit retention or pruning policy inputs and traceable outcomes for bounded context assembly. | GOAL-02 | must | Budget management is one of the key remaining gaps. |
| FR-03 | The core execution result must be decoupled from the current CLI-shaped `SearchResponse` so the same substrate can emit visual, protocol, or latent outputs. | GOAL-02 | must | The current response shape is too narrow for the pivot. |
| FR-04 | Supported public surfaces must expose the simplified artifact-based runtime without forcing embedders into speculative `SearchGraph` or Reactor concepts. | GOAL-03 | must | The architecture should simplify before it generalizes. |
| FR-05 | The active hybrid-and-agentic runtime path must be able to reuse this substrate rather than maintaining separate file and turn assembly logic. | GOAL-01 | must | This mission should reduce duplication across runtimes. |
| FR-06 | The simplified runtime must treat `ContextArtifact` as the primary input/output domain type rather than carrying a supported `Document` compatibility surface alongside it. | GOAL-01 | must | The hard-cutover decision should propagate all the way to the runtime API. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Non-visual emissions, including latent output, must have explicit performance constraints and verification evidence. | GOAL-02 | must | The accepted ADR requires performant latent emission. |
| NFR-02 | The simplified runtime must preserve the current single-turn hybrid experience when the new substrate path is not selected. | GOAL-03 | must | The pivot cannot regress shipped behavior. |
| NFR-03 | Context-assembly results, budgets, and emission records must remain serializable and replayable for traces, tests, and evaluation fixtures. | GOAL-02 | must | Inspectability is part of the product claim. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Runtime contract | Unit tests and API review | Story evidence for context-assembly and emission types |
| CLI/embedder surface | End-to-end local proofs | Story evidence showing supported invocation paths |
| Performance | Targeted benchmarks for latent/protocol/visual outputs | Story evidence for performance and bounded-budget behavior |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A smaller artifact-based runtime surface will reduce implementation drag relative to the current graph-heavy research direction. | The epic may need to retain more abstraction than desired. | Validate during voyage design and implementation proofs. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What is the smallest public API that supports both current CLI behavior and future agentic assembly needs? | Epic owner | Open |
| Should the simplified runtime expose a dedicated `assemble_context` surface, or evolve `search` to return a richer execution result? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Sift has a first supported artifact-based context-assembly path.
- [ ] The runtime can emit visual, protocol, and latent outputs without depending on CLI-shaped results.
- [ ] The public API is simpler than the current graph-heavy target and is reusable by the active runtime mission.
<!-- END SUCCESS_CRITERIA -->
