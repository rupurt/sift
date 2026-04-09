---
# system-managed
id: VGKxupVEs
status: done
created_at: 2026-04-09T15:53:54
updated_at: 2026-04-09T15:56:38
# authored
title: Set Keel Working Hours To 7x7
type: chore
operator-signal:
started_at: 2026-04-09T15:54:01
completed_at: 2026-04-09T15:56:38
---

# Set Keel Working Hours To 7x7

## Summary

Tighten the Keel workflow schedule from an all-day window to a daily
`07:00-19:00` operating window while keeping the board open seven days a week
and preserving a clean board health report under the current `keel` binary.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `keel.toml` sets workflow hours to `07:00-19:00` and keeps all seven weekdays listed in `working_days`. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "working_hours_start = 7" keel.toml && rg -n "working_hours_end = 19" keel.toml && rg -n "\"Monday\"" keel.toml && rg -n "\"Tuesday\"" keel.toml && rg -n "\"Wednesday\"" keel.toml && rg -n "\"Thursday\"" keel.toml && rg -n "\"Friday\"" keel.toml && rg -n "\"Saturday\"" keel.toml && rg -n "\"Sunday\"" keel.toml', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] `keel config show` resolves the effective schedule to `working_hours_start = 7` and `working_hours_end = 19`, and `just keel doctor --status` remains structurally clean after the change. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && keel config show | rg -n "working_hours_start.*7|working_hours_end.*19|mission-completion-evidence|disabled.*true" && just keel doctor --status', SRS-02:start:end, proof: ac-2.log -->
