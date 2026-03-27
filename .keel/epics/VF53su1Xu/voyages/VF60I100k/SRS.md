# Prove Local Acquisition Adapters On The Shared Pipeline - SRS

## Summary

Epic: VF53su1Xu
Goal: Route strictly local context sources through explicit acquisition adapters into one shared artifact normalization, caching, and indexing pipeline.

## Scope

### In Scope

- [SCOPE-01] Define explicit local acquisition-adapter contracts for files and project docs.
- [SCOPE-02] Define explicit local acquisition-adapter contracts for environment facts, tool outputs, and agent-turn-style records.
- [SCOPE-03] Route local adapter outputs through one shared normalization, caching, and indexing path into the artifact substrate.
- [SCOPE-04] Prove the adapter model with strictly local sources only.

### Out of Scope

- [SCOPE-05] Implement remote, web, or MCP-backed adapters in the first voyage.
- [SCOPE-06] Add hosted orchestration or a second source-specific pipeline.
- [SCOPE-07] Add answer-generation or ranking policy changes above acquisition.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system must expose explicit local acquisition-adapter contracts for file-backed content and project docs so those sources stop bypassing the adapter model. | SCOPE-01 | FR-01 | manual: adapter review |
| SRS-02 | The system must expose explicit local acquisition-adapter contracts for environment facts, tool outputs, and agent-turn-style records. | SCOPE-02 | FR-01 | manual: adapter review |
| SRS-03 | Local adapter outputs must flow through the same normalization, caching, and indexing path into the artifact substrate. | SCOPE-03 | FR-02 | manual: end-to-end proof |
| SRS-04 | The first voyage must prove the adapter model using strictly local sources only, with remote, web, and MCP-backed adapters deferred. | SCOPE-04 | FR-03 | manual: architecture review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The local adapter path must preserve the local-first, zero-daemon contract. | SCOPE-04 | NFR-01 | manual: architecture review |
| SRS-NFR-02 | The voyage must not introduce a second cache or indexing pipeline distinct from the shared preparation path. | SCOPE-03 | NFR-02 | manual: code review |
| SRS-NFR-03 | Adapter outputs, provenance, and failure states must remain inspectable for traces, tests, and future evaluation fixtures. | SCOPE-03 | NFR-03 | manual: artifact review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
