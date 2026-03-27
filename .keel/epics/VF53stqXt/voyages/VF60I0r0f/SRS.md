# Hard-Cut Core Search Domain To Context Artifacts - SRS

## Summary

Epic: VF53stqXt
Goal: Replace Document-centric modeling with ContextArtifact as the primary local search substrate and define shared local corpus semantics for artifact kinds.

## Scope

### In Scope

- [SCOPE-01] Introduce artifact-native domain records for the initial strictly local context kinds.
- [SCOPE-02] Hard-cut core search types from `Document`-centric modeling to `ContextArtifact`.
- [SCOPE-03] Define shared local corpus identifiers, segmentation rules, and storage semantics for artifact records.
- [SCOPE-04] Add provenance, freshness, and budget metadata to the artifact substrate for downstream retrieval and trace use.

### Out of Scope

- [SCOPE-05] Introduce a supported compatibility layer that preserves `Document` as a first-class parallel domain type.
- [SCOPE-06] Add remote, web, or MCP-backed artifact kinds to the first voyage.
- [SCOPE-07] Introduce graph-IR, Reactor, or other broad execution abstractions beyond what the hard cutover requires.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must introduce `ContextArtifact`-native records for the initial local context kinds, including file-backed content, project docs, environment facts, tool outputs, and agent-turn-style records. | SCOPE-01 | FR-01 | manual: domain review |
| SRS-02 | The core search substrate must use `ContextArtifact` as its primary domain type rather than keeping `Document` as a supported parallel primary model. | SCOPE-02 | FR-02 | manual: architecture review |
| SRS-03 | The system must define stable artifact identifiers, segmentation rules, and local corpus/storage semantics that support caching, deduplication, and retrieval reuse across local artifact kinds. | SCOPE-03 | FR-03 | manual: API review |
| SRS-04 | Artifact records must expose provenance, freshness, and budget metadata required by retrieval, pruning, and trace-oriented consumers. | SCOPE-04 | FR-04 | manual: type review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The hard cutover must not leave a long-lived supported `Document` compatibility surface behind after this voyage lands. | SCOPE-02 | NFR-01 | manual: code review |
| SRS-NFR-02 | The voyage must avoid introducing speculative graph or engine abstractions beyond what is necessary to prove the artifact substrate. | SCOPE-03 | NFR-02 | manual: architecture review |
| SRS-NFR-03 | Artifact records and metadata must remain serializable and inspectable for traces, tests, and downstream tooling. | SCOPE-04 | NFR-03 | manual: code review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
