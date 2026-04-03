# Add Stable Local Model Preparation Seam - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Expose a stable sift-owned model-preparation API so downstream embedders can prepare local model artifacts without reaching into `sift::internal::*`. | board: VFkJD2NwK |
| MG-02 | Prove a representative GGUF source can be prepared into the current Candle-loadable bundle contract through metamorph-backed conversion. | board: VFkJD2NwK |
| MG-03 | Document the seam honestly as a compatibility/preparation path rather than native 1-bit runtime support. | board: VFkJD2NwK |

## Constraints

- MUST keep sift as the owner of runtime preparation for downstream integrators.
- MUST preserve the local-first, library-friendly, zero-daemon repository contract.
- MUST keep paddles-specific provider or UI concerns out of sift.
- MUST keep metamorph separate; integrate it behind the sift seam rather than absorbing it into sift internals.
- MUST treat GGUF -> Candle safetensors preparation as a compatibility path and document the efficiency tradeoff explicitly.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when the stable preparation seam, representative conversion proof, and compatibility framing are all delivered through epic `VFkJD2NwK`
- YIELD to human when only `metric:` or `manual:` goals remain
