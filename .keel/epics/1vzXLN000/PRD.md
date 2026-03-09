# Composable Search Strategy Architecture - Product Requirements

> Reframe sift as a composable search-strategy platform with layered query
> expansion, retrieval, fusion, and reranking, multiple named presets, a
> champion-backed `hybrid` alias, and benchmark discipline against BM25 and the
> current best known strategy.

## Problem Statement

The completed true-hybrid voyage proved that independent BM25 and vector
retrieval can work functionally in sift, but it also proved that one monolithic
hybrid engine is the wrong long-term product model.

The next product need is broader than latency recovery:

- search should be layered into query expansion, retrieval, fusion, and
  reranking;
- strategies should be combinable and benchmarkable as named presets;
- `bm25` should remain the stable baseline;
- `hybrid` should become a configurable alias to the best known preset;
- the architecture should be maintainable under DDD and hexagonal boundaries;
- and PageIndex-inspired strategies should fit naturally without forcing full
  agentic tree search into the first slice.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Model search as a layered strategy pipeline rather than hard-coded engine branches. | Design and implementation inspection | Query expansion, retrieval, fusion, and reranking are explicit first-class phases |
| GOAL-02 | Support multiple named search presets while keeping `hybrid` stable for users. | CLI and config inspection | `hybrid` resolves through a champion preset instead of hard-coded retrieval logic |
| GOAL-03 | Make strategy evaluation explicit and repeatable. | Benchmark/eval evidence | Every candidate strategy can be compared against BM25 and the champion |
| GOAL-04 | Keep the architecture maintainable as search strategies grow. | Design inspection + test structure | Domain logic sits behind ports/adapters and is reusable across CLI, bench, and eval |
| GOAL-05 | Position sift to express PageIndex-inspired strategy composition without overcommitting to full agentic tree search. | Design and preset inspection | A PageIndex-inspired preset fits the architecture cleanly |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Agentic Developer | Searches local repositories and document corpora from the terminal. | Strong default search plus specialized named strategies when needed |
| Coding Agent | Needs reliable local retrieval and explainable evidence without external infrastructure. | Composable retrieval and reranking layers it can steer programmatically |
| Maintainer / Evaluator | Owns the product contract and benchmark evidence. | Benchmarkable strategies with a stable baseline and champion model |

## Scope

### In Scope

- [SCOPE-01] Model search execution as layered query expansion, retrieval,
  fusion, and reranking phases.
- [SCOPE-02] Introduce named strategy presets and a configurable champion alias
  for `hybrid`.
- [SCOPE-03] Treat BM25, phrase/proximity retrieval, and vector retrieval as
  composable retrieval strategies.
- [SCOPE-04] Keep PageIndex-inspired strategy composition in scope for planning
  and preset design.
- [SCOPE-05] Make DDD and hexagonal boundaries explicit in the architecture.
- [SCOPE-06] Benchmark candidate strategies against BM25 and the champion.

### Out of Scope

- [SCOPE-07] Full PageIndex-style consumer-agent loops, summary generation, or
  LLM-guided tree navigation in the first voyage.
- [SCOPE-08] Making LLM reranking the default path.
- [SCOPE-09] External databases or daemonized background services.
- [SCOPE-10] Unrelated platform-wide refactors outside the search-strategy
  architecture problem.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Search execution must be modeled as layered query expansion, retrieval, fusion, and reranking phases. | GOAL-01, GOAL-04 | must | This is the core architectural shift required to make search strategies composable. |
| FR-02 | Sift must support multiple named strategy presets and resolve `hybrid` through a configurable champion preset. | GOAL-02, GOAL-03 | must | The user-facing default should stay stable while the underlying best strategy can evolve. |
| FR-03 | The retrieval layer must support BM25, phrase/proximity retrieval, and structure-aware vector retrieval as composable strategies. | GOAL-01, GOAL-05 | must | Vector retrieval is valuable but not sufficient as the only hybrid ingredient. |
| FR-04 | The strategy system must be able to express a PageIndex-inspired preset built from structure-aware retrieval, fusion, and optional reranking layers. | GOAL-05 | should | PageIndex is useful architectural prior art for sift's direction. |
| FR-05 | Bench and eval workflows must compare each named strategy against both the BM25 baseline and the current champion preset. | GOAL-03 | must | Product decisions need benchmark evidence tied to a stable baseline and a moving best-known strategy. |
| FR-06 | The reranking layer must be pluggable so dense or LLM rerankers can be added without rewriting retrievers or fusion logic. | GOAL-01, GOAL-04 | should | Reranking is useful, but it should remain a bounded optional layer. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The default strategy stack must remain single-binary, local-first, and free of external databases or resident services. | GOAL-04 | must | Preserves sift's core operating contract. |
| NFR-02 | Search domain logic must remain isolated behind ports/adapters so CLI, benchmark, eval, and runtime integrations do not own retrieval policy directly. | GOAL-04 | must | This is the maintainability requirement behind the DDD and hexagonal direction. |
| NFR-03 | Benchmark artifacts must record exact commands, strategy composition, model/runtime settings, corpus shape, git SHA, hardware assumptions, and measured outputs. | GOAL-03 | must | Prevents hidden performance claims and keeps strategy comparisons auditable. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove strategy layering and preset resolution through unit tests, CLI proofs,
  and design inspection.
- Prove benchmark comparison behavior with explicit bench/eval command outputs.
- Validate the DDD/hexagonal boundary through code inspection and test
  structure.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current BM25 and vector implementations can be lifted into retriever adapters rather than rewritten from scratch. | Delivery cost increases. | Validate during the first architecture story. |
| Phrase/proximity retrieval can materially complement BM25 and vector retrieval on target corpora. | The first named presets may need different lexical strategies. | Measure in the benchmark voyage. |
| A PageIndex-inspired preset is useful even before full agentic tree navigation exists. | The preset may need reframing as structure-aware hybrid rather than PageIndex-like. | Review during preset design and benchmark results. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the first reranker abstraction ship only with `none`, or should it also expose a concrete dense reranker immediately? | Engineering | Open |
| Does the `hybrid` champion alias need user-level configuration now, or is code-level champion selection enough for the first slice? | Product / Engineering | Open |
| How far should the PageIndex-inspired preset go before a later tree-search voyage is required? | Engineering | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Search is modeled as layered query expansion, retrieval, fusion, and
      reranking phases rather than hard-coded engine branches.
- [ ] Named strategy presets exist, and `hybrid` resolves through a champion
      preset rather than a fixed retrieval algorithm.
- [ ] Benchmark/eval flows compare candidate strategies against both BM25 and
      the champion with exact recorded evidence.
- [ ] The architecture expresses DDD and hexagonal boundaries cleanly enough to
      support PageIndex-inspired presets and future rerankers.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

### Opportunity Cost

The main opportunity cost is delaying a narrower latency-only recovery slice.
That is acceptable because the bigger product risk is hard-coding the wrong
strategy model.

### Dependencies

The following must hold:

- search plans and preset resolution need clean domain ownership;
- benchmark flows need shared strategy execution rather than bespoke command
  logic;
- the existing BM25 and vector paths need to be lifted behind retriever ports.

### Alternatives Considered

Alternatives considered:

- Keep optimizing the monolithic hybrid path.
  Rejected because it does not create a platform for future search strategies.
- Make vector retrieval the only semantic answer.
  Rejected because phrase/proximity retrieval covers different failure modes.
- Implement full PageIndex agentic search immediately.
  Deferred until sift has the composition and node-model foundations.

---

*This PRD was re-authored from bearing `1vzXLN000`. See `bearings/1vzXLN000/`
for the current research package.*
