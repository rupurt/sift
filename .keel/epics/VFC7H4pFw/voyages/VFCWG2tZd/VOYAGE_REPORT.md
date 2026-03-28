# VOYAGE REPORT: Add Agent Search CLI Surface

## Voyage Metadata
- **ID:** VFCWG2tZd
- **Epic:** VFC7H4pFw
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Add Autonomous Planner Flag To Search CLI
- **ID:** VFCWVwR6R
- **Status:** done

#### Summary
Add planner-driven search to the shipped executable through `sift search
--agent`, reusing the shared autonomous runtime instead of inventing a CLI-only
planner path.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `sift search --agent` invokes the shared autonomous runtime from the executable. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-02/AC-02] Agent-mode CLI output and JSON expose planner strategy and autonomous trace metadata suitable for inspection and downstream tooling. <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFCWVwR6R/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFCWVwR6R/EVIDENCE/ac-2.log)

### Preserve Non-Agent Search UX
- **ID:** VFCWVxEvA
- **Status:** done

#### Summary
Preserve the current non-agent search command behavior and surrounding
search/evaluation workflows while the executable gains an explicit agent-mode
surface.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Existing non-agent `sift search` behavior remains unchanged when `--agent` is not selected. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-02] The CLI agent surface continues to call the same runtime contract as the supported library autonomous path. <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-02/AC-03] Existing search and evaluation commands do not regress when the agent flag is added. <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFCWVxEvA/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFCWVxEvA/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFCWVxEvA/EVIDENCE/ac-3.log)


