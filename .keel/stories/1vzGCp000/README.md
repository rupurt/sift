---
id: 1vzGCp000
title: Stabilize Zvec Build Toolchain
type: fix
scope: 1vzGCU000/1vzGCb000
status: done
created_at: 2026-03-08T08:38:11
updated_at: 2026-03-08T09:22:03
started_at: 2026-03-08T08:38:44
submitted_at: 2026-03-08T09:21:48
completed_at: 2026-03-08T09:22:03
---

# Stabilize Zvec Build Toolchain

## Summary

Restore the default `sift` build path by aligning the repository shell and the
vendored `zvec-sys` crate with the actual native toolchain constraints exposed
by `zvec v0.2.0`.

## Acceptance Criteria

- [x] [SRS-01/AC-01] `nix develop --command cmake --version` reports a CMake 3.x release from the compatibility input. <!-- verify: nix develop --command cmake --version contains "cmake version 3.", SRS-01:start:end, proof: ac-1.log -->
- [x] [SRS-02/AC-01] `nix develop --command sh -lc 'echo $CC; $CC --version | head -n 1'` resolves to the compatibility GCC 14 wrapper. <!-- verify: nix develop --command sh -lc 'echo $CC; $CC --version | head -n 1' contains "gcc-wrapper-14.3.0", SRS-02:start:end, proof: ac-2.log -->
- [x] [SRS-03/AC-01] The vendored `zvec-sys` patch builds `zvec` and the C wrapper from isolated `OUT_DIR` paths and matches the `zvec v0.2.0` `AddColumn` signature. <!-- verify: manual, SRS-03:start:end, proof: ac-3.log -->
- [x] [SRS-04/AC-01] `nix develop --command cargo check` completes successfully without manual `PATH`, `CC`, or `CXX` overrides. <!-- verify: nix develop --command cargo check, SRS-04:start:end, proof: ac-4.log -->
- [x] [SRS-05/AC-01] The compatibility pin is limited to the required native build components, and the shell does not carry an unused alternate compiler toolchain. <!-- verify: manual, SRS-05:start:end, proof: ac-5.log -->
