# Implement Qwen Model Integration - SRS

> Implement the core Qwen2/3 model loading and inference logic using candle.

## Scope

### In Scope

- [SCOPE-01] Define `QwenModelSpec` for managing model ID and revision.
- [SCOPE-02] Implement `QwenReranker` loading logic using `candle-transformers`.
- [SCOPE-03] Implement `Reranker` trait for `QwenReranker`.
- [SCOPE-04] Add support for `Qwen3-Reranker-0.6B`.

### Out of Scope

- [SCOPE-05] Implementing a custom tokenizer (use `tokenizers` or existing ` BertTokenizer` if compatible, though unlikely).
- [SCOPE-06] Quantization (deferred to second slice).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-01 | `QwenReranker` must load weights and config for Qwen2 architecture from Hugging Face. | FR-01 | SCOPE-01, SCOPE-02 | manual: Model loads without error |
| SRS-02 | `QwenReranker` must score query-document pairs by taking the last token logit or specific classification head. | FR-02 | SCOPE-03 | manual: Verification of scores |
| SRS-03 | Search service must replace `MockLlmReranker` with `QwenReranker` for `Llm` policy. | FR-03 | SCOPE-04 | manual: Search behavior change |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-04 | Reranking 10 documents must complete in under 1 second on a modern CPU. | NFR-01 | SCOPE-03 | command: just bench |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Automated tests for model loading and scoring.
- Manual verification of search results for specific queries.
