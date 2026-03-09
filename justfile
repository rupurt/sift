mod keel '.justfiles/keel.just'
mod bench '.justfiles/bench.just'
mod eval '.justfiles/eval.just'

default:
    @echo "Root recipes:"
    @just --list
    @echo
    @echo "Keel module:"
    @just --list keel
    @echo
    @echo "Bench module:"
    @just --list bench
    @echo
    @echo "Eval module:"
    @just --list eval

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

search query path:
    cargo run -- search "{{query}}" "{{path}}"

search-json query path:
    cargo run -- search --json "{{query}}" "{{path}}"
