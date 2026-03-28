# Configuration Guide

Sift is zero-config by default, but you can customize the executable and the
underlying retrieval substrate through `sift.toml`, CLI overrides, and
environment variables.

Today configuration is focused on retrieval behavior and model selection. The
turn-aware controller surface is library-first; there is not yet a dedicated
`[agentic]` section in `sift.toml`.

## Configuration Locations

Sift loads configuration in this order, later files overriding earlier ones:

1. System-wide: `/etc/sift.toml`
2. User-specific: `~/.config/sift/sift.toml` (or platform equivalent)
3. Local project: `./sift.toml`

You can inspect the merged effective configuration with:

```bash
sift config
```

## Ignore Files (`.siftignore`)

Sift supports ignore rules using `gitignore`-style syntax. Ignore files are
loaded from:

1. System-wide: `/etc/siftignore`
2. User-specific: `~/.config/sift/siftignore`
3. Local project: `./.siftignore`

Example:

```ignore
target/
dist/
*.log
data/raw/
```

## `sift.toml` Surface

### `[search]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `strategy` | String | `"hybrid"` | Default search strategy for the executable and library builder defaults. |
| `limit` | Integer | `10` | Maximum number of results returned. |
| `shortlist` | Integer | `8` | Number of fused candidates passed to reranking. |

`hybrid` is the default runtime strategy. `page-index-hybrid` is the richer
benchmark champion preset used in comparative evaluation docs.

### Built-in Strategies

The current registry exposes the following named plans:

- `lexical`: BM25-only lexical search.
- `bm25`: Same execution plan as `lexical`, but reported under the `bm25` name.
- `vector`: Dense-vector-only semantic search.
- `hybrid`: BM25 + vector fusion with no reranking.
- `legacy-hybrid`: `page-index-hybrid` retrieval stack without expansion.
- `page-index-hybrid`: SPLADE expansion + BM25 + phrase + vector + position-aware reranking.
- `page-index-llm`: HyDE expansion + BM25 + phrase + vector + Qwen reranking.
- `page-index-qwen`: No expansion + BM25 + phrase + vector + Qwen reranking.
- `page-index-splade`: SPLADE expansion + BM25 + phrase + vector + position-aware reranking.
- `page-index-classified`: Classified expansion + BM25 + phrase + vector + position-aware reranking.
- `page-index-jina`: SPLADE expansion + BM25 + phrase + vector + Jina reranking.
- `page-index-gemma`: SPLADE expansion + BM25 + phrase + vector + Gemma reranking.

### Strategy Matrix

| Strategy | Expansion | Retrievers | Fusion | Reranking |
|----------|-----------|------------|--------|-----------|
| `lexical` | `none` | `bm25` | `rrf` | `none` |
| `bm25` | `none` | `bm25` | `rrf` | `none` |
| `vector` | `none` | `vector` | `rrf` | `none` |
| `hybrid` | `none` | `bm25, vector` | `rrf` | `none` |
| `legacy-hybrid` | `none` | `bm25, phrase, vector` | `rrf` | `position-aware` |
| `page-index-hybrid` | `splade` | `bm25, phrase, vector` | `rrf` | `position-aware` |
| `page-index-llm` | `hyde` | `bm25, phrase, vector` | `rrf` | `llm` |
| `page-index-qwen` | `none` | `bm25, phrase, vector` | `rrf` | `llm` |
| `page-index-splade` | `splade` | `bm25, phrase, vector` | `rrf` | `position-aware` |
| `page-index-classified` | `classified` | `bm25, phrase, vector` | `rrf` | `position-aware` |
| `page-index-jina` | `splade` | `bm25, phrase, vector` | `rrf` | `jina` |
| `page-index-gemma` | `splade` | `bm25, phrase, vector` | `rrf` | `gemma` |

### CLI Overrides

The executable can override parts of the configured strategy per request:

