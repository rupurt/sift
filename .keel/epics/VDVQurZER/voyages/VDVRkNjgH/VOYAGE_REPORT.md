# VOYAGE REPORT: Library Facade and Packaging Cutover

## Voyage Metadata
- **ID:** VDVRkNjgH
- **Epic:** VDVQurZER
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Introduce Canonical Embedded Search Facade
- **ID:** VDVSF0yAX
- **Status:** done

#### Summary
Create the first supported high-level library entrypoint for running `sift`
search from another Rust project so embedders no longer need to compose broad
internal modules manually.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The library exposes one canonical embedded search facade with a high-level request/response path suitable for external Rust callers. <!-- verify: cargo check --tests, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-01/AC-02] Integration coverage exercises the supported facade directly rather than reaching through multiple internal modules. <!-- verify: cargo check --test library_facade_test, SRS-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDVSF0yAX/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VDVSF0yAX/EVIDENCE/ac-2.log)

### Decouple Public API From CLI Concerns
- **ID:** VDVSF1KAM
- **Status:** done

#### Summary
Narrow the supported library API so it no longer depends on CLI parsing,
terminal rendering, or other executable-only concerns in public types.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] Supported public library types used by embedders do not require `clap` derives or CLI-only enums in their contract. <!-- verify: cargo check --test library_facade_test, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] Terminal rendering helpers and similar CLI presentation concerns are no longer part of the canonical embedded API path. <!-- verify: sh -lc '! rg -n "render_search_response|RetrieverPolicy|FusionPolicy|RerankingPolicy" src/lib.rs', SRS-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDVSF1KAM/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VDVSF1KAM/EVIDENCE/ac-2.log)

### Adopt The Facade In The Executable
- **ID:** VDVSF1bAN
- **Status:** done

#### Summary
Rewire `src/main.rs` to consume the curated library facade while preserving the
current executable command surface and help contract.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] The executable builds and routes search execution through the curated library boundary instead of depending on newly unsupported internals. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo check --bin sift && rg -n "Sift::builder" src/main.rs && ! rg -n "SearchRequest|run_search|LocalFileCorpusRepository|StrategyPresetRegistry|DenseReranker|Embedder|RetrieverPolicy|FusionPolicy|RerankingPolicy" src/main.rs', SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-02] User-facing command names, argument shapes, and baseline behavior remain intact across the cutover. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && rg -n "Search\\(SearchCommand\\)|override_usage|after_help|retrievers: Option<Vec<SearchRetriever>>|fusion: Option<SearchFusion>|reranking: Option<SearchReranking>|value = \\\"bm25\\\"|value = \\\"phrase\\\"|value = \\\"vector\\\"|value = \\\"rrf\\\"|value = \\\"none\\\"|value = \\\"position-aware\\\"|value = \\\"llm\\\"" src/main.rs', SRS-03:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDVSF1bAN/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VDVSF1bAN/EVIDENCE/ac-2.log)

### Document The Supported Embedded API Boundary
- **ID:** VDVSF1tAK
- **Status:** done

#### Summary
Document which exports and modules are part of the supported embedded API and
trim or mark the rest as internal, then add a concrete guide showing how
another Rust project should actually depend on and call the library.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Repository documentation explains the supported embedded API path and distinguishes it from internal implementation modules. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && rg -n "## Embedded Library|supported library contract|sift::internal" README.md && rg -n "Supported embedded API|Everything under \\[`internal`\\]" src/lib.rs', SRS-04:start:end, proof: ac-1.log-->
- [x] [SRS-05/AC-02] Repository documentation includes a concrete library-usage guide with the supported dependency and a minimal embedding example. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && rg -n "git = \\\"https://github.com/rupurt/sift\\\"|path = \\\"../sift\\\"|Minimal Embedding Example|let response = sift.search|SearchInput::new" README.md', SRS-05:start:end, proof: ac-2.log-->
- [x] [SRS-06/AC-03] The public export surface is intentionally narrower than the current broad module re-export pattern, preserving the single-package rollout without adding an immediate workspace split. <!-- verify: sh -lc 'cd /home/alex/workspace/rupurt/sift && cargo check --all-targets && rg -n "^pub mod internal" src/lib.rs && ! rg -n "^pub mod (cache|config|dense|eval|extract|hybrid|search|segment|system|vector);" src/lib.rs', SRS-06:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VDVSF1tAK/EVIDENCE/ac-1.log)
- [ac-3.log](../../../../stories/VDVSF1tAK/EVIDENCE/ac-3.log)
- [ac-2.log](../../../../stories/VDVSF1tAK/EVIDENCE/ac-2.log)


