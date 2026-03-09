---
id: 1vzMXf000
---

# Rich Document Extraction Support — Survey

## Market Research

### Existing Solutions

The developer-tooling baseline is still heavily text-first. Tools like
`ripgrep` and plain filesystem search work well on UTF-8 source files but do
not natively surface content trapped in HTML exports, PDFs, or Office bundles.
Teams usually compensate with ad hoc conversion steps or separate desktop/cloud
products, which breaks the "single local binary in the workflow" thesis that
drives sift.

For the extraction layer itself, the current Rust ecosystem has credible
format-specific libraries:

- PDF: [`pdf-extract`](https://docs.rs/pdf-extract/latest/pdf_extract/) offers
  focused text extraction for text-bearing PDFs, while
  [`pdf_oxide`](https://docs.rs/pdf_oxide/latest/pdf_oxide/) is a broader and
  newer PDF processing stack.
- HTML: [`html2text`](https://docs.rs/html2text/latest/html2text/) converts
  HTML into readable plain text, and
  [`html5ever`](https://docs.rs/html5ever/latest/html5ever/) remains the
  robust parser foundation if sift later needs richer DOM-aware behavior.
- Office: [`undoc`](https://docs.rs/undoc/latest/undoc/) exposes plain-text,
  Markdown, and JSON extraction for DOCX, XLSX, and PPTX.

### Competitive Landscape

The main alternatives split into two camps:

- External-tool pipelines such as `pandoc`, PDFium wrappers, or LibreOffice
  conversions. These increase format coverage but violate sift's current
  no-daemon/single-binary preference and complicate cross-platform packaging.
- Hosted or heavyweight retrieval tools that ingest rich documents after an
  offline indexing step. Those improve recall but move away from sift's
  transient, JIT search thesis.

### Market Size

For sift, the opportunity is not a generic TAM story; it is recall coverage
inside real developer corpora. Local documentation sets, vendor knowledge
bases, specs, design docs, and exported reports frequently arrive as HTML,
PDF, or OOXML files. Supporting those formats materially expands the number of
documents sift can search without asking users to normalize inputs manually.

## Technical Research

### Feasibility

The extension is feasible, but each format has a different risk profile:

- HTML is low risk. The content is already textual, and Rust-native parsing plus
  HTML-to-text rendering is mature.
- PDF is medium risk. Text-bearing PDFs are tractable, but scanned PDFs would
  require OCR, which is explicitly out of scope for the current contract.
- Office is medium risk. Modern OOXML containers are ZIP + XML and therefore
  compatible with a pure-Rust path, but preserving useful reading order and
  worksheet/slide context still needs format-specific handling.

The common architectural requirement is an extraction boundary below the search
pipeline: `Path -> ExtractedDocument { text, metadata, warnings }`. That keeps
BM25 indexing, dense reranking, and benchmarks format-agnostic.

### Prior Art

The most relevant building blocks are:

- `pdf-extract` for first-pass PDF text recovery from text-bearing files.
- `html2text` for quick HTML normalization into readable text.
- `undoc` for OOXML extraction across DOCX/XLSX/PPTX without shelling out to
  external applications.

If any of these prove insufficient, the fallback is still pure Rust:

- HTML can drop to `html5ever` + custom text walking.
- OOXML can be parsed directly through ZIP + XML readers for narrower control.
- PDF can move down-stack to a lower-level parser, but that is higher effort.

### Proof of Concepts

No code experiments have been run for this bearing yet. The proof in this slice
is architectural: the candidate libraries all support an in-process Rust path
and can slot under the existing transient corpus loader instead of requiring a
persisted indexing redesign.

## User Research

### Target Users

The target users are engineers and agents searching local working sets that mix
source code with exported documentation, specifications, slide decks, and
reports.

### Pain Points

Current pain points:

- Relevant documents are invisible to sift unless the user manually converts
  them to plain text first.
- Hybrid retrieval quality is capped by corpus coverage; unsupported formats are
  recall losses before ranking even starts.
- External conversion steps are brittle in CI/dev-shell workflows and often
  introduce native dependencies that do not belong in this repository yet.

### Validation

The product objective itself already names PDF, HTML, and Office support as the
next boundary after MVP. That is consistent with the observed gap in sift's
current behavior: the search path is now solid for text files, so the next
meaningful recall improvement is broader document ingestion rather than another
ranking rewrite.

## Key Findings

1. HTML support is the lowest-risk immediate addition and does not require any
   search-architecture changes beyond a document extraction boundary.
2. PDF support is viable for text-bearing PDFs with a pure-Rust library, but
   scanned/OCR-heavy PDFs must remain explicitly out of scope for the next slice.
3. Modern Office support should target OOXML (`.docx`, `.xlsx`, `.pptx`) first;
   legacy binary Office formats should remain deferred because they add
   complexity without fitting the current lightweight contract.
4. A unified extractor abstraction lets sift expand format coverage while
   keeping the existing transient BM25 + dense hybrid pipeline intact.

## Unknowns

- Whether `undoc` is the best long-term Office dependency or only the fastest
  path to the first implementation slice.
- How much retrieval quality changes once extracted HTML/PDF/OOXML text is fed
  through the current hybrid ranking pipeline.
- Whether PDF layout artifacts require lightweight post-processing heuristics
  before indexing.

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | web | manual:web-search | https://docs.rs/pdf-extract/latest/pdf_extract/ | 2026-03-08 | 2026-03-08 | medium | high | pdf-extract offers focused text extraction for text-bearing PDFs |
| SRC-02 | web | manual:web-search | https://docs.rs/pdf_oxide/latest/pdf_oxide/ | 2026-03-08 | 2026-03-08 | medium | high | pdf_oxide provides broader PDF processing stack in pure Rust |
| SRC-03 | web | manual:web-search | https://docs.rs/html2text/latest/html2text/ | 2026-03-08 | 2026-03-08 | medium | high | html2text converts HTML into readable plain text |
| SRC-04 | web | manual:web-search | https://docs.rs/html5ever/latest/html5ever/ | 2026-03-08 | 2026-03-08 | high | high | html5ever is the robust HTML parser foundation for DOM-aware processing |
| SRC-05 | web | manual:web-search | https://docs.rs/undoc/latest/undoc/ | 2026-03-08 | 2026-03-08 | medium | high | undoc exposes text, Markdown, and JSON extraction for DOCX, XLSX, and PPTX |
