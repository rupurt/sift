# Reflection - Wire Cache Into Prepared Corpus

The `corpus.rs` module has been successfully integrated with the global file cache. 
- Fast path (Heuristic match) skips all extraction and hashing.
- Medium path (Hash match, Heuristic miss) skips extraction.
- Slow path (Total miss) performs extraction, warms the global `blobs` store, and updates the local `manifest`.
- Changes are safely flushed to disk at the end of the `load_search_corpus` routine.
