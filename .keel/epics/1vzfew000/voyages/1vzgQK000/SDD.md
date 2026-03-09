# Vector Embedding Caching - Software Design Description

> Update Segment model and corpus loading to compute and store dense embeddings in the global blob store.

## Architecture

We are extending the existing "Search Asset Pipeline" to include dense vector embeddings in the serialized `Document` blobs.

### The "Fast Retrieval" Flow

When a document is indexed:
1.  Text is extracted.
2.  Segments are built.
3.  **NEW:** The dense model encodes every segment.
4.  The `Document` (including segments with embeddings) is serialized to `~/.cache/sift/blobs/`.

When a query is executed:
1.  The query is embedded by the model.
2.  For every `Segment` in the corpus:
    - If `embedding` is `Some`, perform dot-product similarity immediately.
    - If `embedding` is `None` (legacy or miss), fallback to inference.

## Implementation Details

### Segment Model Change
```rust
pub struct Segment {
    pub id: String,
    // ... other fields
    pub embedding: Option<Vec<f32>>,
}
```

### Corpus Loading Change
In `src/search/corpus.rs`, `index_document` will need access to the `DenseReranker` to compute embeddings for new segments.
Alternatively, we can pass the model into `load_search_corpus`.
