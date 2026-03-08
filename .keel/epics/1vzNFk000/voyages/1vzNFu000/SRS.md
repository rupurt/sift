# Refresh README For Current Retrieval Contract - Software Requirements Specification

> Rewrite the repository README so it reflects the current single-binary,
> indexless hybrid search CLI, its actual commands, supported formats, and
> measured evidence.

**Epic:** [1vzNFk000](../../README.md) | **SDD:** [SDD.md](SDD.md)

## Scope

### In Scope

- [SCOPE-01] Replace stale README architecture and workflow claims with the current sift contract.
- [SCOPE-02] Document the current CLI command surface with executable examples.
- [SCOPE-03] Summarize supported formats and recorded benchmark evidence already proven on the board.

### Out of Scope

- [SCOPE-04] New retrieval implementation work, command additions, or benchmark methodology changes.
- [SCOPE-05] Marketing copy that promises unsupported packaging or hosted-service behavior.

## Assumptions & Dependencies

| Assumption/Dependency | Type | Impact if Invalid |
|-----------------------|------|-------------------|
| `src/main.rs` is the authoritative source for the current public CLI surface | dependency | README examples could still drift from the binary |
| Existing Keel evidence from the hybrid and rich-document voyages is sufficient to cite measured facts | dependency | The voyage would need to broaden into new benchmark work |
| Root README is the canonical public product thesis for repository visitors | assumption | The wrong document could remain the dominant contract |

## Constraints

- The README must follow the repository's hard-cutover policy and remove stale claims instead of leaving compatibility wording behind.
- Claims about latency, quality, formats, or architecture must be traceable to current code or existing board evidence.
- The voyage must not introduce new runtime features just to make the README easier to write.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The README SHALL describe sift as an indexless single-binary retrieval CLI with `search`, `bench`, and `eval` workflows instead of the stale `index`/`zvec` contract. | SCOPE-01 | FR-01 | text check + CLI help proof |
| SRS-02 | The README SHALL document executable usage examples that match the current commands and options exposed by the CLI. | SCOPE-02 | FR-01 | CLI help proof + manual review |
| SRS-03 | The README SHALL document the currently supported document families and architectural constraints: UTF-8 text, HTML, text-bearing PDF, OOXML, no database, no daemon, and no persisted sidecar index. | SCOPE-03 | FR-02 | manual review + text check |
| SRS-04 | The README SHALL summarize measured benchmark evidence already recorded on the board for hybrid quality/latency and rich document support. | SCOPE-03 | FR-03 | manual review + evidence cross-check |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-05 | The README cutover SHALL remove contradictory obsolete claims in the same change slice rather than introducing dual-path documentation. | SCOPE-01 | NFR-01 | text check + manual review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
