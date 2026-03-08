---
id: 1vzMXf000
---

# Rich Document Extraction Support — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | Format coverage is the next major recall bottleneck now that text-only hybrid retrieval works end to end. |
| Confidence | 4 | The ecosystem has viable Rust-native libraries for HTML, text-bearing PDFs, and OOXML Office extraction. |
| Effort | 4 | The work spans extractor abstraction, fixtures, format-specific normalization, and benchmark updates. |
| Risk | 3 | Main risks are extraction quality and dependency fit, not fundamental feasibility. |

*Scores range from 1-5:*
- 1 = Very Low
- 2 = Low
- 3 = Medium
- 4 = High
- 5 = Very High

## Analysis

### Opportunity Cost

Pursuing rich document support delays deeper ranking work such as chunking,
query-time caching, and more aggressive hybrid fusion tuning. That tradeoff is
acceptable because unsupported formats are a larger practical recall gap than
another small ranking iteration on text-only corpora.

### Dependencies

The following need to hold:

- The chosen extraction crates must stay compatible with Linux and macOS in a
  pure-Rust, in-process path.
- Sift needs a narrow extractor interface so new formats do not fork the search
  or benchmark codepaths.
- The next stories must include local fixtures and evidence for extraction
  correctness, not just happy-path parsing.

### Alternatives Considered

Alternatives considered:

- Stay text-only for longer. Rejected because the board has already finished the
  text-only MVP and the user objective explicitly continues into richer formats.
- Shell out to system converters such as LibreOffice, Pandoc, or PDF toolkits.
  Rejected because this weakens the single-binary/local-runtime contract.
- Hand-roll every extractor from ZIP/XML/PDF primitives. Rejected as the first
  move because it increases implementation cost without evidence that the
  existing Rust libraries are inadequate.

## Recommendation

[x] Proceed → convert to epic
[ ] Park → revisit later
[ ] Decline → document learnings

Proceed with a new epic focused on rich document ingestion.

Recommended sequence:

1. Introduce a document extractor boundary beneath corpus loading.
2. Add HTML extraction first with `html2text`.
3. Add PDF extraction for text-bearing PDFs using `pdf-extract`, explicitly
   documenting that OCR/scanned PDFs remain unsupported.
4. Add OOXML Office extraction for `.docx`, `.xlsx`, and `.pptx`, preferring
   `undoc` first and revisiting a narrower hand-rolled path only if the library
   proves too heavy or too lossy.
