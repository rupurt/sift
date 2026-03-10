# Embeddable Library and Executable Packaging - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-10T15:30:00-07:00

Started a packaging research slice for turning `sift` into an embeddable Rust
library while preserving the existing executable.

Initial repo inspection showed that:

- the crate already builds both `src/lib.rs` and `src/main.rs`
- the binary already consumes the library directly
- the main issue is not feasibility but public API hygiene and packaging shape

Research will compare a curated single-package facade, an immediate workspace
split, and a deeper internal multi-crate decomposition.

## 2026-03-10T15:31:19

Research concluded that sift should stay a single package for now, add a canonical embedded facade, remove CLI leakage from the public API, and defer any workspace split until the facade proves insufficient. Seeded epic VDVQurZER and planned voyage VDVRkNjgH with four implementation stories.
