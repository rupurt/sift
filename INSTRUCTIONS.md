# INSTRUCTIONS.md

Procedural instructions and workflow guidance for agents and operators working with the Sift repository.

## The Tactical Loop

Sift uses Keel as its project management engine. Your job is to perform tactical moves that push work through the state machine while eliminating drift.

Every session follows this deterministic cycle:

1. **Mission Orientation**: Start by running `just keel mission next --status`. This gives you the top high-signal moves required by the engine. Check `just keel flow --scene` to visualize whether the workflow is autonomous or blocked waiting for human input.
2. **Role Selection**: Identify whether you are acting as a `manager` (planning and decisions) or an `operator` (implementation). Do not drift across these roles in a single atomic change.
3. **Execute Move**: Perform exactly one move, such as planning a voyage, implementing a story, or fixing a diagnostic.
4. **Seal Move**: Close the loop with `story submit`, `voyage plan`, or `bearing lay`. This mutates the `.keel` state and may leave temporary worktree energy that should be cleared by the sealing commit.
5. **Log & Commit**:
   - Record your move in the mission `LOG.md`.
   - **Heartbeat Check**: Use `just keel heartbeat` if you need to inspect the current activity source or confirm the circuit is still energized before the commit boundary.
   - **Commit**: Execute `git commit`. The installed hook stack can run repo-local checks and append `doctor --status` to the commit message. Resolve any issues if the commit is rejected.
6. **Re-orient**: After the commit lands, run `just keel doctor --status` and `just keel flow --scene` to see what the board needs next.

This is the plug-the-chord-back-in moment: reconnect to the board's current state. If the delivery lane has ready work, start the next loop immediately. Only stop to ask the human when you reach a manual lane, such as design direction, bearing assessment, or human verification.

## Primary Workflows

### Operator (Implementation)
Focus on evidence-backed delivery.
- **Context**: `keel story show <id>` and `keel voyage show <id>`.
- **Action**: Implement requirements, record proofs with `keel story record`, and `submit`.
- **Constraint**: Every acceptance criterion must have a proof.

### Manager (Planning)
Focus on strategic alignment and unblocking.
- **Context**: `keel epic show <id>` and `keel flow --scene`.
- **Action**: Author `PRD.md`, `SRS.md`, `SDD.md`, and decompose stories.
- **Constraint**: Move voyages from `draft` to `planned` only when requirements are coherent.

### Explorer (Research)
Focus on technical discovery and fog reduction.
- **Context**: `keel bearing list`.
- **Action**: Fill `BRIEF.md`, collect `EVIDENCE.md`, and `assess`.
- **Constraint**: Graduate findings to epics only when research is conclusive.

## Human Interaction & Pokes

Keel's autonomous flow is governed by a physical battery metaphor, but the charge is derived from real repository activity rather than a synthetic wake file.

If a human user pokes you, for example "I'm poking you" or "Wake up", you MUST:
1. **Inspect the Charge**: Immediately execute `just keel heartbeat` to see whether recent repository activity is still energizing the board and whether the worktree is carrying uncommitted energy.
2. **Autonomous Scan**: Run `just keel mission next --status` and `just keel pulse` to identify new work that has become ready or materialized.
3. **Visual Confirmation**: Run `just keel flow --scene` to verify whether the light is ON or whether the board is idle waiting for a real move.

## Autonomous Backlog Discharge

As long as the system is AUTONOMOUS and the circuit is healthy, you are responsible for discharging the delivery backlog.

1. **Identify Ready Work**: Scan the delivery lane for stories in `backlog` that are not blocked by dependencies.
2. **Autonomous Start**: For each ready story, execute `keel story start <id>`.
3. **Rube Goldberg Loop**: Transitioning a story to `in-progress` mutates the repository, which refreshes the derived heartbeat and keeps the circuit closed.
4. **Priority**: Discharging the backlog is your primary tactical objective once energized. Continue until the backlog is empty or the circuit trips.
5. **Loop Closure**: After every successful implementation or transition, land a sealing commit that captures the resulting board and code changes. This applies to all work, including storyless gardening or engine changes. The pacemaker warning is cleared by committing the dirty worktree, not by touching a synthetic heartbeat file.

## Global Hygiene Checklist

Apply these checks to every change before finalizing work:

1. **Doctor First**: `just keel doctor` is the ultimate source of truth for board integrity. Run the doctor at the start of every session. If it reports errors or short circuits, fix those diagnostic orders before attempting other work or architectural changes.
2. **The Health Loop**: Use `just keel health --scene` for high-level triage. Subsystems are mapped as follows:
   - **NEURAL**: Stories (ID consistency, acceptance-criterion completion)
   - **MOTOR**: Voyages (structure, SRS/SDD authorship)
   - **STRATEGIC**: Epics (PRD and goal lineage)
   - **SENSORY**: Bearings (research and evidence quality)
   - **SKELETAL**: ADRs (architecture decisions)
   - **VITAL**: Missions (strategic achievement)
   - **AUTONOMIC**: Routines (cadence and materialization)
   - **CIRCULATORY**: Workflow (graph integrity and topology)
   - **PACEMAKER**: Heartbeat (derived repository activity and open-loop warning state)
   - **KINETIC**: Delivery (backlog liquidity and execution capacity)
