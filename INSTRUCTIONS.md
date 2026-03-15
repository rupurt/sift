# Instructions

Formal workflow and operational guidance for AI agents working with this repository.

## Workflow Orchestration (Keel)

All strategic planning, tactical decomposition, and verification-driven delivery must be managed through `keel`.

### 1. Mission Workflow (Strategic Steering)
Missions are the top-level loop for long-running autonomous objectives.
- **Bootstrap**: `keel mission new "<Title>"` if no active mission covers the goal.
- **Refine**: Fill `CHARTER.md` with goals (MG-*), constraints, and halting rules.
- **Activate**: `keel mission activate <id>`.
- **Execution Loop**:
  - `keel next --agent` to pull the highest priority task.
  - `keel mission log <id> --msg "<Description>"` for tactical choices/deviations.
  - `keel mission digest <id>` regularly to maintain context.
- **Completion**: `keel mission achieve <id>` followed by `keel mission verify <id>`.

### 2. Planning Workflow (Tactical Decomposition)
Architects must ensure every tactical unit is atomic and traceable.
- **Strategic Scaffolding**: `keel epic new "<Title>" --problem "<Problem>"`.
- **Tactical Decomposition**: `keel voyage new "<Title>" --epic <epic-id> --goal "<Goal>"`.
- **Requirements (SRS)**: Every requirement must be uniquely identified and traced to `FR-*` or `NFR-*`.
- **Design (SDD)**: Define the architectural approach before implementation.
- **Story Creation**: Break design into `keel story new "<Title>"`. Use `icebox` status until `keel voyage plan <id>` succeeds.
- **Sealing**: `keel voyage plan <id>` to lock the scope and thaw stories.

### 3. Execution Workflow (Verification-Driven Delivery)
Implementers must follow a strict TDD and evidence-capture loop.
- **Context**: `keel flow --scene` to identify bottlenecks.
- **Claim**: `keel next --agent` to pull the next item.
- **TDD Loop**:
  - Write failing test/proof first.
  - Implement minimum code to pass.
  - Refactor and verify.
- **Evidence**: `keel story record <id> --ac <NUM> --cmd "<Command>"` or `--msg` for each AC.
- **Reflect**: `keel story reflect <id>` to capture learnings.
- **Submit**: Exactly one atomic [Conventional Commit] per story, then `keel story submit <id>`.

## Foundational Hygiene

- **Doctor Check**: `keel doctor --scene` must pass before any story submission or planning seal.
- **Board Integrity**: `keel generate` after any structural change.
- **Hard Cutover**: Replace legacy patterns immediately; do not bridge or add aliases.
- **Traceability**: All stories must link to SRS rows; all SRS rows must link to Source tokens.
