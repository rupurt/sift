# Zig-Style Global File Cache - Product Requirements

> By adopting a Zig-style incremental caching model (hashing at the file level and looking up cached metadata via fast filesystem heuristics like `mtime`, `inode`, and `size`), we can build a lightning-fast "Search Asset Pipeline" without the operational overhead of a daemon or full relational database.

## Problem Statement

`sift` currently extracts text and computes dense embeddings (which is expensive) for every file on every run. As the corpus grows, this transient approach becomes a massive performance bottleneck. We need a way to reuse extraction and vectorization work across runs and across different projects, but we must strictly avoid the complexity of traditional sidecar databases or daemons (like the removed `zvec`).

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Drop indexing latency for unchanged files to near-zero. | Repeated search latency | Repeated searches skip Candle inference entirely. |
| GOAL-02 | Avoid database daemons and project-local artifacts. | Architectural compliance | State is purely file-based in `~/.cache/sift/`. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Searches local projects | Fast, sub-second responses even for large repos. |

## Scope

### In Scope

- [SCOPE-01] Introduce a `blake3` file hashing pipeline.
- [SCOPE-02] Build a global binary Blob Store in `~/.cache/sift/blobs/` using `bincode`.
- [SCOPE-03] Build a global project-keyed Manifest directory to map heuristics to hashes.
- [SCOPE-04] Integrate this caching layer into the `PreparedCorpus` pipeline.

### Out of Scope

- [SCOPE-05] Background indexing daemons.
- [SCOPE-06] Database implementations (SQLite, RocksDB).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `sift` must read and write binary representations of `Document`s to `~/.cache/sift/blobs/<blake3_hash>`. | GOAL-01 | must | Cache hit must avoid extraction and inference. |
| FR-02 | `sift` must map `(absolute_project_root)` to a binary manifest file in `~/.cache/sift/manifests/`. | GOAL-02 | must | Avoids locking contention across different projects. |
| FR-03 | The manifest must map `(relative_path, inode, mtime, size)` to the corresponding `blake3_hash`. | GOAL-01 | must | Bypasses file hashing when heuristics match. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | File reading must be as zero-copy as feasible. | GOAL-01 | should | Keeps load times minimal. |
| NFR-02 | Cross-process manifest access must be protected by advisory file locks. | GOAL-02 | must | Prevents corruption when `sift` is invoked concurrently. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove cache hits and misses via integration tests manipulating `mtime`.
- Demonstrate speedup via the `bench latency` command.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| `bincode` is fast enough for document deserialization | May need `rkyv` or `mmap` | Measure latency overhead during deserialization |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How big will the blob store get over time? | Engineering | Open (we may need an eviction command later) |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Define the storage format and metadata lookup mechanisms that avoid SQLite/databases while maintaining fast cache hits.
- [ ] Determine how to uniquely identify a file state (e.g., `(inode, mtime, size) -> blake3_hash`) safely and globally.
- [ ] Ensure the approach supports fast binary reading of pre-computed embeddings and term frequencies.
- [ ] Validate the alignment with `sift`'s core principles (Zero-Friction Operations, Determinism).
<!-- END SUCCESS_CRITERIA -->
