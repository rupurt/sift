# Reflect - Model Layered Search Plans (1vzdDJ000)

## What was learned?
- Transitioning to a layered search architecture required moving many domain objects out of `src/search.rs` (now `legacy.rs`) into `src/search/domain.rs` and `src/search/corpus.rs`.
- Hexagonal architecture with explicit ports (traits) for retrievers, fusers, and rerankers makes the system much more extensible.
- Using a `SearchPlan` to describe the execution pipeline allows for flexible named presets and easy comparison in benchmarks.
- The `hybrid` alias now correctly resolves to a "champion" preset, fulfilling the goal of stable user experience with evolving underlying technology.

## Any surprises?
- `keel story record` with `cargo run` required an absolute path to the corpus in some environments, likely due to how the subprocess is spawned or managed.
- The initial monolithic `rank_corpus` function had a lot of intertwined logic that had to be carefully separated into modular adapters.

## Future improvements?
- Implement more query expansion strategies (e.g., lexical variants).
- Add more retrievers (e.g., proximity-based) and rerankers (e.g., dense/LLM).
- Further decouple `SearchService` from concrete model types by moving model management into its own adapter layer.
