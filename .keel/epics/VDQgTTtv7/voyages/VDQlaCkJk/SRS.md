# SRS: Implement Scanned PDF Fallback

## Scope

<!-- BEGIN SCOPE -->
### In scope
- [SCOPE-01] Detect when standard PDF extraction yields little to no text.
- [SCOPE-02] Trigger OCR extraction for detected scanned PDFs when the `ocr` feature is enabled.
- [SCOPE-03] Provide unit tests for the PDF OCR fallback heuristic.

### Out of scope
- [SCOPE-04] Pure-Rust PDF-to-image rendering (if Tesseract cannot natively process the PDF, we may rely on external tools or document limitations).
- [SCOPE-05] Complex layout analysis or table extraction from PDFs.
<!-- END SCOPE -->

## Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Priority | Verification |
|----|-------------|--------|-------|----------|--------------|
| SRS-01 | `extract_pdf` must check the length of extracted text to determine if it is a scanned PDF (heuristic). | FR-01 | SCOPE-01 | must | manual: Inspect src/extract.rs |
| SRS-02 | If a scanned PDF is detected and the `ocr` feature is enabled, `extract_pdf` must attempt to use Tesseract for extraction. | FR-01 | SCOPE-02 | must | manual: Inspect src/extract.rs |
| SRS-03 | If the `ocr` feature is disabled, `extract_pdf` should return the empty text (existing behavior) and log a warning. | FR-01 | SCOPE-02 | must | manual: Inspect src/extract.rs |
<!-- END FUNCTIONAL_REQUIREMENTS -->

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Priority | Verification |
|----|-------------|--------|-------|----------|--------------|
| SRS-04 | The fallback heuristic must have minimal latency overhead for standard text-heavy PDFs. | NFR-01 | SCOPE-01 | should | manual: Review code |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Success Criteria
- [ ] Image-only PDFs yield searchable text when the `ocr` feature is enabled.
- [ ] No performance regression for standard, text-extractable PDFs.
