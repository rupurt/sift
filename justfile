mod keel '.justfiles/keel.just'
mod eval '.justfiles/eval.just'
mod dataset '.justfiles/dataset.just'

default:
    @echo "Root recipes:"
    @just --list
    @echo
    @echo "Keel module:"
    @just --list keel
    @echo
    @echo "Eval module:"
    @just --list eval
    @echo
    @echo "Dataset module:"
    @just --list dataset

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

build profile="debug":
    @set -eu; \
    local_target="{{justfile_directory()}}/target"; \
    case "{{profile}}" in \
        debug) cargo build; profile_dir=debug ;; \
        release) cargo build --release; profile_dir=release ;; \
        *) echo "unsupported build profile: {{profile}} (expected debug or release)" >&2; exit 1 ;; \
    esac; \
    source_root="${CARGO_TARGET_DIR:-$local_target}"; \
    source_bin="$source_root/$profile_dir/sift"; \
    dest_dir="$local_target/$profile_dir"; \
    dest_bin="$dest_dir/sift"; \
    if [ ! -f "$source_bin" ]; then \
        echo "expected build artifact not found: $source_bin" >&2; \
        exit 1; \
    fi; \
    mkdir -p "$dest_dir"; \
    if [ "$source_bin" != "$dest_bin" ]; then \
        cp -f "$source_bin" "$dest_bin"; \
        echo "copied $source_bin -> $dest_bin"; \
    fi

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo nextest run

test-doc:
    cargo test --doc

check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo nextest run
    cargo test --doc

build-static:
    @if [ -f "flake.nix" ]; then \
        echo "Building static binary using Nix..."; \
        nix build .#sift-static --out-link target/release/sift-static; \
        echo "Static binary created at target/release/sift-static/bin/sift"; \
    else \
        echo "Static build only supported via Nix in this recipe."; \
        exit 1; \
    fi

search features="" *args:
    cargo run --release --features "{{features}}" -- search {{args}}

config:
    cargo run --release -- config

eval-flamegraph *args:
    cargo flamegraph -- eval {{args}}

eval-micro:
    cargo bench
