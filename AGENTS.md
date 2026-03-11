# AGENTS.md

Shared guidance for AI agents working with this repository.

## Bootstrap Workflow

1. Enter the development shell with `nix develop`.
2. If the board is not initialized yet, run `keel init` in the repo root.
3. Regenerate board summaries after structural changes with `keel generate`.
4. Validate board health before finalizing work with `keel doctor`.

## Mission Workflow (Autonomous Operation)

Missions are the top-level steering loop for long-running, autonomous objectives.
An agent should ALWAYS be working within a mission when the user provides a
broad product or feature goal.

1. **Bootstrap Mission**: If no active mission covers the current user request,
   create one: `keel mission new "<Title>"`.
2. **Refine Charter**: Fill out `CHARTER.md` with specific goals (MG-*),
   constraints, and halting rules.
   - Every goal should have a clear verification path (`board:`, `metric:`, or `manual:`).
   - Use `keel mission refine <id>` to ensure the charter is structurally sound.
3. **Activate**: Promote the mission to `active` status: `keel mission activate <id>`.
4. **Log Decisions**: Record significant tactical choices, research findings,
   or deviations in the mission log: `keel mission log <id> --msg "Description"`.
5. **Digest Regularly**: Periodically summarize log entries into the mission's
   session digest to maintain context: `keel mission digest <id>`.
6. **Execution Loop**: While the mission is active:
   - Run `keel next --agent` to pull the next task.
   - If no tasks are ready but mission goals are incomplete, use Planning or
     Research workflows to extend the board.
   - Stop ONLY when the mission's halting rules are met or an external blocker
     is reached.
7. **Achievement**: Once all goals are satisfied, mark the mission as achieved:
   `keel mission achieve <id>`.
8. **Final Verification**: Run `keel mission verify <id>` to transition from
   Achieved to Verified, ensuring all evidence is traceable.

## Execution Workflow (Implementer)

1. **Pull Context**: Read current board health and identify bottlenecks with `keel flow`.
2. **Claim Work**: Pull the highest-priority implementation item with `keel next --agent`.
   - If no story is ready and the mission goals are still incomplete, switch
     to research or planning work immediately instead of stopping.
3. **Check Story Coherence Before Coding**: Confirm acceptance criteria are traceable and verifiable:
   - Acceptance criteria are linked to source requirements (for example `[SRS-XX/AC-YY]`).
   - Use canonical `SRS-XX` IDs in story acceptance criteria. If a non-functional
     requirement needs implementation traceability, model it through a canonical
     `SRS-XX` row sourced from `NFR-*` rather than referencing `SRS-NFR-*`
     directly from the story.
   - Evidence strategy is clear for each criterion (test, CLI proof, or manual proof).
   - If requirements are ambiguous, loop back to planning artifacts before implementation.
4. **Execute (TDD)**: Follow test-driven development:
   - Write a failing test first.
   - Implement only enough to pass.
   - Refactor within the same change slice.
5. **Record Evidence**: Capture proof of requirement satisfaction for each acceptance criterion:
   - `keel story record <ID> --ac <NUM> --msg "Description of the proof"`
   - For command-based proofs, use `--cmd`.
   - For manual proofs, use `--msg` or attached files.
