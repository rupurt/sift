# Zig-Style Global File Cache — Brief

## Hypothesis

By adopting a Zig-style incremental caching model (hashing at the file level and looking up cached metadata via fast filesystem heuristics like `mtime`, `inode`, and `size`), we can build a lightning-fast "Search Asset Pipeline" without the operational overhead of a daemon or full relational database.

## Problem Space

`sift` currently extracts text and computes dense embeddings (which is expensive) for every file on every run. As the corpus grows, this transient approach becomes a massive performance bottleneck. We need a way to reuse extraction and vectorization work across runs and across different projects, but we must strictly avoid the complexity of traditional sidecar databases or daemons (like the removed `zvec`).

## Success Criteria

- [ ] Define the storage format and metadata lookup mechanisms that avoid SQLite/databases while maintaining fast cache hits.
- [ ] Determine how to uniquely identify a file state (e.g., `(inode, mtime, size) -> blake3_hash`) safely and globally.
- [ ] Ensure the approach supports `mmap` for fast reading of pre-computed embeddings and term frequencies.
- [ ] Validate the alignment with `sift`'s core principles (Zero-Friction Operations, Determinism).

## Open Questions

- How do we handle metadata cache concurrency if multiple `sift` processes run at the same time?
- Should the fast-path index (`path/inode/mtime/size -> hash`) be global or stored per-project? (User confirmed metadata lookup must be fast and potentially global, but we need to design the exact mechanism).
- How do we handle file path collisions across different repos if the index is global, or does the index key on absolute paths?
- What serialization format offers the best balance of simplicity and `mmap` compatibility for storing blobs?
