# VOYAGE REPORT: Configurable Prompts

## Voyage Metadata
- **ID:** VE1xdk4hF
- **Epic:** VE1xUOaxK
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Add Prompt Configuration to Sift Toml
- **ID:** VE1xpmsiN
- **Status:** done

#### Summary
This story adds the `[prompts]` section to the `sift.toml` configuration structure, enabling users to define custom prompts for generative query expansion strategies.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Add `prompts` section to `Config` struct. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Parse prompts correctly from `sift.toml`. <!-- verify: manual, SRS-01:start:end, proof: ac-2.log -->
- [x] [SRS-02/AC-03] Provide default fallback constants. <!-- verify: manual, SRS-02:start:end, proof: ac-3.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VE1xpmsiN/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VE1xpmsiN/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VE1xpmsiN/EVIDENCE/ac-3.log)

### Update Expanders to Use Configured Prompts
- **ID:** VE1xptJ3y
- **Status:** done

#### Summary
This story refactors the `SearchServiceBuilder` and expansion strategies to utilize the prompts loaded from the configuration, making the expansion process dynamic.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Update `SearchServiceBuilder` to read prompts from config. <!-- verify: manual, SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-01/AC-02] Refactor `HydeStrategy`, `SpladeStrategy`, and `ClassifiedStrategy` to accept configurable prompts. <!-- verify: manual, SRS-01:start:end, proof: ac-2.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VE1xptJ3y/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VE1xptJ3y/EVIDENCE/ac-2.log)

### Implement Sift Optimize Command
- **ID:** VE1xpzs5w
- **Status:** done

#### Summary
This story introduces the `sift optimize` CLI command, an automated offline loop that mutates prompts using the local LLM and evaluates them to maximize Signal Gain against a test dataset.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Create `sift optimize` CLI command. <!-- verify: manual, SRS-03:start:end, proof: ac-1.log -->
- [x] [SRS-03/AC-02] Implement greedy hill-climbing optimization loop over `test-queries.tsv` and `qrels`. <!-- verify: manual, SRS-03:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-03] Save highest-yielding prompts to `./sift.toml`. <!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-04/AC-04] Ensure LLM generation errors are handled gracefully without crashing the loop. <!-- verify: manual, SRS-04:start:end, proof: ac-4.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/VE1xpzs5w/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VE1xpzs5w/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VE1xpzs5w/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VE1xpzs5w/EVIDENCE/ac-4.log)


