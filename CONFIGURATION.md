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

## Configuration Options (`sift.toml`)

### `[search]` Section

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `strategy` | String | `"hybrid"` | The named search strategy to use. |
| `limit` | Integer | `10` | The maximum number of results to return. |
| `shortlist` | Integer | `8` | The number of candidates to pass to the reranking phase. |

#### Available Strategies
- **`hybrid`** (default, alias for `page-index`): A robust strategy combining BM25 lexical search, Phrase matching, and Dense Vector semantic search, followed by Position-Aware reranking.
- **`bm25`**: Lexical search only. Fast and strictly keyword-based.
- **`legacy-hybrid`**: Simple BM25 + Vector fusion without phrase matching or structural bonuses.

---

### `[model]` Section

These settings control the local machine learning model used for semantic vector search.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `model_id` | String | `"sentence-transformers/all-MiniLM-L6-v2"` | HuggingFace model ID to download and run. |
| `model_revision` | String | `"main"` | The specific git revision/branch of the model. |
| `max_length` | Integer | `40` | Maximum sequence length (tokens) for embedding. |

---

## Ranking & Reranking

Currently, reranking logic is tied to the chosen **Search Strategy**.

### Position-Aware Reranking
Used by the `hybrid` strategy. It applies bonuses to the base retrieval score to prioritize structural matches:
- **Filename Bonus (+0.05):** Added if the query matches the file name.
- **Heading/Location Bonus (+0.02):** Added if the query matches a structural label (e.g., a PDF Page or HTML Heading).

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SIFT_CACHE` | Root directory for all sift state (defaults to standard OS cache dir). |
| `SIFT_BLOBS_CACHE` | Specific override for the blob store. |
| `SIFT_MANIFESTS_CACHE` | Specific override for the project manifests. |
| `SIFT_MODELS_CACHE` | Specific override for downloaded ML models. |
