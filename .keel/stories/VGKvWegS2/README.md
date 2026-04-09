---
# system-managed
id: VGKvWegS2
status: done
created_at: 2026-04-09T15:44:25
updated_at: 2026-04-09T15:46:07
# authored
title: Refresh Keel Flake Input
type: chore
operator-signal:
started_at: 2026-04-09T15:44:29
completed_at: 2026-04-09T15:46:07
---

# Refresh Keel Flake Input

## Summary

Advance the repository's `keel` flake input from the current pinned revision to
the latest upstream `main` revision and confirm the updated tool still works
inside the repo shell with the repo-local `just keel ...` wrappers.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `flake.lock` updates the `keel` input from `e7a2a2ed00dd7342b94ac56a4eb32a3b683f4242` to upstream `main` revision `c90a1a3ac12c365082dd3d0d8ddb31502de9afaf` without changing the flake wiring in `flake.nix`. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "\"keel\": \\{" flake.lock && rg -n "\"rev\": \"c90a1a3ac12c365082dd3d0d8ddb31502de9afaf\"" flake.lock && rg -n "git\\+ssh://git@github.com/spoke-sh/keel.git|ssh://git@github.com/spoke-sh/keel.git" flake.nix flake.lock', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] The upgraded tool evaluates inside the repo shell, reports a version, and `just keel doctor --status` completes successfully against the updated input. <!-- verify: nix develop -c sh -lc 'cd "$(git rev-parse --show-toplevel)" && keel --version && just keel doctor --status', SRS-02:start:end, proof: ac-2.log -->
