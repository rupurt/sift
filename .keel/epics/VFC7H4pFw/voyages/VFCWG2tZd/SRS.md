# Add Agent Search CLI Surface - SRS

## Summary

Epic: VFC7H4pFw
Goal: Let the shipped CLI trigger the same autonomous planner runtime through sift search --agent while preserving existing non-agent search behavior.

## Scope

### In Scope

- [SCOPE-01] CLI support for planner-driven search through `sift search --agent`.
- [SCOPE-02] Agent-mode output and JSON behavior that preserve planner strategy and trace visibility.
- [SCOPE-03] Preservation of current non-agent search behavior and surrounding search/eval workflows.

### Out of Scope

- [SCOPE-04] Generic interactive agent shells.
- [SCOPE-05] Branching search UX.
- [SCOPE-06] Hosted orchestration.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The executable must support planner-driven search through `sift search --agent` using the shared autonomous runtime. | SCOPE-01 | FR-03 | manual: CLI proof |
| SRS-02 | Agent-mode CLI output and JSON must expose planner strategy and autonomous trace metadata appropriate for inspection and downstream tooling. | SCOPE-02 | FR-04 | manual: output review |
| SRS-03 | Non-agent `sift search` behavior must remain unchanged when the agent flag is not selected. | SCOPE-03 | FR-03 | manual: regression review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The CLI agent surface must call the same runtime contract as the supported library autonomous path rather than reimplementing planner execution. | SCOPE-01, SCOPE-02 | NFR-01 | manual: architecture review |
| SRS-NFR-02 | Adding the agent flag must not regress the current search command or existing evaluation/search workflows. | SCOPE-03 | NFR-02 | manual: regression proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
