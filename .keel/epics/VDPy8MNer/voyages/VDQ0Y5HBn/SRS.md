# Mmap-Accelerated Blob Retrieval - SRS

> Improve I/O efficiency for large document corpora by using memory-mapped files for blob retrieval.

## Scope

### In Scope

- [SCOPE-01] Replace standard file I/O with memory-mapped files in `load_blob`.
- [SCOPE-02] Evaluate if `mmap` improves performance for large blob deserialization.
- [SCOPE-03] Ensure fallback to standard I/O if `mmap` fails or is not supported.

### Out of Scope

- [SCOPE-04] Implementing zero-copy deserialization (requires major refactoring to `Document` types).
- [SCOPE-05] Mmapping the entire manifest file (staying with `bincode` for now).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-01 | `load_blob` must use `memmap2` to map the document blob before deserialization. | FR-04 | SCOPE-01 | board: VDQ0Yf9R1 |
| SRS-02 | The system must gracefully handle mapping failures by falling back to `File::read`. | FR-04 | SCOPE-03 | manual: Error injection test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-03 | `mmap` based loading should not increase latency for small documents (<1MB). | NFR-02 | SCOPE-02 | command: cargo bench |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Run `cargo bench` to compare standard I/O vs `mmap` for blob loading.
- Verify `sift search -vv` works correctly after the change.
