---
id: VE1xpzs5w
title: Implement Sift Optimize Command
type: feat
status: done
created_at: 2026-03-16T04:56:40
updated_at: 2026-03-27T17:27:18
operator-signal: 
scope: VE1xUOaxK/VE1xdk4hF
index: 3
started_at: 2026-03-16T05:16:28
submitted_at: 2026-03-26T16:01:05
completed_at: 2026-03-27T17:27:18
---

# Implement Sift Optimize Command

## Summary

This story introduces the `sift optimize` CLI command, an automated offline loop that mutates prompts using the local LLM and evaluates them to maximize Signal Gain against a test dataset.

## Acceptance Criteria

- [x] [SRS-03/AC-01] Create `sift optimize` CLI command. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Implement greedy hill-climbing optimization loop over `test-queries.tsv` and `qrels`. <!-- verify: manual, SRS-03:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-03] Save highest-yielding prompts to `./sift.toml`. <!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-04/AC-04] Ensure LLM generation errors are handled gracefully without crashing the loop. <!-- verify: manual, SRS-04:start:end, proof: ac-4.log -->
