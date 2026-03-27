# Acquisition Adapters And Shared Artifact Pipeline - Product Requirements

> A context substrate is only real if evidence enters it through explicit adapters instead of ad hoc prompt assembly. This epic formalizes how Sift acquires context and feeds it into one local-first pipeline.

## Problem Statement

Context sources are implicit and uneven today; Sift needs explicit acquisition adapters that normalize repo, environment, tool, agent, and optional remote evidence into one local-first artifact pipeline.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Explicit acquisition adapters | Context sources are modeled as supported adapters rather than hidden special cases | Adapter contracts exist for the initial source classes |
| GOAL-02 | Shared artifact pipeline | Acquired evidence flows through one normalization, caching, and indexing path | More than one source class reuses the same preparation pipeline |
| GOAL-03 | Local-first optionality | Optional remote sources remain adapters over the local substrate rather than a second architecture | Remote or MCP-backed evidence is represented with the same artifact pipeline and policy metadata |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Context Integrator | Maintainer wiring new context sources into Sift. | Needs one adapter pattern rather than one-off ingestion logic. |
| Agent Workflow Builder | Developer expecting Sift to gather repo, environment, and runtime evidence together. | Needs consistent acquisition semantics across source types. |
| Security-Conscious Operator | User who wants optional networked context without abandoning local-first guarantees. | Needs explicit policy boundaries around remote acquisition. |

## Scope

### In Scope

- [SCOPE-01] Define acquisition-adapter contracts for local project docs, working-tree files, environment context, tool/subagent outputs, and agent-turn-style records.
- [SCOPE-02] Route acquired evidence through a shared normalization, caching, and indexing pipeline into the artifact substrate.
- [SCOPE-03] Define sequencing and policy boundaries so the first voyage stays strictly local-only.
- [SCOPE-04] Define policy and provenance handling for optional remote, web, or MCP-style adapters without making them foundational.

### Out of Scope

- [SCOPE-05] Remote, web, or MCP-backed adapter delivery in the first voyage.
- [SCOPE-06] Hosted orchestration or service-first connector infrastructure.
- [SCOPE-07] Learned ranking or answer-generation policies above the acquisition layer.
- [SCOPE-08] A separate indexing stack for each source type.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must expose explicit acquisition-adapter contracts for the initial codex-style source classes: project docs, environment context, tool/subagent outputs, agent turns or logs, and file-backed content. | GOAL-01 | must | These are the missing context surfaces relative to the pivot. |
| FR-02 | The system must route acquired evidence through the same normalization, caching, and indexing path into the shared artifact substrate rather than maintaining per-source ingestion logic. | GOAL-02 | must | One substrate is simpler and more testable than many bespoke paths. |
| FR-03 | The first voyage for this epic must prove the acquisition model using strictly local adapters before any remote, web, or MCP-backed adapter is implemented. | GOAL-03 | must | The first slice should reduce risk and architectural noise. |
| FR-04 | Optional remote, web, or MCP-backed sources must enter the system as later adapters over the same artifact pipeline with explicit source and policy metadata. | GOAL-03 | must | Optional remote evidence should not create a second architecture. |
| FR-05 | Adapter failures and degraded modes must remain explicit and traceable rather than silently dropping context. | GOAL-02 | must | Hidden acquisition failures will make agentic retrieval untrustworthy. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Acquisition adapters must preserve the local-first, zero-daemon contract, with remote sources disabled or optional by default. | GOAL-03 | must | This is a constitution-level requirement. |
| NFR-02 | The adapter layer must avoid introducing a second cache or index pipeline distinct from the existing preparation path. | GOAL-02 | must | Architectural simplification depends on one pipeline. |
| NFR-03 | Adapter outputs and provenance must remain inspectable for traces, tests, and future evaluation fixtures. | GOAL-01 | must | Context gathering must be replayable, not magical. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Adapter contracts | Unit and integration tests | Story evidence for adapter behavior and degraded-mode handling |
| Shared pipeline | End-to-end proof across multiple source classes | Story evidence showing one normalization/indexing path |
| Policy boundaries | Manual review and targeted proofs | Story evidence for local-first defaults and explicit remote gating |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Existing cache and extraction seams can be extended to non-file sources without pathological complexity. | The epic may need a narrower first slice or migration plan. | Validate during voyage design. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which later-slice remote or MCP-style source, if any, is worth supporting first without distorting the local-first design? | Epic owner | Open |
| How should environment facts be materialized as artifacts without over-noising retrieval? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Sift has explicit acquisition adapters for the first codex-style source classes.
- [ ] Multiple source classes reuse the same artifact preparation pipeline.
- [ ] The first voyage proves the model with strictly local adapters only.
- [ ] Optional remote evidence remains an adapter over the local substrate, not a separate stack.
<!-- END SUCCESS_CRITERIA -->
