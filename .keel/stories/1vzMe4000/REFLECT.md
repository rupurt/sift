---
created_at: 2026-03-08T15:39:14
---

# Reflection - Add Extractor Boundary And HTML Search Support

## Knowledge

- [1vzMmS000](../../knowledge/1vzMmS000.md) Add Formats Below The Loader Boundary

## Observations

This story went quickly once the extraction work was isolated from ranking. The
shared `extract_path` boundary let HTML support land as a loader concern rather
than a search rewrite, which kept the existing BM25 and hybrid logic intact.

The only real friction was verifier path handling. The CLI proofs needed
`git rev-parse --show-toplevel` in the `verify:` annotations because Keel did
not reliably execute them from the repository root.

The fixture-based proof style also worked well here. A tiny checked-in corpus
was enough to prove HTML indexing at the CLI level, while temporary invalid
binary files in unit tests covered deterministic skip handling without forcing a
binary artifact into the repository.
