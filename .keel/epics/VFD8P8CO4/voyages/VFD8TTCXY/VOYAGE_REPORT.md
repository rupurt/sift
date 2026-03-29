# VOYAGE REPORT: Benchmark Graph Search Against Linear Baselines

## Voyage Metadata
- **ID:** VFD8TTCXY
- **Epic:** VFD8P8CO4
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Benchmark Graph Search Against Linear Autonomous Baselines
- **ID:** VFDQ2VZnq
- **Status:** done

#### Summary
Benchmark graph search against the shipped linear autonomous path, planned
controller fixtures, and collapsed single-turn baselines on the same tasks.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Graph search is evaluated against linear autonomous planning, planned-controller fixtures, and collapsed single-turn baselines on shared task sets. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-NFR-01/AC-02] Graph evaluation does not regress the current linear evaluation path when graph mode is not selected. <!-- verify: manual, SRS-NFR-01:start:end -->

### Report Graph Metrics And Comparative Deltas
- **ID:** VFDQ2Vkov
- **Status:** done

#### Summary
Emit graph-specific evaluation metrics and replayable report artifacts so graph
and linear planner tradeoffs remain inspectable in regression review.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] Evaluation reports expose graph-specific metrics such as frontier expansion cost, merge or prune counts, or branch efficiency. <!-- verify: manual, SRS-02:start:end -->
- [x] [SRS-03/AC-02] Graph evaluation artifacts remain replayable and suitable for regression comparison across graph and linear planner revisions. <!-- verify: manual, SRS-03:start:end -->


