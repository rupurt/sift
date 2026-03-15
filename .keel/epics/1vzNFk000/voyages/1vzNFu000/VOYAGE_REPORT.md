# VOYAGE REPORT: Refresh README For Current Retrieval Contract

## Voyage Metadata
- **ID:** 1vzNFu000
- **Epic:** 1vzNFk000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Rewrite README For Current CLI Contract
- **ID:** 1vzNHY000
- **Status:** done

#### Summary
Rewrite `README.md` so it matches the current sift CLI, supported formats,
architecture constraints, and recorded benchmark evidence.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The README replaces stale `index`/`zvec`/disk-backed/API-embedding claims with the current indexless `search`, `bench`, and `eval` contract. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "zvec|sift index|Indexing a directory|disk-backed indices|API-based embeddings" README.md && cargo run -- --help && cargo run -- search --help && cargo run -- bench --help && cargo run -- eval --help', SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] The README includes executable usage examples that map to the current CLI commands and options. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "sift search" README.md && rg -n "sift bench quality" README.md && rg -n "sift bench latency" README.md && rg -n "sift eval download" README.md && rg -n "sift eval materialize" README.md && cargo run -- --help && cargo run -- search --help && cargo run -- bench quality --help && cargo run -- bench latency --help && cargo run -- eval download --help && cargo run -- eval materialize --help', SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-01] The README documents the supported document families and architectural constraints now implemented by sift. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "HTML" README.md && rg -n "PDF" README.md && rg -n "docx" README.md && rg -n "xlsx" README.md && rg -n "pptx" README.md && rg -n "Single Rust binary" README.md && rg -n "No external database" README.md && rg -n "No persisted database or background indexing service" README.md', SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-04/AC-01] The README summarizes measured hybrid benchmark evidence and rich-document support already recorded on the board. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "SciFact run over" README.md && rg -n "0\\.6647" README.md && rg -n "0\\.6764" README.md && rg -n "0\\.6328" README.md && rg -n "0\\.6466" README.md && rg -n "170\\.2 ms" README.md && rg -n "180\\.8 ms" README.md && rg -n "214\\.7 ms" README.md && rg -n "all-MiniLM-L6-v2" README.md', SRS-04:start:end, proof: ac-4.log -->
- [x] [SRS-05/AC-01] The README cutover removes contradictory obsolete claims instead of documenting both the old and new product shapes. <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "zvec|sift index|disk-backed indices|API-based embeddings" README.md', SRS-05:start:end, proof: ac-5.log -->

#### Implementation Insights
- **1vzNKa000: Avoid literal commas inside Keel verify command fields**
  - Insight: Keel's verify annotation parser treats commas as structural separators, so a literal comma inside the command field can corrupt parsing and produce misleading verification failures even when the underlying shell command succeeds.
  - Suggested Action: Keep verify command strings comma-free or express the same proof with alternate probes before relying on `keel verify run`.
  - Applies To: `.keel/stories/*/README.md`, verification annotations, documentation stories
  - Category: process


#### Verified Evidence
- [ac-1.log](../../../../stories/1vzNHY000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzNHY000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzNHY000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vzNHY000/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/1vzNHY000/EVIDENCE/ac-5.log)


