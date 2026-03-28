# Introduce Local Autonomous Planning and Decomposition - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-28T15:21:56-07:00

- Completed voyage `VFCW6PVzz` `Ship Heuristic Planner Baseline`.
- Added public `HeuristicAutonomousPlanner` contracts and tests for initial query generation, retained-evidence follow-up deduplication, explicit stop reasons, and deterministic bounded replay.
- Verified with `cargo test --test autonomous_heuristic_planner_test` and `just check`.

## 2026-03-28T15:29:35-07:00

- Completed voyage `VFCW85Y1r` `Wire Strategy-Selected Autonomous Runtime`.
- Added `Sift::search_autonomous`, verified built-in heuristic execution without custom planner injection, and proved resume/controller reuse behavior through facade tests.
- Verified with `cargo test --test library_facade_test` and `just check`.

## 2026-03-28T15:31:49-07:00

- Completed voyage `VFCW9fu6V` `Add Model-Driven Planner Strategy` and finalized epic `VFC7H4QFx`.
- Added `ModelDrivenAutonomousPlanner`, in-process generative model injection for library tests/runtime selection, and shared strategy/profile routing through `Sift::search_autonomous`.
- Verified with `cargo test --test library_facade_test`, planner contract tests in `src/search/planner.rs`, and `just check`.
