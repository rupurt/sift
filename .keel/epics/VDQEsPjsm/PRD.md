# Qwen LLM Reranking Architecture - Product Requirements

> This epic introduces a real LLM-based reranking layer using the Qwen3-Reranker model series. This will significantly improve search quality by performing deep semantic comparison between the query and the top candidates retrieved by the primary engines (BM25, Vector).

## Problem Statement

The current `MockLlmReranker` provides no real semantic value. Fusion-based ranking (RRF) is excellent for general retrieval but often fails to distinguish between "keyword-similar" and "semantically-correct" results for complex queries. A true cross-encoder reranker can bridge this gap.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Semantic verification of candidates | NDCG@10 on SciFact | > 10% improvement over baseline hybrid |
| GOAL-02 | Efficient CPU inference | Reranking latency | < 500ms for 10 candidates on standard CPU |
| GOAL-03 | Easy strategy adoption | Reranker availability | `page-index-llm` uses real Qwen reranker |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Searching for specific code patterns | "Needles in a haystack" found via semantic understanding. |
| Power User | Asks natural language questions | Accurate answers backed by semantically verified documents. |

## Scope

### In Scope

- [SCOPE-01] Integration of `Qwen2` model architecture into `sift` using `candle`.
- [SCOPE-02] Implementation of `QwenReranker` struct and `Reranker` trait.
- [SCOPE-03] Configuration for `Qwen3-Reranker-0.6B`.
- [SCOPE-04] Integration with `SearchService` and `RerankingPolicy`.

### Out of Scope

- [SCOPE-05] Reranking more than top-20 candidates (too slow for CPU).
- [SCOPE-06] GPU acceleration for reranking (out of current scope).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Sift must download and cache Qwen weights from Hugging Face if not present. | GOAL-03 | must | Zero-friction setup. |
| FR-02 | The reranker must score query-document pairs and produce a normalized relevance score. | GOAL-01 | must | Core reranking function. |
| FR-03 | Search plans must allow selecting the LLM reranker via policy. | GOAL-03 | must | Composable strategy. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Reranking must be performed on the CPU by default. | GOAL-02 | must | Maintain single binary/local-first tenet. |
| NFR-02 | The reranker should support 4-bit/8-bit quantization to reduce memory usage. | GOAL-02 | should | Accessibility on lower-end hardware. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Use `just eval hybrid` to measure quality gains.
- Benchmark reranking phase separately using tracing.
- Manual verification of "Test function for cache" query.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Qwen3-Reranker-0.6B is sufficiently small for standard laptop memory. | OOM errors for users. | Monitor memory usage during inference. |
| Candle's Qwen2 implementation is compatible with the latest Qwen3-Reranker weights. | We might need to implement a custom model definition. | Test loading weights into Candle model. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What is the best quantization format for CPU speed (GGUF vs native Candle)? | Engineering | Investigating. |
| Should we rerank candidates or segments? | Product | Current architecture reranks candidates (documents). |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `QwenReranker` implemented and loading Qwen3-Reranker-0.6B.
- [ ] Search for "Test function for cache" returns `domain.rs` as top result.
- [ ] Reranking latency is sub-second for top-10 results.
<!-- END SUCCESS_CRITERIA -->
