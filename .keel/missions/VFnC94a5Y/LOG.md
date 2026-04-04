# Build Sector-Aware Frontier Search Cache Reuse - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-04-03T21:17:42-07:00

- Created the mission after confirming that restart-time search still performs a full corpus scan before BM25 cache reuse.
- Framed the strategic problem around three linked concepts: SectorMap for cache validity, FrontierLedger for partial-coverage scoring, and BreadcrumbJournal for resumable indexing progress.
- Set the mission constraints to preserve Sift's local-first single-binary contract while enabling useful frontier search before a fully sealed index exists.

## 2026-04-03T21:23:33

Reviewed the reactor-created mission and tightened the plan to require direct-search-first rollout, extension of the existing manifest/blob/BM25 cache substrate, and same-process convergence language instead of ambiguous background-validation wording.
