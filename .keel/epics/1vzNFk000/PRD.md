# Align Product Thesis With Current CLI - Product Requirements

> Replace the stale README thesis with the product that actually ships today:
> an indexless single-binary Rust CLI for hybrid local retrieval, evaluation,
> and measured benchmark reporting.

## Problem Statement

The public README still describes an older product direction centered on
`zvec`, an `index` command, disk-backed indices, and optional API embeddings.
That contract is now wrong. The shipped CLI uses transient in-memory retrieval,
defaults to hybrid BM25 plus dense reranking, runs dense inference through a
pure-Rust Candle path, and supports local text, HTML, PDF, and OOXML document
search without a database or daemon.

This mismatch is more than cosmetic. It misleads users about how to install,
run, evaluate, and reason about sift, and it obscures the architecture
constraints that the project has already proven through benchmark evidence.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make the README describe the current CLI surface and retrieval architecture accurately. | Stale command and architecture claims removed | No README references to indexing, `zvec`, disk-backed indices, or API embeddings remain |
| GOAL-02 | Document the executable workflows users can run today. | Command coverage | README examples map to `search`, `bench`, and `eval` commands that exist in `src/main.rs` |
| GOAL-03 | Ground the README thesis in recorded evidence instead of aspirational claims. | Evidence coverage | README includes measured benchmark facts and current format support drawn from Keel evidence |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Agentic Developer | Uses sift directly inside local coding or debugging workflows | Understand how to search local corpora immediately without pre-indexing infrastructure |
| Maintainer / Evaluator | Validates repository claims against the implemented CLI | Keep public docs consistent with the board, benchmark evidence, and current binary behavior |

## Scope

### In Scope

- [SCOPE-01] Rewrite the root README thesis and feature list to match the current single-binary retrieval design.
- [SCOPE-02] Replace stale usage examples with commands that exist in the current CLI.
- [SCOPE-03] Document supported formats, architectural constraints, and measured benchmark evidence already captured on the board.

### Out of Scope

- [SCOPE-04] Adding new runtime behavior, commands, or retrieval features that do not already exist.
- [SCOPE-05] Re-running large benchmarks solely to chase better README marketing numbers.
- [SCOPE-06] Publishing or packaging work outside the repository README.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The README SHALL describe sift as an indexless local retrieval CLI with the current search, benchmark, and evaluation workflows. | GOAL-01, GOAL-02 | must | Users need the public contract to match what they can actually run. |
| FR-02 | The README SHALL document the current architecture constraints and supported document families. | GOAL-01, GOAL-03 | must | The no-database, no-daemon, single-binary thesis is part of the product identity and needs to stay explicit. |
| FR-03 | The README SHALL cite measured evidence already captured in the board for hybrid quality, latency, and rich document support. | GOAL-03 | must | Claims about speed and retrieval quality should come from recorded proof, not aspiration. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | README changes SHALL use current, verifiable facts only and remove obsolete claims in the same slice. | GOAL-01, GOAL-03 | must | The repository follows a hard-cutover policy; stale contracts should not linger beside the new ones. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Stale-claim removal | targeted text checks + manual review | Story evidence showing removed `zvec`/index/API-embedding claims |
| CLI contract accuracy | command proofs against `cargo run -- --help`, `search --help`, `bench --help`, and `eval --help` | Story evidence linked to acceptance criteria |
| Evidence grounding | manual review against existing benchmark logs and rich-doc support artifacts | README diff plus cited metrics already present in Keel evidence |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current CLI surface in `src/main.rs` is the intended public contract for the next repository iteration. | The README could be aligned to a moving target. | Validate examples against the implemented commands during the story. |
| Existing benchmark evidence is sufficient to support a concise README summary. | The story might need to broaden into additional benchmarking work. | Cross-check against recorded Keel evidence before editing the README. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much benchmark detail belongs in the README versus Keel evidence links? | Engineering | Open |
| The README may still underspecify future directions once stale claims are removed. | Engineering | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [x] The new epic defines a documentation cutover that replaces stale product claims with the current contract.
- [ ] The README reflects the real CLI surface, architecture constraints, and measured behavior now shipped in the repo.
<!-- END SUCCESS_CRITERIA -->
