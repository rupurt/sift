# Layered Search Strategy Foundation - Software Design Description

> Formalize layered search execution with named strategies, BM25 baseline/champion benchmarking, and PageIndex-inspired preset composition.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage reframes sift around a DDD + hexagonal search architecture.

Instead of a CLI command selecting one engine branch with embedded retrieval
logic, sift will expose a layered search plan:

1. query expansion,
2. retrieval,
3. fusion,
4. reranking.

The domain owns those concepts. Adapters implement concrete strategies such as
BM25, phrase/proximity retrieval, vector retrieval, text/JSON rendering, and
model runtimes.

This keeps two product promises aligned:

- search stays composable and benchmarkable;
- the codebase stays maintainable as new strategies and rerankers are added.

## Context & Boundaries

```
┌──────────────────────────────────────────────────────────────────────┐
│                            This Voyage                              │
│                                                                      │
│  CLI / Bench / Eval Adapters                                         │
│             │                                                        │
│             v                                                        │
│  Search Application Service                                          │
│             │                                                        │
│             v                                                        │
│  Search Domain                                                       │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ SearchPlan -> QueryExpansion -> RetrieverRuns -> Fusion ->     │  │
│  │ OptionalRerank -> FinalCandidates                              │  │
│  └────────────────────────────────────────────────────────────────┘  │
│      │             │              │               │                  │
│      v             v              v               v                  │
│  ExpanderPort  RetrieverPort   FuserPort    RerankerPort            │
│      │             │              │               │                  │
│      ├──── lexical adapters ──────┤               │                  │
│      ├──── vector adapters ───────┤               │                  │
│      └──── benchmark sinks / renderers / runtime adapters ─────────┘ │
└──────────────────────────────────────────────────────────────────────┘
          ↑                         ↑                        ↑
   Local file corpus          local model runtime      benchmark artifacts
```

### In Scope

- domain search-plan model,
- hexagonal ports for expansion, retrieval, fusion, reranking, and presets,
- named strategies and champion alias resolution,
- benchmark comparison against BM25 and champion,
- PageIndex-inspired preset composition from available layers.

### Out of Scope

- full PageIndex consumer-agent runtime,
- node-summary generation,
- persisted vector sidecars,
- mandatory LLM reranking,
- external retrieval services.

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing BM25 implementation | internal Rust module | Baseline lexical retriever adapter | current `search` module |
| Existing segment/vector implementation | internal Rust modules | Structure-aware vector retriever adapter | current `segment`, `vector`, and `dense` modules |
| Existing extractors | internal Rust modules | Prepare document and segment corpus state | current `extract` module |
| Benchmark/eval harness | internal Rust modules | Shared strategy comparison and evidence emission | current `bench` and `eval` modules |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Architectural style | DDD + hexagonal | Keeps search policy in the domain and concrete runtimes in adapters |
| Search abstraction | Layered search plan | Makes strategies composable and benchmarkable |
| Baseline strategy | `bm25` | Stable lexical baseline for all comparisons |
| Default fusion | RRF | Already proven in sift and robust across independent retrievers |
| Default reranking | `none` | Keeps default behavior predictable and low-cost |
| `hybrid` meaning | Champion alias | Preserves user-facing stability while allowing the best preset to evolve |
| PageIndex scope | PageIndex-inspired preset now, full tree-search later | Borrow the useful structure-aware composition without overcommitting to agentic runtime work |

## Architecture

### Domain

The core domain models:

- `SearchPlan`
- `QueryVariant`
- `RetrieverRun`
- `CandidateList`
- `FusionPolicy`
- `RerankPolicy`
- `StrategyPreset`
- `ChampionStrategy`

These types should not know about Clap, filesystem traversal details, Candle,
JSON formatting, or benchmark output files.

### Application

The application layer owns orchestration:

- resolve the requested strategy or `hybrid` alias,
- prepare shared corpus state,
- execute expansion/retrieval/fusion/reranking through ports,
- hand the final candidates to CLI or benchmark adapters.

