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

How will we know if this research was valuable?

- [x] Identify a plausible Rust-native extraction path for HTML, PDF, and Office formats that keeps sift as a single local binary with no daemon or external database.
- [x] Recommend an implementation sequence, explicit format boundaries, and known limitations that can be turned directly into the next epic, voyage, and stories.

## Open Questions

- Which PDF extraction crate gives the best first implementation tradeoff between text quality, maintenance cost, and performance without introducing OCR or native dependencies?
- Should Office support rely on a dedicated Rust library such as `undoc`, or should sift hand-roll narrow OOXML extraction using `zip` + XML parsing for tighter control?
- Which formats should be explicitly out of scope for the next slice, especially scanned PDFs and legacy binary Office formats (`.doc`, `.xls`, `.ppt`)?