```bash
sift search --intent "find the trait definition" --retrievers bm25,phrase --reranking none "engine"
```

Available overrides:

- `--intent`
- `--retrievers bm25,phrase,vector`
- `--fusion rrf`
- `--reranking none|position-aware|llm|jina|gemma`
- `--limit`
- `--shortlist`
- `--model-id`
- `--model-revision`
- `--rerank-model-id`
- `--rerank-revision`
- `--max-length`

`shortlist` controls how many fused candidates reach reranking. It does not set
the final number of returned results. `limit` controls the final output size.

The executable does not currently expose a raw `--expansion` flag. Expansion is
selected indirectly through the chosen strategy. If you need an explicit custom
expansion policy, use an explicit `SearchPlan` through the library surface.

### `[embedding]`

`[model]` is still accepted for backward compatibility, but `[embedding]` is
the preferred section name.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `model_id` | String | `"sentence-transformers/all-MiniLM-L6-v2"` | Embedding model ID. |
| `model_revision` | String | `"main"` | Embedding model revision. |
| `max_length` | Integer | `40` | Max sequence length for embedding. |

### `[rerank]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `model_id` | String | `"Qwen/Qwen2.5-0.5B-Instruct"` | Qwen reranker model ID. |
| `model_revision` | String | `"main"` | Qwen reranker revision. |
| `max_length` | Integer | `512` | Max sequence length for reranking. |

### `[gemma]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `model_id` | String | `"google/gemma-3-1b-it"` | Gemma reranker model ID. |
| `model_revision` | String | `"main"` | Gemma reranker revision. |
| `max_length` | Integer | `512` | Max sequence length for reranking. |

### `[prompts]`

Prompt overrides are optional and are consumed by generative expansion and the
optimizer.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `hyde` | String | unset | Override the HyDE system prompt. |
| `splade` | String | unset | Override the SPLADE system prompt. |
| `classified` | String | unset | Override the classified-expansion system prompt. |

## Ranking Notes

### Position-aware Reranking

This reranker adds structural bonuses such as filename and heading matches.

### Qwen / Gemma / Jina Reranking

These rerankers perform deeper semantic scoring on the shortlist. They improve
quality but typically cost more latency than `position-aware` or `none`.

## Agentic Status in Configuration

- There is no first-class `agentic` strategy or `[agentic]` controller section yet.
- Controller budgets, retained artifacts, and emission modes are exposed
  through the library request types rather than `sift.toml`.
- Agentic evaluations reuse the configured retrieval substrate instead of
  introducing a separate hidden stack.

## Library Configuration Notes

The stable embedding contract is documented in [LIBRARY.md](LIBRARY.md). The
short version is:

- Use `SearchOptions` to configure direct searches.
- Use `ContextAssemblyRequest` to build retained evidence for downstream tools.
- Use `SearchTurnRequest` for a single turn with explicit emission mode.
- Use `SearchControllerRequest` for deterministic multi-turn execution.

Some builder methods and option setters accept `sift::internal` types, such as
internal model specs or loaded config structs. Those are useful for advanced
embedding, but they are tighter couplings than the crate-root request/response
surface.

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SIFT_CACHE` | Root directory for all Sift state. |
| `SIFT_BLOBS_CACHE` | Override for the blob store. |
| `SIFT_MANIFESTS_CACHE` | Override for project manifests. |
| `SIFT_MODELS_CACHE` | Override for downloaded models. |
| `SIFT_DENSE_DEVICE` | Dense embedding device override: `cpu` or `cuda`. |
| `SIFT_LLM_DEVICE` | Default Candle-backed LLM device override: `cpu` or `cuda`. |
| `SIFT_QWEN_DEVICE` | Qwen-specific device override. |
| `SIFT_JINA_DEVICE` | Jina-specific device override. |
| `SIFT_GEMMA_DEVICE` | Gemma-specific device override. |
| `HF_TOKEN` | Hugging Face token for gated model downloads. |
