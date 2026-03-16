# Configurable Prompts and Optimizer - Product Requirements

> Hardcoded prompts in generative expansion prevent users from adapting the retrieval physics to their specific codebase's vocabulary and domain.

## Problem Statement

Currently, the prompts used for generative query expansion (HyDE, SPLADE, Classified) are hardcoded in the application source. This prevents users from adapting the "physics" of the retrieval engine to their specific codebase's vocabulary and domain. An optimizer is needed to automatically tune these prompts.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Configurable Prompts | Users can override default prompts | Custom prompts apply via `sift.toml` |
| GOAL-02 | Optimization Command | Users can auto-tune prompts | `sift optimize` yields higher Signal Gain |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Power User | Embeds sift in agent workflows | Wants to tune the retrieval physics to their specific domain. |

## Scope

### In Scope

- [SCOPE-01] Add `prompts` section to `sift.toml`.
- [SCOPE-02] Update `SearchServiceBuilder` to use configured prompts.
- [SCOPE-03] Create `sift optimize` CLI command.

### Out of Scope

- [SCOPE-04] Auto-tuning LLM reranking parameters (temperature, top_p).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The system must read custom prompts from the configuration file. | GOAL-01 | must | Allows manual overriding of the default physics. |
| FR-02 | The system must provide a command to empirically tune prompts against a dataset. | GOAL-02 | must | Automates the discovery of high-yield prompt "control rods". |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Prompt configuration parsing must not significantly delay CLI startup. | GOAL-01 | must | Fast startup is a core principle of sift. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Test `sift optimize` locally to ensure it writes a valid `sift.toml`.
- Verify standard searches respect the overridden prompts in the local `sift.toml`.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Domain-specific prompts yield higher Signal Gain than generic prompts. | Optimization is useless. | Empirical results from `sift optimize`. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Will the LLM deviate too far during mutation? | Engineer | Open. We might need constraints. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Users can define a custom prompt in `sift.toml`.
- [ ] `sift optimize` command successfully tunes and saves prompts.
<!-- END SUCCESS_CRITERIA -->