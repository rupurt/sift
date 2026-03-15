# VOYAGE REPORT: Layered Search Strategy Foundation

## Voyage Metadata
- **ID:** 1vzdCx000
- **Epic:** 1vzXLN000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Model Layered Search Plans
- **ID:** 1vzdDJ000
- **Status:** done

#### Summary
Model the search domain around explicit search plans and hexagonal ports so
query expansion, retrieval, fusion, reranking, CLI execution, and benchmark
execution all share the same orchestration model.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Search execution is represented as a domain search plan with explicit query-expansion, retrieval, fusion, and reranking phases. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] CLI, bench, and eval entrypoints use the same application-layer search-plan orchestration instead of embedding strategy policy directly in each command path. <!-- verify: command, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-14/AC-03] The domain model and search-plan orchestration remain independent of CLI parsing, benchmark rendering, filesystem traversal, and concrete model-runtime adapters. <!-- verify: manual, SRS-14:start:end, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdDJ000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdDJ000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzdDJ000/EVIDENCE/ac-3.log)

### Add Query Expansion And Phrase Retrieval
- **ID:** 1vzdDn000
- **Status:** done

#### Summary
Introduce a configurable query-expansion phase and first-class lexical retriever
adapters so BM25 remains the baseline while phrase/proximity retrieval becomes a
parallel option that can complement vector search.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Named strategies can enable or disable query expansion and produce zero or more query variants through the shared search-plan model. <!-- verify: cargo test search::application::tests::search_service_orchestrates_multiple_variants_and_retrievers, SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-04/AC-02] Multiple retrievers can run independently over the same prepared corpus state and return separate candidate lists for later fusion. <!-- verify: cargo test search::application::tests::search_service_fuses_results_from_multiple_retrievers, SRS-04:start:end, proof: ac-2.log -->
- [x] [SRS-05/AC-03] The lexical retriever set includes `bm25` as the stable baseline and at least one phrase/proximity-aware retriever. <!-- verify: manual, SRS-05:start:end, proof: ac-3.log -->
- [x] [SRS-06/AC-04] Structure-aware segment vector retrieval is adapted into the shared retriever layer as an independent retriever rather than a reranking stage. <!-- verify: manual, SRS-06:start:end, proof: ac-4.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdDn000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdDn000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzdDn000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vzdDn000/EVIDENCE/ac-4.log)

### Add Fusion And Reranking Layers
- **ID:** 1vzdDu000
- **Status:** done

#### Summary
Separate result fusion from reranking so hybrid strategies can combine multiple
retrievers with RRF by default and leave reranking as a bounded optional stage
behind its own port.

#### Acceptance Criteria
- [x] [SRS-07/AC-01] The shared fusion layer uses Reciprocal Rank Fusion by default and preserves contributor provenance for explanation and benchmarking. <!-- verify: cargo test search::adapters::tests::rrf_fuser_preserves_provenance, SRS-07:start:end, proof: ac-1.log -->
- [x] [SRS-08/AC-02] The reranking layer is optional, exposed behind a reranker port, and ships with `none` as the default implementation. <!-- verify: manual, SRS-08:start:end, proof: ac-2.log -->
- [x] [SRS-13/AC-03] The default fusion and reranking stack remains local-first and does not require external databases or resident services. <!-- verify: manual, SRS-13:start:end, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdDu000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdDu000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzdDu000/EVIDENCE/ac-3.log)

### Add Named Hybrid Strategy Presets
- **ID:** 1vzdE0000
- **Status:** done

#### Summary
Add a registry of named strategy presets so sift can expose baseline, hybrid,
and PageIndex-inspired search compositions while keeping `hybrid` as a stable
alias to the current champion preset.

#### Acceptance Criteria
- [x] [SRS-09/AC-01] Sift ships named strategy presets including `bm25`, at least one composite hybrid preset, and one PageIndex-inspired preset definition composed from the available layers. <!-- verify: manual, SRS-09:start:end, proof: ac-1.log -->
- [x] [SRS-10/AC-02] The top-level `hybrid` strategy resolves through a configurable champion preset instead of embedding retrieval logic directly in the CLI command path. <!-- verify: cargo test search::application::tests::strategy_preset_registry_resolves_named_presets_and_hybrid_alias, SRS-10:start:end, proof: ac-2.log -->
- [x] [SRS-13/AC-03] The shipped presets preserve the default single-binary, local-first operating contract. <!-- verify: manual, SRS-13:start:end, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdE0000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdE0000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzdE0000/EVIDENCE/ac-3.log)

### Benchmark Strategies Against Baseline And Champion
- **ID:** 1vzdE8000
- **Status:** done

#### Summary
Extend bench and eval so every named strategy can be executed through the
shared search-plan pipeline and compared against the BM25 baseline and the
current champion preset with exact recorded evidence.

#### Acceptance Criteria
- [x] [SRS-11/AC-01] Bench and eval commands compare candidate strategies against both the BM25 baseline and the configured champion preset through the shared strategy pipeline. <!-- verify: command cargo --version, SRS-11:start:end, proof: ac-1.log -->
- [x] [SRS-12/AC-02] Benchmark artifacts record strategy composition, query expansion settings, fusion/reranking settings, segment configuration, model/runtime settings, corpus shape, git SHA, hardware summary, and command lines. <!-- verify: manual, SRS-12:start:end, proof: ac-2.log -->
- [x] [SRS-13/AC-03] Comparative benchmark evidence makes the default local operating posture explicit when strategies are evaluated. <!-- verify: manual, SRS-13:start:end, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzdE8000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzdE8000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzdE8000/EVIDENCE/ac-3.log)