3. **Pacemaker Protocol**: The system's heartbeat is derived from Git and worktree activity and inspected with `keel heartbeat`. A clean repo falls back to the latest commit, while a dirty repo uses the freshest changed path it can observe. `doctor` warns when the worktree carries uncommitted energy, and the sealing commit is what clears that warning. The installed hooks can keep quality checks and `doctor --status` metadata tied to the commit boundary.
4. **Gardening First**: Tend to the garden before notifying the human operator or requesting input.
5. **Notification Threshold**: Only request human intervention when you reach a manual lane that requires design direction or a decision on application behavior, such as assessing a bearing, planning a voyage, or human verification of a complex story.
6. **Automated Guardrails**: You do not need to run `just check` or `just test` manually before every commit. The installed hook stack via `just keel hooks install` can enforce repo-local checks and append doctor metadata. If a commit fails, resolve the reported issues and try again.
7. **Lifecycle Before Commit**: Run board-mutating lifecycle commands before the atomic commit when they generate or rewrite `.keel` artifacts, for example `story submit`, `voyage plan`, `voyage done`, `bearing assess`, or `bearing lay`. After the transition, inspect `git status` and include the resulting `.keel` churn in the same commit.
8. **Atomic Commits**: Commit once per logical unit of work. Use [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation
   - `refactor:` for code changes without behavior change
   - `test:` for adding or updating tests
   - `chore:` for build and tooling changes
9. **Mission Loop Discipline**: For mission-driven work, return to the mission steward loop after every completed story, planning unit, or bearing instead of continuing ad hoc from the last worker context.
10. **Knowledge Quality Bar**: Prefer no new knowledge over low-signal knowledge. A new knowledge entry should be novel, reusable across stories, and materially reduce future drift. Otherwise, link existing knowledge or omit capture entirely.
11. **Config Completeness**: Whenever introducing a new property to the configuration struct, immediately update the command or surface that renders runtime configuration so the new field is visible to operators.

## Compatibility Policy (Hard Cutover)

At this stage of development, this repository uses a hard cutover policy by default.

1. **No Backward Compatibility by Default**: Do not add compatibility aliases, dual-write logic, soft-deprecated schema fields, or fallback parsing for legacy formats unless a story explicitly requires it.
2. **Replace, Don’t Bridge**: When introducing a new canonical token, field, command behavior, or document contract, remove the old path in the same change slice.
3. **Fail Fast in Validation**: `keel doctor` and transition gates should treat legacy or unfilled scaffold patterns as hard failures when they violate the new contract.
4. **Single Canonical Path**: Keep one source of truth for rendering, parsing, and validation. Avoid parallel implementations meant only to preserve old behavior.
5. **Migration Is Explicit Work**: If existing board artifacts need updates, handle that in a dedicated migration pass or story instead of embedding runtime compatibility logic.

## Commands

### Command Execution Model

Use one path for each concern:

- `nix develop` for the repository shell and shared tooling.
- `just ...` for repo build, test, formatting, benchmarking, and example workflows.
- `keel ...` for all board and workflow operations.
- `just keel ...` as thin convenience wrappers for a small subset of board commands.

### `just` Workflow Commands

| Command | Purpose |
|---------|---------|
| `just` | List available recipes |
| `just fmt` | Format the workspace |
| `just fmt-check` | Check formatting |
| `just clippy` | Run workspace clippy |
| `just check` | Run formatting, clippy, main tests, and doc tests |
| `just test` | Run the main test suite with `cargo nextest` |
| `just test-doc` | Run doc tests |
| `just build [profile]` | Build `sift` for `debug` or `release` |
| `just build-static` | Build the static Nix artifact |
| `just sift ...` | Run the CLI via `cargo run --release` |
| `just embed-build` | Build the embedded example binary |
| `just embed-sift <path> <query>` | Run the embed example against a target path |
| `just embed-sift-here <query>` | Run the embed example against the current directory |
| `just bench` | Run benchmarks |
| `just flamegraph ...` | Run flamegraph profiling for eval flows |

### `keel` Board Workflow Commands

Run `keel --help` for the full command tree. The core commands you should rely on:

| Category | Commands |
|----------|----------|
| Discovery | `keel bearing new <name>` `keel bearing research <id>` `keel bearing assess <id>` `keel bearing list` |
| Planning | `keel epic new "<title>" --problem "<problem>"` `keel voyage new "<title>" --epic <epic-id> --goal "<goal>"` |
| Execution | `keel story new "<title>" [--type <type>] [--epic <epic-id> [--voyage <voyage-id>]]` |
| Board Ops | `keel mission next [<id>]` `keel next --role manager` `keel next --role operator` `keel flow --scene` `keel doctor` `keel health --scene` `keel generate` `keel config show` `keel mission show <id>` |
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
| Mission achieve | `keel mission achieve <id>` |
