# Expose Artifact-Based Context Assembly And Emissions - SRS

## Summary

Epic: VF53suDXv
Goal: Assemble bounded local context over artifacts and expose simplified CLI and library outputs without a Document compatibility surface.

## Scope

### In Scope

- [SCOPE-01] Define artifact-based context-assembly request and response contracts.
- [SCOPE-02] Make retention and pruning inputs and outcomes explicit for bounded local context assembly.
- [SCOPE-03] Decouple visual, protocol, and latent outputs from the current CLI-shaped `SearchResponse`.
- [SCOPE-04] Expose supported CLI and library surfaces for the simplified artifact runtime.

### Out of Scope

- [SCOPE-05] Introduce a public graph-IR or Reactor API as part of the first cut.
- [SCOPE-06] Build the full multi-turn controller policy already scoped under the active runtime mission.
- [SCOPE-07] Preserve a supported `Document` compatibility surface alongside artifact-native runtime contracts.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must define explicit request and response contracts for artifact-based bounded context assembly. | SCOPE-01 | FR-01 | manual: API review |
| SRS-02 | The assembly contract must expose retention or pruning inputs and traceable outcomes rather than hiding them inside implicit prompt shaping. | SCOPE-02 | FR-02 | manual: contract review |
| SRS-03 | The runtime must decouple visual, protocol, and latent outputs from the current CLI-shaped `SearchResponse`. | SCOPE-03 | FR-03 | manual: type review |
| SRS-04 | Supported CLI and library entry points must expose the simplified artifact runtime without forcing consumers into graph or Reactor concepts. | SCOPE-04 | FR-04 | manual: embedder proof |
| SRS-05 | The simplified runtime must operate over `ContextArtifact` as the primary domain type without keeping a supported `Document` compatibility surface. | SCOPE-04 | FR-06 | manual: architecture review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Non-visual outputs, including latent output, must carry explicit performance expectations and supporting evidence. | SCOPE-03 | NFR-01 | manual: benchmark review |
| SRS-NFR-02 | The current single-turn hybrid experience must remain available when the simplified artifact runtime path is not selected. | SCOPE-04 | NFR-02 | manual: regression review |
| SRS-NFR-03 | Context-assembly inputs, outputs, budgets, and emissions must remain serializable and replayable for traces, tests, and evaluation fixtures. | SCOPE-03 | NFR-03 | manual: artifact review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
