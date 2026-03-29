# AGENTS.md

Shared guidance for AI agents working with this repository.

## Operational Guidance

This repository uses Keel as its project management engine. Your primary responsibility is to execute tactical moves that advance the board state while maintaining 100% integrity.

### Core Principles
1. **Gardening First**: You MUST tend to the garden (fixing `doctor` errors, discharging automated backlog, and resolving structural drift) BEFORE notifying the human operator or requesting input.
2. **Heartbeat Hygiene**: Monitor the system's pulse via `just keel heartbeat` and `just keel health --scene`. The pacemaker is derived from repository activity; uncommitted energy in the worktree is tactical debt that should be closed autonomously by landing the sealing commit.
3. **Notification Discipline**: Ping the human operator ONLY when you need input on design direction or how the application behaves. Resolve technical drift and tactical moves autonomously.

### Canonical Turn Loop
Keel's operator rhythm is the `Orient -> Inspect -> Pull -> Ship -> Close` loop surfaced by `just keel turn`.

- **Orient**: Inspect charge and board stability with `just keel heartbeat`, `just keel health --scene`, `just keel flow --scene`, and `just keel doctor`.
- **Inspect**: Read current demand with `just keel mission next --status`, `just keel pulse`, `just keel roles`, and `just keel next --role <role> --explain` when routing is unclear.
- **Pull**: Select one role-scoped slice with `just keel next --role <role>`.
- **Ship**: Execute the slice, record proof while the work is fresh, and land the relevant lifecycle transition (`story submit`, `voyage plan`, `bearing lay`, and similar) before the commit boundary when it rewrites `.keel` artifacts.
- **Close**:
  - Record the move in the mission `LOG.md`.
  - Use `just keel heartbeat` if you need to confirm the current activity source or inspect whether open-loop energy is still present before sealing the turn.
  - Inspect `git status` and include the resulting code and `.keel` churn in the same atomic commit.
  - Execute `git commit`. The installed hooks can run repo-local checks and append `doctor --status` metadata to the commit message; resolve any rejection before proceeding.
  - Re-orient with `just keel doctor --status` and `just keel flow --scene` after the commit lands so the next turn starts from the real board state.

### Session Start & Human Interaction
When a human user opens the chat or "pokes" you (e.g., "Wake up", "I'm poking you"), you MUST immediately perform the `Orient` and `Inspect` halves of the turn loop by following the **Human Interaction & Pokes** workflow in [INSTRUCTIONS.md](INSTRUCTIONS.md):
1. **Heartbeat**: Run `just keel heartbeat` to inspect current charge and whether the worktree is carrying uncommitted energy.
2. **Pulse**: Run `just keel health --scene` to check subsystem stability.
3. **Scan**: Run `just keel mission next --status` and `just keel pulse`.
4. **Confirm**: Run `just keel flow --scene` to verify whether the LIGHT IS ON or the board is idle waiting for fresh repository activity.
5. **Diagnose**: Run `just keel doctor` to ensure board integrity before proceeding.

### Procedural Instructions
Follow the formal procedural loops and checklists defined in:
👉 **[INSTRUCTIONS.md](INSTRUCTIONS.md)**

## Decision Resolution Hierarchy

When faced with ambiguity, resolve decisions in this descending order:
1. **ADRs**: Binding architectural constraints.
2. **CONSTITUTION**: The collaboration philosophy and delivery rules.
3. **WORLD**: The conceptual model and "physics" of Sift.
4. **ARCHITECTURE**: Source layout and technical boundaries.
5. **CONFIGURATION**: Runtime, build, and environment topology.
6. **PLANNING**: PRD, SRS, SDD, and mission artifacts authored for the current scope.

## Foundational Documents

These define the constraints and workflow of this repository:

| Document | Purpose |
|----------|---------|
| `README.md` | Entrypoint and canonical project navigation |
| `INSTRUCTIONS.md` | Step-by-step procedural loops and checklists |
| `CONSTITUTION.md` | Collaboration philosophy and decision hierarchy |
| `WORLD.md` | Conceptual world model for Sift |
| `ARCHITECTURE.md` | Implementation architecture and component boundaries |
| `CONFIGURATION.md` | Runtime, build, and environment configuration |
| `EVALUATIONS.md` | Evaluation datasets and methodology |
| `RESEARCH.md` | Strategic research and future-facing direction |
| `RELEASE.md` | Release process and artifacts |
| `.keel/adrs/` | Binding architecture decisions |

Use this order when interpreting constraints: ADRs → Constitution → World → Architecture → Configuration → Planning artifacts.

