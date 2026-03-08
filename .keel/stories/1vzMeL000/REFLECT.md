---
created_at: 2026-03-08T16:06:33
---

# Reflection - Add OOXML Office Extraction And Mixed-Format Verification

## Knowledge

- [1vzN83000](../../knowledge/1vzN83000.md) Use `undoc::extract_text` as the canonical OOXML runtime path

## Observations

The extractor boundary absorbed OOXML support cleanly. Search-path loading already had the right shape once extraction was normalized into `ExtractedDocument`, so the main code change was extension routing plus recursive materialized-corpus loading for benchmark fixtures.

The main surprise was verification behavior rather than extraction behavior. The benchmark corpus loader correctly excludes `test-queries.tsv` and `qrels/`, but `sift search` over the fixture root still indexes those files because the interactive search path intentionally treats them as ordinary UTF-8 text. That does not break this story's acceptance criteria because the three OOXML documents still rank first and repeated runs are deterministic, but it is a real product distinction worth planning explicitly if we want search over eval corpora to hide benchmark metadata by default.
