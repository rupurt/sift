# Implement Qwen Model Integration - SDD

> Implement architectural integration for Qwen-based reranking.

## Architecture Overview

We will introduce a new adapter `QwenReranker` that implements the `Reranker` trait. It will use `candle-transformers::models::qwen2` for the model definition.

## Components

### `QwenReranker`
Loads the model configuration and weights into a `candle` `Model` instance. It handles tokenization of `(query, document)` pairs and executes the forward pass to extract relevance scores.

### `SearchServiceBuilder`
Updated to instantiate `QwenReranker` instead of `MockLlmReranker` when the `Llm` policy is selected.

## Data Flow

1. `SearchService` collects top-N candidates from primary retrievers.
2. `SearchService` passes `(query, candidates)` to `QwenReranker`.
3. `QwenReranker` tokenizes each `(query, candidate.text)` pair.
4. `QwenReranker` runs inference and extracts the score.
5. `SearchService` re-sorts candidates based on these scores and returns.

## Design Approach

### Model Specifics
- **Model:** `Qwen/Qwen3-Reranker-0.6B`
- **Architecture:** Qwen2 (supported by `candle-transformers`).
- **Scoring:** Rerankers typically use a classification head or the logit of a specific "Yes/No" token. We will follow the model's specific scoring instructions (likely a linear layer on top of the last hidden state).

## Deployment Strategy

- Add necessary dependencies to `Cargo.toml`.
- Implement core model loading in a new module or expand `src/dense.rs`.
- Integrate into `SearchService`.
