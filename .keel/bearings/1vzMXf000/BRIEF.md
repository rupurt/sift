# Rich Document Extraction Support — Brief

## Hypothesis

Sift can extend beyond raw text files without violating the single-binary,
no-daemon, no-database contract by adding Rust-native extraction adapters for
HTML, PDF, and modern Office documents. If the extraction path is good enough
to recover stable plain-text search corpora on demand, the existing transient
BM25 + hybrid ranking pipeline can stay unchanged above the loader boundary.

## Problem Space

The MVP only supports ASCII and UTF-8 text files. Real developer corpora
regularly include local HTML exports, API docs, PDFs, and OOXML Office files.
Without native extraction support, sift misses a large share of relevant
documents and forces users back to ad hoc conversion steps or separate tools.

The main risk is choosing format handlers that quietly violate the repository
constraints by pulling in native runtimes, OCR stacks, or service-like
behavior. PDF and Office extraction are also quality-sensitive: poor text
recovery or layout loss can degrade both lexical and dense retrieval.

## Success Criteria

The research is considered successful if it achieves the following outcomes:

- **Library Evaluation:** Evaluate `lopdf`, `selectng`, and `undoc` against the Rust-only, single-binary constraints and identify the best first implementation path.
- **Unified Extraction Boundary:** Propose a single `SourceKind` and extraction trait that covers PDF, HTML, and OOXML without leaking format-specific details into the core search logic.
- **Performance Baseline:** Establish an expected extraction latency for typical (1-10MB) document blobs to ensure search startup remains "local-fast".
- **Implementation Strategy:** Provide a clear sequence of epics and stories that can be executed to add support for these formats in the next release cycle.

## Open Questions

- Which PDF extraction crate gives the best first implementation tradeoff between text quality, maintenance cost, and performance without introducing OCR or native dependencies?
- Should Office support rely on a dedicated Rust library such as `undoc`, or should sift hand-roll narrow OOXML extraction using `zip` + XML parsing for tighter control?
- Which formats should be explicitly out of scope for the next slice, especially scanned PDFs and legacy binary Office formats (`.doc`, `.xls`, `.ppt`)?