## Project Overview

This repository is `sift` — a standalone hybrid and agentic search tool for fast document retrieval in agentic coding workflows.

Sift is intended to stay lightweight: a single Rust binary with no external database requirement, combining BM25 keyword ranking with vector-backed semantic search and LLM-based reranking today, with turn-based search scaffolding growing into a formal local search controller.

| Path | Purpose |
|------|---------|
| `README.md` | Current project description |
| `flake.nix` | Nix flake for the dev shell and shared tooling |
| `justfile` | Repo-local workflow wrappers |
| `AGENTS.md` | Shared agent workflow contract |
| `INSTRUCTIONS.md` | Procedural instructions and checklists |
| `.keel/` | Project board, planning artifacts, and ADRs |

## Board Directory (`.keel/`)

A `.keel/` directory is the runtime data directory that `keel` operates on.

| Path | Contents |
|------|----------|
| `.keel/adrs/` | Architecture decision records |
| `.keel/bearings/` | Research and discovery artifacts |
| `.keel/epics/` | Epic-level planning artifacts |
| `.keel/epics/<epic-id>/voyages/` | Voyage planning artifacts |
| `.keel/missions/` | Long-running mission charters and logs |
| `.keel/stories/` | Implementable work items |
| `.keel/README.md` | Board state overview |

## Commands

### Command Execution Model

Use one path for each concern:

- `nix develop` for the repository shell and shared tooling.
- `just ...` for repo build, test, formatting, benchmarking, and example workflows.
- `keel ...` for all planning, mission, execution, research, and verification workflows.
- `just keel ...` as thin convenience wrappers for a small subset of board commands.

### `just` Workflow Commands

| Command | Purpose |
|---------|---------|
| `just` | List available recipes |
| `just fmt` | Format the workspace |
| `just fmt-check` | Check formatting |
| `just clippy` | Run workspace clippy |
| `just check` | Run formatting, clippy, tests, and doc tests |
| `just test` | Run the main test suite with `cargo nextest` |
| `just test-doc` | Run doc tests |
| `just build [profile]` | Build `sift` for `debug` or `release` |
| `just build-static` | Build the static Nix artifact |
| `just sift ...` | Run the CLI via `cargo run --release` |
| `just embed-build` | Build the embedded example binary |
| `just embed-sift <path> <query>` | Run the embed example against a path |
| `just embed-sift-here <query>` | Run the embed example against the current directory |
| `just bench` | Run benchmarks |
| `just flamegraph ...` | Run flamegraph profiling for eval flows |

### `keel` Board Workflow Commands

Run `keel --help` for the full command tree. Common commands:

| Category | Commands |
|----------|----------|
| Discovery | `keel bearing new <name>` `keel bearing research <id>` `keel bearing assess <id>` `keel bearing list` |
| Planning | `keel epic new "<title>" --problem "<problem>"` `keel voyage new "<title>" --epic <epic-id> --goal "<goal>"` |
| Execution | `keel story new "<title>" [--type <type>] [--epic <epic-id> [--voyage <voyage-id>]]` |
| Board Ops | `keel turn` `keel mission next [<id>]` `keel next --role manager` `keel next --role operator` `keel next --role <role> --explain` `keel roles` `keel flow --scene` `keel doctor` `keel health --scene` `keel generate` `keel config show` `keel mission show <id>` `keel mission attach <mission-id> --epic <epic-id>` |
| Verification | `keel verify run <id>` `keel verify detect` `keel verify recommend` |
| Pulse | `keel heartbeat` `keel pulse` `keel poke "<summary>"` |

### Story and Milestone State Changes

Use CLI commands only. Do not move `.keel` files manually.

| Action | Command |
|--------|---------|
| Start | `keel story start <id>` |
| Reflect | `keel story reflect <id>` |
| Submit | `keel story submit <id>` |
| Reject | `keel story reject <id> "reason"` |
| Accept | `keel story accept <id> --role manager` |
| Ice | `keel story ice <id>` |
| Thaw | `keel story thaw <id>` |
| Voyage plan | `keel voyage plan <id>` |
| Voyage done | `keel voyage done <id>` |
| Bearing assess | `keel bearing assess <id>` |
| Bearing lay | `keel bearing lay <id>` |
| Mission activate | `keel mission activate <id>` |
| Mission attach | `keel mission attach <mission-id> --epic <epic-id> \| --bearing <bearing-id> \| --adr <adr-id>` |
| Mission achieve | `keel mission achieve <id>` |
