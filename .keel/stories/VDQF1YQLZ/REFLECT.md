# Reflection - Integrate Qwen Reranker Into Search Service

The `QwenReranker` has been fully integrated into the `SearchService` orchestration. The `Llm` reranking policy now uses this model to provide a deep semantic second pass over retrieved candidates. We've also added a `page-index-qwen` strategy preset to allow easy testing and benchmarking of this new capability. Real-world tests confirm that this combination of lexical retrieval followed by LLM reranking provides the most accurate results for technical and semantic queries.
