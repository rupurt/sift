---
source_type: Story
source: stories/1vzMe4000/REFLECT.md
scope: 1vzMXf000/1vzMd0000
source_story_id: 1vzMe4000
created_at: 2026-03-08T15:39:14
---

### 1vzMmS000: Add Formats Below The Loader Boundary

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Extending sift from plain UTF-8 files into additional document formats |
| **Insight** | Rich-format support stays tractable when the format-specific work stops at a shared extraction boundary that returns normalized text and metadata. BM25 indexing, dense reranking, CLI rendering, and benchmarks can then stay unchanged above that seam. |
| **Suggested Action** | Add future PDF and Office handlers by implementing the extractor path first, and only revisit ranking code if extracted text quality proves insufficient. |
| **Applies To** | `src/extract.rs`, `src/search.rs`, future rich-document handlers |
| **Linked Knowledge IDs** |  |
| **Observed At** | 2026-03-08T22:40:00+00:00 |
| **Score** | 0.91 |
| **Confidence** | 0.94 |
| **Applied** | yes |
