---
created_at: 2026-03-08T16:08:34
---

# Knowledge - 1vzMd0000

> Automated synthesis of story reflections.

## Story Knowledge

## Story: Add Text-Bearing PDF Extraction (1vzMeH000)

### 1vzMmS000: Add Formats Below The Loader Boundary

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Extending sift from plain UTF-8 files into additional document formats |
| **Insight** | Rich-format support stays tractable when the format-specific work stops at a shared extraction boundary that returns normalized text and metadata. BM25 indexing, dense reranking, CLI rendering, and benchmarks can then stay unchanged above that seam. |
| **Suggested Action** | Add future PDF and Office handlers by implementing the extractor path first, and only revisit ranking code if extracted text quality proves insufficient. |
| **Applies To** | `src/extract.rs`, `src/search.rs`, future rich-document handlers |
| **Applied** | yes |



---

## Story: Add Extractor Boundary And HTML Search Support (1vzMe4000)

### 1vzMmS000: Add Formats Below The Loader Boundary

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Extending sift from plain UTF-8 files into additional document formats |
| **Insight** | Rich-format support stays tractable when the format-specific work stops at a shared extraction boundary that returns normalized text and metadata. BM25 indexing, dense reranking, CLI rendering, and benchmarks can then stay unchanged above that seam. |
| **Suggested Action** | Add future PDF and Office handlers by implementing the extractor path first, and only revisit ranking code if extracted text quality proves insufficient. |
| **Applies To** | `src/extract.rs`, `src/search.rs`, future rich-document handlers |
| **Applied** | yes |



---

## Story: Add OOXML Office Extraction And Mixed-Format Verification (1vzMeL000)

### 1vzN83000: Use `undoc::extract_text` as the canonical OOXML runtime path

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Adding local `.docx`, `.xlsx`, or `.pptx` retrieval without converters or helper services |
| **Insight** | `undoc::extract_text(path)` provides a single in-process pure-Rust extraction path across Word, Excel, and PowerPoint OOXML files, which keeps the runtime aligned with the single-binary and no-daemon constraints. |
| **Suggested Action** | Reuse the shared extractor boundary for future rich-format handlers and prefer one canonical library entrypoint per family instead of format-specific wrappers. |
| **Applies To** | `src/extract.rs`, rich document ingestion, mixed-format search fixtures |
| **Applied** | yes |



---

## Synthesis

### iXOCh6FSd: Add Formats Below The Loader Boundary

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Extending sift from plain UTF-8 files into additional document formats |
| **Insight** | Rich-format support stays tractable when the format-specific work stops at a shared extraction boundary that returns normalized text and metadata. BM25 indexing, dense reranking, CLI rendering, and benchmarks can then stay unchanged above that seam. |
| **Suggested Action** | Add future PDF and Office handlers by implementing the extractor path first, and only revisit ranking code if extracted text quality proves insufficient. |
| **Applies To** | `src/extract.rs`, `src/search.rs`, future rich-document handlers |
| **Linked Knowledge IDs** | 1vzMmS000 |
| **Score** | 0.91 |
| **Confidence** | 0.94 |
| **Applied** | yes |

### OL3CdIDl6: Add Formats Below The Loader Boundary

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Extending sift from plain UTF-8 files into additional document formats |
| **Insight** | Rich-format support stays tractable when the format-specific work stops at a shared extraction boundary that returns normalized text and metadata. BM25 indexing, dense reranking, CLI rendering, and benchmarks can then stay unchanged above that seam. |
| **Suggested Action** | Add future PDF and Office handlers by implementing the extractor path first, and only revisit ranking code if extracted text quality proves insufficient. |
| **Applies To** | `src/extract.rs`, `src/search.rs`, future rich-document handlers |
| **Linked Knowledge IDs** | 1vzMmS000 |
| **Score** | 0.91 |
| **Confidence** | 0.94 |
| **Applied** | yes |

### OiIO5Vp3I: Use `undoc::extract_text` as the canonical OOXML runtime path

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Adding local `.docx`, `.xlsx`, or `.pptx` retrieval without converters or helper services |
| **Insight** | `undoc::extract_text(path)` provides a single in-process pure-Rust extraction path across Word, Excel, and PowerPoint OOXML files, which keeps the runtime aligned with the single-binary and no-daemon constraints. |
| **Suggested Action** | Reuse the shared extractor boundary for future rich-format handlers and prefer one canonical library entrypoint per family instead of format-specific wrappers. |
| **Applies To** | `src/extract.rs`, rich document ingestion, mixed-format search fixtures |
| **Linked Knowledge IDs** | 1vzN83000 |
| **Score** | 0.88 |
| **Confidence** | 0.95 |
| **Applied** | yes |

