# VOYAGE REPORT: Refresh Foundational Structural Retrieval Docs

## Voyage Metadata
- **ID:** VG8oUK3wL
- **Epic:** VG8oL9Xok
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Refresh Structural Retrieval Foundational Docs
- **ID:** VG8oeF6U9
- **Status:** done

#### Summary
Refresh the foundational documentation so the shipped structural retrieval
surface is described consistently across conceptual, operator, embedder, and
release-facing docs, with explicit strategy guidance and downstream direct-
search adoption notes for `paddles`-style embedders.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The foundational docs describe the shipped structural retrieval stack in terms that match the code and public surface. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "path-fuzzy|segment-fuzzy|PositionAwareReranker|SearchPlan::default_page_index_hybrid" README.md WORLD.md ARCHITECTURE.md CONFIGURATION.md EVALUATIONS.md RESEARCH.md LIBRARY.md CONSTITUTION.md RELEASE.md', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-02] The docs explain strategy selection, including when `path-hybrid` and the page-index family should be used. <!-- verify: manual, SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-03] The docs explain the downstream `paddles` adoption seam without implying a planner boundary change. <!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-04/AC-04] The foundational set remains internally consistent after the update, including process or release-facing references. <!-- verify: manual, SRS-04:start:end, proof: ac-4.log -->


