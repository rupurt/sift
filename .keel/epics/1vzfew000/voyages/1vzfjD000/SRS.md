# Incremental File Caching - Software Requirements Specification

> Implement the manifest and blob storage logic to bypass document extraction.

## Scope

### In Scope

- [SCOPE-01] Implement `blake3` hashing for file contents.
- [SCOPE-02] Implement binary serialization of `Document` using `bincode`.
- [SCOPE-03] Implement project-level manifest files storing file heuristics.
- [SCOPE-04] Wire the cache into `corpus.rs`.

### Out of Scope

- [SCOPE-05] Cache eviction or GC.
- [SCOPE-06] In-memory persistence.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | A file's `blake3` hash must be used as the blob storage key. | SCOPE-01 | FR-01 | Test verifying hash generation and blob lookup. |
| SRS-02 | Extracted documents must be serialized to `~/.cache/sift/blobs/<hash>` using `bincode`. | SCOPE-02 | FR-01 | Test verifying successful round-trip binary serialization. |
| SRS-03 | A manifest file must be stored per searched path in `~/.cache/sift/manifests/<path_hash>`. | SCOPE-03 | FR-02 | Test verifying manifest creation upon search. |
| SRS-04 | The manifest must map `(relative_path, inode, mtime, size)` to the file's `blake3` hash. | SCOPE-03 | FR-03 | Test verifying the mapping bypasses full rehashing when heuristics match. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-05 | Manifest writes must acquire a file lock to prevent corruption from concurrent access. | SCOPE-03 | NFR-02 | Code inspection for `fs2` or `fd-lock` usage. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
