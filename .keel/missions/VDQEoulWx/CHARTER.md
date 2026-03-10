# Implement Qwen LLM Reranker for Search Quality - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Implement `QwenReranker` using `candle-transformers` and `Qwen2.5-0.5B-Instruct` | board: VDQEsPjsm |
| MG-02 | Integrate `QwenReranker` as a choice for the `RerankingPolicy` | board: VDQEsPjsm |
| MG-03 | Verify search quality improvement for complex queries like "Test function for cache" | manual: High relevance for domain.rs in search results |
| MG-04 | Ensure sub-second reranking latency for top-10 candidates on CPU | manual: Benchmarking reranking phase |

## Constraints

- Maintain "Single Binary" and "Local First" tenets.
- Ensure the model size is manageable (~1GB or less for the 0.5B model).
- Use `candle` for inference to remain consistent with existing architecture.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
