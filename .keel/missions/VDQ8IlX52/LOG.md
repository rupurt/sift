# Public Release Preparation - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-09T17:46:10

Added Homebrew platform support. Configured dedicated tap 'rupurt/homebrew-tap' and updated RELEASE.md with installation instructions.

## 2026-03-09T17:51:09

Implemented static Linux executable support via 'musl' target. Synchronized keel board state and finalized release pipeline.

## 2026-03-09T17:52:08

Fixed TOML syntax error in Cargo.toml where homebrew configuration was incorrectly nested under an array key.
