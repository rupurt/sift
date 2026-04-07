---
# system-managed
id: VG7SznTh9
status: done
created_at: 2026-04-07T08:29:33
updated_at: 2026-04-07T08:32:42
# authored
title: Upgrade Keel Flake Input
type: chore
operator-signal:
started_at: 2026-04-07T08:30:09
completed_at: 2026-04-07T08:32:42
---

# Upgrade Keel Flake Input

## Summary

Advance the repository's `keel` flake input to the latest upstream `main`
revision and confirm the updated tool still works with this board and the
repo-local `just keel ...` wrappers.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `flake.lock` updates the `keel` input from the current pinned revision to the latest upstream `main` revision without changing the flake wiring in `flake.nix`. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "\"keel\": \\{" flake.lock && rg -n "\"rev\": \"e7a2a2ed00dd7342b94ac56a4eb32a3b683f4242\"" flake.lock && rg -n "git\\+ssh://git@github.com/spoke-sh/keel.git|ssh://git@github.com/spoke-sh/keel.git" flake.nix flake.lock', SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] The upgraded tool evaluates inside the repo shell and `just keel doctor --status` completes successfully against the updated input. <!-- verify: nix develop -c sh -lc 'cd "$(git rev-parse --show-toplevel)" && just keel doctor --status', SRS-02:start:end, proof: ac-2.log-->