### Adapters

Adapters implement concrete behavior:

- `Bm25Retriever`
- `PhraseRetriever`
- `SegmentVectorRetriever`
- `RrfFuser`
- `NoReranker`
- future `DenseReranker`
- future `LlmReranker`
- CLI renderer adapters
- benchmark/eval report adapters
- local model runtime adapters

### PageIndex-Inspired Composition

The first PageIndex-inspired preset should compose:

- structure-aware query expansion as needed,
- BM25 retrieval,
- phrase/proximity retrieval,
- structure-aware vector retrieval,
- RRF fusion,
- optional reranking set to `none` initially.

This captures the parallel-search and section-to-parent ideas from PageIndex
without yet implementing LLM-guided tree navigation.

## Components

- `SearchPlan`
  Purpose: canonical domain description of a strategy.
  Behavior: identifies the expansion, retrieval, fusion, and reranking stages
  to execute.

- `StrategyPresetRegistry`
  Purpose: resolve named presets and the `hybrid` champion alias.
  Behavior: maps CLI-visible names to domain `SearchPlan` definitions.

- `QueryExpander`
  Purpose: emit query variants for a strategy.
  Behavior: can return the original query only, or richer lexical variants for
  strategies that opt in.

- `Retriever`
  Purpose: produce ranked candidates from shared prepared corpus state.
  Behavior: lexical and semantic retrievers implement the same port.

- `ResultFuser`
  Purpose: merge ranked candidate lists into one ranked output.
  Behavior: default implementation is RRF with contributor provenance.

- `Reranker`
  Purpose: optionally reorder a fused shortlist.
  Behavior: default is `none`; later adapters can implement dense or LLM-based
  reranking.

- `BenchmarkComparator`
  Purpose: compare candidate presets against BM25 and champion.
  Behavior: runs shared search plans and emits delta-oriented reports.

## Interfaces

Planned internal interface shape:

- `StrategyPresetRegistry::resolve(name: &str) -> SearchPlan`
- `QueryExpander::expand(query: &str, plan: &SearchPlan) -> Vec<QueryVariant>`
- `Retriever::retrieve(query_variants: &[QueryVariant], corpus: &PreparedCorpus, limit: usize) -> CandidateList`
- `ResultFuser::fuse(inputs: &[CandidateList], limit: usize) -> CandidateList`
- `Reranker::rerank(query: &str, candidates: CandidateList, limit: usize) -> CandidateList`
- `BenchmarkComparator::compare(strategies: &[String], baseline: &str, champion: &str) -> StrategyComparisonReport`

Ports should live in the domain/application boundary. Concrete implementations
should sit in adapters.

## Data Flow

1. CLI, bench, or eval selects a named strategy.
2. The preset registry resolves that name to a `SearchPlan`.
3. If the name is `hybrid`, the registry resolves the configured champion
   preset.
4. Shared corpus preparation runs once.
5. The query expander emits zero or more query variants.
6. Selected retrievers run independently over the shared prepared corpus.
7. The fuser merges candidate lists, preserving provenance.
8. An optional reranker reorders the shortlist.
9. CLI renders results, or bench/eval emits comparison artifacts against BM25
   and champion.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Requested strategy name is unknown | preset registry lookup fails | fail fast with explicit available strategies | fix config or request a valid preset |
| `hybrid` champion alias points to a missing preset | registry validation fails | fail fast during startup/tests | correct champion mapping before release |
| One retriever fails while others are available | retriever port returns error | surface explicit failure in strict mode; optionally allow later degraded-mode policy | fix adapter or add explicit degraded-mode semantics later |
| Reranker adapter is configured but unavailable | reranker construction fails | fail fast with adapter/runtime context | switch to `none` or fix adapter configuration |
| Benchmark run omits baseline or champion metadata | report validation fails | reject artifact generation | fix benchmark invocation or plan configuration |
| Domain logic leaks adapter types | compile/test/code review failure | reject the design change | move boundary-specific code back behind ports |
