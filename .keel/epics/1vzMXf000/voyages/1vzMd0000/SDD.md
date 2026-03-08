# Implement Rich Document Extractors - Software Design Description

> Add a shared extractor layer plus end-to-end HTML, PDF, and OOXML document support with mixed-format verification

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds a format-aware extraction layer beneath the existing corpus
loader. Search and benchmark code keep operating on normalized plain text; the
new work happens in the ingestion boundary that maps a filesystem path to
`ExtractedDocument` content plus warnings/metadata.

## Context & Boundaries

```
┌────────────────────────────────────────────────────┐
│                    This Voyage                     │
│                                                    │
│  WalkDir -> Extractor Dispatch -> Normalized Text  │
│                 |               |                  │
│                 |               +-> warnings/meta  │
│                 v                                  │
│        HTML / PDF / OOXML extractors               │
│                        |                           │
│                        v                           │
│        Existing transient BM25 + hybrid ranking    │
└────────────────────────────────────────────────────┘
            ↑                         ↑
      local rich files         local CLI / benchmarks
```

In scope:

- Extractor dispatch by extension/content type.
- HTML, text-bearing PDF, and OOXML extraction.
- Mixed-format tests, demos, and benchmark/report updates.

Out of scope:

- OCR/scanned PDFs.
- Legacy binary Office documents.
- New persisted indexing or background workers.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `html2text` | Rust crate | Convert HTML into readable plain text for indexing | latest compatible crate release |
| `pdf-extract` | Rust crate | Recover text from text-bearing PDFs in-process | latest compatible crate release |
| `undoc` or equivalent OOXML crate | Rust crate | Extract text from `.docx`, `.xlsx`, and `.pptx` locally | latest compatible crate release |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Extraction boundary | Introduce a shared extractor layer below corpus loading | Keeps ranking and benchmark logic format-agnostic |
| HTML strategy | Use HTML-to-text normalization instead of DOM-aware ranking | Fastest low-risk path to searchable content |
| PDF scope | Support text-bearing PDFs only in this voyage | OCR would violate the current lightweight contract |
| Office scope | Target OOXML first (`.docx`, `.xlsx`, `.pptx`) | Matches the pure-Rust path and avoids legacy binary complexity |

## Architecture

The search stack becomes:

1. Filesystem traversal identifies candidate files.
2. Extractor dispatch selects the handler based on extension and format support.
3. Each handler returns normalized UTF-8 text plus warnings/metadata.
4. Existing corpus loading builds BM25 statistics and feeds hybrid reranking
   exactly as it does today.
5. Search/bench output reports indexed vs skipped files and any extraction
   failures in deterministic counts.

## Components

- `extract` module:
  format dispatch, normalized output type, warning/error taxonomy.
- HTML extractor:
  converts HTML into plain text while stripping markup noise.
- PDF extractor:
  recovers text from text-bearing PDFs and returns explicit failures for
  unsupported/empty outputs.
- OOXML extractor:
  pulls readable text from Word, Excel, and PowerPoint containers.
- corpus loader integration:
  replaces direct UTF-8 file reads with extractor-driven loading.
- mixed-format fixtures and benchmarks:
  prove correctness and operational behavior.

## Interfaces

Planned internal interface shape:

- `extract_path(path: &Path) -> Result<Option<ExtractedDocument>>`
- `ExtractedDocument { text: String, source_kind: SourceKind, warnings: Vec<String> }`
- `SourceKind` includes at least `text`, `html`, `pdf`, `docx`, `xlsx`, `pptx`

Callers should treat `Ok(None)` as a deterministic skip for unsupported files
and `Err(...)` as a surfaced extraction failure that increments skip/failure
accounting without terminating the whole command.

## Data Flow

`WalkDir` discovers files -> extractor dispatch selects a handler -> handler
produces normalized text -> corpus loader tokenizes/indexes text -> existing
search and benchmark flows rank documents -> CLI output renders results and
aggregate counts.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Unsupported extension or non-target format | Dispatch miss | Skip deterministically | Continue corpus walk |
| HTML/PDF/OOXML extraction failure | Handler returns error | Record warning/skip count | Continue corpus walk |
| Empty extracted text | Handler output inspection | Treat as skip or low-signal document per format policy | Continue corpus walk |
| Dependency/runtime mismatch | Build/test or CLI failure | Fail the story during verification | Re-plan dependency choice before rollout |
