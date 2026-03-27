# VOYAGE REPORT: Define Turn Model and Emission Contract

## Voyage Metadata
- **ID:** VF1Vt0WCT
- **Epic:** VF1VhxmqI
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Add Turn-Native Search Domain And Controller Requests
- **ID:** VF1VuFBgx
- **Status:** done

#### Summary
Add the first explicit turn-oriented search records so controller state and trace data stop living only in design docs and implicit runtime behavior.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Turn-oriented search request and response records exist for controller-facing execution. <!-- verify: manual, SRS-01:start:end -->
- [x] [SRS-02/AC-02] Trace records exist for turn progression and controller decisions. <!-- verify: manual, SRS-02:start:end -->

### Add Explicit Emission Modes And Agentic Outputs
- **ID:** VF1VuFDgy
- **Status:** done

#### Summary
Define explicit emission modes so the retrieval substrate can return view, protocol, or latent-oriented outputs without overloading the existing file-hit shape.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Explicit emission modes are defined in supported search contracts. <!-- verify: manual, SRS-03:start:end -->
- [x] [SRS-NFR-02/AC-02] Emission-oriented outputs remain inspectable for traces and tests. <!-- verify: manual, SRS-NFR-02:start:end -->

### Expose Stable Agentic Entry Points In The Public Facade
- **ID:** VF1VuFbgz
- **Status:** done

#### Summary
Promote the new turn and emission contracts through a supported public API so embedders do not need to depend on `sift::internal`.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] A supported facade or crate-root entry point exposes the new contracts. <!-- verify: manual, SRS-04:start:end -->
- [x] [SRS-NFR-01/AC-02] Existing single-turn hybrid callers remain supported during the cutover. <!-- verify: manual, SRS-NFR-01:start:end -->


