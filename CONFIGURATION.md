# Configuration Guide

Sift is designed to be zero-config by default, but it can be customized via a `sift.toml` file or environment variables.

## Configuration Locations

Sift looks for configuration in the following order, merging them together (local overrides system):

1.  **System-wide:** `/etc/sift.toml`
2.  **User-specific:** `~/.config/sift/sift.toml` (or platform equivalent)
3.  **Local Project:** `./sift.toml` (in the directory where you run `sift`)

You can view your effective configuration at any time by running:
```bash
sift config
```

---

## Ignoring Files (`.siftignore`)

Sift supports ignoring files using standard `gitignore` pattern syntax. This is useful for excluding large data directories, build artifacts, or sensitive files from being indexed and searched.

Similar to configuration, `.siftignore` files are loaded from multiple locations:

1.  **System-wide:** `/etc/siftignore`
2.  **User-specific:** `~/.config/sift/siftignore` (or platform equivalent)
3.  **Local Project:** `./.siftignore` (in the directory where you run `sift`)

**Example `.siftignore`:**
```ignore
# Ignore build directories
target/
dist/

# Ignore specific file types
*.log
*.tmp

# Ignore a specific large data folder
data/raw/
```

---

## Configuration Options (`sift.toml`)

### `[search]` Section

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `strategy` | String | `"page-index-hybrid"` | The named search strategy to use. |
| `limit` | Integer | `10` | The maximum number of results to return. |
| `shortlist` | Integer | `8` | The number of candidates to pass to the reranking phase. |

#### Available Strategies
- **`page-index-hybrid`** (default): Our champion strategy. Combines BM25, Phrase matching, and Vector search, followed by Position-Aware reranking.
- **`page-index-llm`**: Combines BM25, Phrase matching, and Vector search, followed by a **Qwen-based LLM reranker**.
- **`page-index`**: Lexical-focused strategy (inspired by qmd). Uses BM25 and Phrase matching with Position-Aware reranking (no vectors).
- **`bm25`**: Lexical search only. Fast and strictly keyword-based.
- **`vector`**: Semantic search only. Uses dense embeddings.
- **`legacy-hybrid`**: Simple BM25 + Vector fusion (no phrase matching or structural bonuses).

---

## Decoupled Execution (CLI Overrides)

You can override the components of any strategy directly from the CLI:

```bash
sift search --retrievers bm25,phrase --reranking position-aware "my query"
```

### Override Flags
- `--retrievers`: Comma-separated list (`bm25`, `phrase`, `vector`).
- `--fusion`: Currently only `rrf` is supported.
- `--reranking`: `none`, `position-aware`, or `llm`.
- `--model-id`: Override the embedding model ID.
- `--rerank-model-id`: Override the LLM rerank model ID.

---

### `[embedding]` Section
*Previously `[model]`, which is still supported for backward compatibility.*

These settings control the local machine learning model used for semantic vector search.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `model_id` | String | `"sentence-transformers/all-MiniLM-L6-v2"` | HuggingFace model ID to download and run. |
| `model_revision` | String | `"main"` | The specific git revision/branch of the model. |
| `max_length` | Integer | `40` | Maximum sequence length (tokens) for embedding. |

---

### `[rerank]` Section

These settings control the local LLM model used for semantic reranking.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `model_id` | String | `"Qwen/Qwen2.5-0.5B-Instruct"` | HuggingFace model ID for the reranker. |
| `model_revision` | String | `"main"` | The specific git revision/branch of the rerank model. |
| `max_length` | Integer | `512` | Maximum sequence length (tokens) for reranking. |

---

## Ranking & Reranking

### Position-Aware Reranking
Applies "soft bonuses" to prioritizing structural matches:
- **Filename Bonus (+0.05):** Added if the query matches the file name.
- **Heading/Location Bonus (+0.02):** Added if the query matches a structural label (e.g., a PDF Page or HTML Heading).

### LLM Reranking (Qwen)
Performs a deep semantic pass over the top candidates. It prompts a local LLM to evaluate the relevance of each document to the query, providing the highest accuracy but with more computational overhead.

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SIFT_CACHE` | Root directory for all sift state (defaults to standard OS cache dir). |
| `SIFT_BLOBS_CACHE` | Specific override for the blob store. |
| `SIFT_MANIFESTS_CACHE` | Specific override for the project manifests. |
| `SIFT_MODELS_CACHE` | Specific override for downloaded ML models. |
