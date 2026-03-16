# Configurable Prompts - SRS

> Allow users to override HyDE, SPLADE, and Classification prompts via sift.toml.

## Scope

### In Scope

- [SCOPE-01] Add `prompts` section to `sift.toml` configuration.
- [SCOPE-02] Update `SearchServiceBuilder` to pass configured prompts to `LlmExpander`.
- [SCOPE-03] Create `sift optimize` CLI command to tune and save prompts.

### Out of Scope

- [SCOPE-04] Auto-tuning LLM reranking parameters (temperature, top_p).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-01 | Users can define custom prompts in `sift.toml`. | FR-01 | SCOPE-01 | manual: Edit config and verify |
| SRS-02 | The system must fall back to built-in prompts if none are defined. | FR-01 | SCOPE-02 | manual: Run without config |
| SRS-03 | `sift optimize` must accept paths to query and qrels files and save results to `sift.toml`. | FR-02 | SCOPE-03 | manual: Run optimize |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Source | Scope | Verification |
|----|-------------|--------|-------|--------------|
| SRS-04 | `sift optimize` must handle LLM errors gracefully. | NFR-01 | SCOPE-03 | manual: Code inspection |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->