6. **Reflect**: Run `keel story reflect <ID>` and document what was learned during implementation.
7. **Commit (Required)**: Create exactly one atomic [Conventional Commit](https://www.conventionalcommits.org/) for this story before submission.
8. **Submit**: Use `keel story submit <ID>` to run the transition gate.
   - If the story auto-completes, continue to the next item.
   - If the story requires manual verification and you have directly performed
     that review, complete the acceptance step explicitly and keep moving.
9. **Continue The Product**: After each completed story, re-run `keel flow` and
   `keel next --agent`. If the queue is empty but the mission goals are not,
   extend the board and continue.

## Verification-Driven Delivery Loop (Required)

Treat verification as the steering loop for every story instead of a final
cleanup step.

1. **Mandatory quality gates before any change is accepted**: For every code/config/documentation change, run `just check` before continuing. If `just check` cannot run, record the exact failing command, truncated output, and blocker reason in the story/handoff before proceeding.

1. **Plan Proof Before Editing**: Before changing code, map every acceptance
   criterion to a concrete proof path:
   - automated test
   - command proof
   - or direct manual review artifact
2. **Start With A Failing Check**: Add or run the failing test, assertion, or
   proof target first so the gap is observable before implementation begins.
3. **Keep Proofs Local To The Slice**: Prefer small targeted checks during the
   story, then run broader repo verification before submission.
4. **Run Repo Verification Before `keel story submit`**:
   - repo-specific formatting commands
   - repo-specific lint commands, if present
   - repo-specific test commands, if present
   - `keel verify run <story-id>` once evidence and annotations are in place
5. **Apply The Global Hygiene Checklist Explicitly**:
   - run `keel generate` after structural board changes
   - run `keel doctor` before finalizing the story
   - state exactly what was and was not verified when automation is incomplete
6. **Record Exact Evidence**: Capture the exact commands, outputs, or manual
   inspection notes that prove each acceptance criterion. Do not rely on
   memory-based summaries alone.
7. **Close The Review Loop Immediately**:
   - run `keel story submit <story-id>`
   - inspect the resulting state with `keel story show <story-id>` when useful
   - if manual review is required and you directly reviewed the result, run
     `keel story accept <story-id>` in the same workstream
8. **Do Not Stop At A Clean Commit**: A passing test suite, a clean commit, or
   an accepted story is not a stopping condition while mission goals remain.

## Planning Workflow (Architect)

1. **Identify Gaps**: Use `keel flow` or `keel status` to find epics needing tactical decomposition.
   - If execution is starved but mission goals are incomplete, create the
     next planning unit instead of waiting for more instructions.
2. **Scaffold Planning Unit**:
   - For new strategic work, create an Epic: `keel epic new "<Title>" --problem "<Problem>"`
   - For tactical decomposition, create a Voyage: `keel voyage new "<Title>" --epic <epic-id> --goal "<Specific outcome>"`
3. **Author Epic PRD Immediately After Creation**: Before decomposing into voyages or stories, fill out `PRD.md` with authored content for every required section.
4. **Define Requirements (SRS)**: Fill out the voyage `SRS.md`. Requirements should be atomic, uniquely identified, and directly traceable to story acceptance criteria.
   - In both `PRD.md` and `SRS.md`, the `Scope` section must include explicit
     `In Scope` and `Out of Scope` subsections with canonical `[SCOPE-*]`
     bullets or `keel doctor` will fail.
   - Each SRS row's `Source` column must contain exactly one canonical `FR-*`
     or `NFR-*` token.
5. **Detail Design (SDD)**: Fill out `SDD.md` with the architectural approach and component changes.
6. **Decompose Stories**: Break the design into implementable units with `keel story new "<Title>"`.
   - After creating a story, verify that its frontmatter includes
     `scope: <epic-id>/<voyage-id>`. Keel may create the story file without
     attaching scope automatically, even when invoked from inside a voyage
     directory.
   - Keep newly created stories in `icebox` while the voyage remains `draft`.
     Thaw them only after `keel voyage plan <id>` succeeds.
7. **Align Verification Techniques From Config**: Run `keel config show`, `keel verify detect`, and `keel verify recommend` before finalizing verification planning.
8. **Run Coherence Review**: Ensure every requirement has story coverage and every acceptance criterion has a concrete verification path.
   - Prefer a one-story-to-one-requirement-slice mapping when possible. If
     multiple stories share the same implementation-facing SRS rows, Keel may
     report dependency-cycle errors during `keel doctor` or `keel voyage plan`.
9. **Loop Back Upstream if Needed**: If decomposition exposes ambiguity, update PRD, SRS, or SDD first.
10. **Generate Planning Summary In Chat (Required)**: Publish a terse planning summary in the harness response for every newly planned Epic or Voyage.
11. **Commit (Required)**: Create exactly one atomic [Conventional Commit](https://www.conventionalcommits.org/) for the planning unit.
12. **Seal Planning**: Promote the voyage when planning is complete with `keel voyage plan <id>`.
    - `keel voyage plan` will fail if the voyage has no scoped stories, if any
      acceptance criterion is missing a canonical `SRS-XX` reference, or if the
      scoped stories form a dependency cycle.
13. **Return To Execution**: After planning, immediately resume `keel next --agent`
    and continue implementation unless a real blocker remains.

## Research Workflow (Explorer)

1. **Identify Fog**: Create a bearing when the path forward is ambiguous: `keel bearing new "<Name>"`.
   - If execution is blocked by architectural uncertainty and the queue has no
     safe next step, create the bearing yourself and keep moving.
2. **Discovery (Play)**: Use `keel play <id>` to explore the problem space through different perspectives.
3. **Draft Brief**: Fill out `BRIEF.md`. The sections `Hypothesis`, `Problem Space`, `Success Criteria`, and `Open Questions` are mandatory.
4. **Survey Findings**: Document research, technical constraints, and alternatives in `SURVEY.md`.
5. **Seal Survey**: Transition to surveying with `keel bearing survey <id>`.
6. **Assess Impact**: Document the recommendation in `ASSESSMENT.md`.
7. **Seal Assessment**: Transition to assessment with `keel bearing assess <id>`.
8. **Commit (Required)**: Create exactly one atomic [Conventional Commit](https://www.conventionalcommits.org/) for the research package.
9. **Graduate**: If research is conclusive, graduate the bearing with `keel bearing lay <id>`.
10. **Feed Planning Or Execution Immediately**: Once the bearing is conclusive,
    create or update the downstream epic, voyage, or stories in the same
    overall workstream rather than stopping at research.

## Global Hygiene Checklist

Apply these checks to every change before finalizing work:

1. **Doctor Check**: `keel doctor` must pass with zero warnings or errors.
2. **Project Verification**: Run the repo-specific formatting, linting, and test commands that exist. If automation is not available yet, state exactly what was and was not verified.
3. **Board Regeneration**: Run `keel generate` after structural board changes so summaries stay current.
4. **Atomic Commits**: Commit once per logical unit of work using [Conventional Commits](https://www.conventionalcommits.org/).
5. **Verification Gate Discipline**: Run story-level verification (`keel verify run <story-id>` when applicable), submit through the transition gate, and complete manual acceptance immediately after direct review rather than leaving stories waiting.

## Compatibility Policy (Hard Cutover)

At this stage of development, this repository uses a hard cutover policy by default.

1. **No Backward Compatibility by Default**: Do not add compatibility aliases, dual-write logic, soft-deprecated schema fields, or fallback parsing for legacy formats unless a story explicitly requires it.
2. **Replace, Don’t Bridge**: When introducing a new canonical token, field, command behavior, or document contract, remove the old path in the same change slice.
3. **Fail Fast in Validation**: `keel doctor` and transition gates should treat legacy or unfilled scaffold patterns as hard failures when they violate the current contract.
4. **Single Canonical Path**: Keep one source of truth for rendering, parsing, and validation.
5. **Migration Is Explicit Work**: If existing artifacts need updates, handle that in a dedicated migration pass instead of embedding runtime compatibility logic.

## Foundational Documents

These define current constraints and workflow:

- `CONSTITUTION.md` — repository operating principles and binding delivery rules.
- `ARCHITECTURE.md` — current system architecture and component boundaries.
- `CONFIGURATION.md` — runtime, build, and environment configuration guidance.
- `EVALUATION.md` — evaluation datasets, methodology, and benchmark expectations.
- `RELEASE.md` — release process, packaging, and distribution workflow.
- `README.md` — repository intent and product positioning.
- `flake.nix` — Nix development environment and shared tooling entrypoint.
- `Cargo.toml` — crate dependencies and current package metadata.
- `src/main.rs` — current CLI surface and execution entrypoint.
- `.keel/adrs/` — binding architecture decisions once the board is initialized.
- `.keel/` planning artifacts — executable requirements, design, and work breakdown.

Use this order when interpreting constraints: ADRs, when present, then
`CONSTITUTION.md`, `ARCHITECTURE.md`, `CONFIGURATION.md`, `EVALUATION.md`,
`RELEASE.md`, `README.md`, then planning artifacts.

## Project Overview

This repository is `sift` — a standalone **Hybrid Information Retrieval (IR) system** for fast
document retrieval in agentic coding workflows.

Sift is intended to stay lightweight: a single Rust binary with no external
database requirement, combining BM25 keyword ranking with vector-backed semantic
search and LLM-based re-ranking to capture user **intent**. The product thesis
is that retrieval quality and CLI ergonomics should be available directly
inside the developer workflow rather than delegated to a separate service stack.

The current repository is still in bootstrap mode. Expect the CLI surface,
indexing model, and planning artifacts to evolve together while the board
becomes the authoritative record for requirements and implementation slices.

| Path | Purpose |
|------|---------|
| `README.md` | Current project description and product framing |
| `flake.nix` | Nix flake for the dev shell and shared tooling |
| `Cargo.toml` | Crate metadata and dependency graph |
| `src/main.rs` | Current CLI entrypoint |
| `AGENTS.md` | Shared agent workflow contract |
| `.keel/` | Project board, planning artifacts, and ADRs |

## Board Directory (`.keel/`)

A `.keel/` directory is the runtime data directory that `keel` operates on. It
lives in this repository once initialized.

| Path | Contents |
|------|----------|
| `.keel/adrs/` | Architecture decision records |
| `.keel/epics/` | Epic-level planning artifacts |
| `.keel/epics/<epic-id>/voyages/` | Voyage planning artifacts (`SRS.md`, `SDD.md`) |
| `.keel/stories/` | Implementable work items |
| `.keel/README.md` | Board state overview |

## Commands

### Command Execution Model

Use one path for each concern:

- `nix develop` for the repository shell and shared tooling.
- `keel ...` for planning, execution, research, and verification workflows.
- `just ...` for primary development workflows (search, test, bench, eval).
- `cargo ...` for low-level Rust operations.

### Core `just` Recipes

| Recipe | Description |
|--------|-------------|
| `just search` | Run a release search (e.g., `just search . "query"`) |
| `just test` | Run the full verification suite (fmt, clippy, nextest) |
| `just build` | Build the project and sync the binary to `target/debug` |
| `just dataset prepare` | Download and materialize the evaluation dataset |
| `just eval all` | Run comparative benchmarks across all strategies |

### Core `keel` Commands

| Category | Commands |
|----------|----------|
| Setup | `keel init` `keel config show` `keel generate` |
| Mission | `keel mission new <name>` `keel mission activate <id>` `keel mission log <id> --msg <msg>` `keel mission digest <id>` `keel mission achieve <id>` `keel mission verify <id>` |
| Discovery | `keel bearing new <name>` `keel bearing survey <id>` `keel bearing assess <id>` `keel bearing list` |
| Planning | `keel epic new <name> --problem <problem>` `keel voyage new <name> --epic <epic-id> --goal <goal>` `keel voyage plan <id>` |
| Execution | `keel next --agent` `keel story new <title>` `keel story thaw <id>` `keel story start <id>` `keel story record <id>` `keel story reflect <id>` `keel story submit <id>` `keel story accept <id>` |
| Diagnostics | `keel doctor` `keel status` `keel flow` `keel gaps` `keel verify detect` `keel verify recommend` `keel verify run <id>` |
