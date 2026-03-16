# Prompt Optimization Engine - Charter

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Support user-configurable prompts for generative expansion strategies (HyDE, SPLADE, Classified) via `sift.toml`. | board: VE1xUOaxK |
| MG-02 | Implement a `sift optimize` command that runs a local tuning pass against a provided query/qrels file and saves the highest-yielding prompts to the local configuration. | board: VE1xUOaxK |

## Constraints

- MUST preserve the default prompts if no configuration is provided.
- MUST NOT require an external API for the optimization loop (use the configured local LLM).
- `sift optimize` MUST use the same `ReactorMetrics` (Signal Gain) logic to determine the best prompt.

## Halting Rules

- DO NOT halt until `sift optimize` can successfully write improved prompts to `sift.toml`.
- HALT when the `Configurable Prompts and Optimizer` epic is fully delivered and verified.
- YIELD to human if the local LLM generation errors consistently prevent the optimization loop from finishing.