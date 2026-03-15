# VOYAGE REPORT: Restore Zvec Build Compatibility

## Voyage Metadata
- **ID:** 1vzGCb000
- **Epic:** 1vzGCU000
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Stabilize Zvec Build Toolchain
- **ID:** 1vzGCp000
- **Status:** done

#### Summary
Restore the default `sift` build path by aligning the repository shell and the
vendored `zvec-sys` crate with the actual native toolchain constraints exposed
by `zvec v0.2.0`.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `nix develop --command cmake --version` reports a CMake 3.x release from the compatibility input. <!-- verify: nix develop --command cmake --version contains "cmake version 3.", SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] `nix develop --command sh -lc 'echo $CC; $CC --version | head -n 1'` resolves to the compatibility GCC 14 wrapper. <!-- verify: nix develop --command sh -lc 'echo $CC; $CC --version | head -n 1' contains "gcc-wrapper-14.3.0", SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-01] The vendored `zvec-sys` patch builds `zvec` and the C wrapper from isolated `OUT_DIR` paths and matches the `zvec v0.2.0` `AddColumn` signature. <!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-04/AC-01] `nix develop --command cargo check` completes successfully without manual `PATH`, `CC`, or `CXX` overrides. <!-- verify: nix develop --command cargo check, SRS-04:start:end, proof: ac-4.log -->
- [x] [SRS-05/AC-01] The compatibility pin is limited to the required native build components, and the shell does not carry an unused alternate compiler toolchain. <!-- verify: manual, SRS-05:start:end, proof: ac-5.log -->

#### Verified Evidence
- [ac-1.log](../../../../stories/1vzGCp000/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/1vzGCp000/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/1vzGCp000/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/1vzGCp000/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/1vzGCp000/EVIDENCE/ac-5.log)


