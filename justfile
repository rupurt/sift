mod keel '.justfiles/keel.just'

default:
    @echo "Root recipes:"
    @just --list
    @echo
    @echo "Keel module:"
    @just --list keel

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

build profile="debug":
    @set -eu; \
    local_target="{{justfile_directory()}}/target"; \
    case "{{profile}}" in \
        debug) \
            cargo build; \
            profile_dir=debug; \
            source_root="${CARGO_TARGET_DIR:-$local_target}"; \
            source_bin="$source_root/$profile_dir/sift" \
        ;; \
        release) \
            cargo build --release; \
            profile_dir=release; \
            source_root="${CARGO_TARGET_DIR:-$local_target}"; \
            source_bin="$source_root/$profile_dir/sift"; \
        ;; \
        *) echo "unsupported build profile: {{profile}} (expected debug or release)" >&2; exit 1 ;; \
    esac; \
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

sift *args:
    @bash -eu -c '\
        cargo_args=(--release); \
        env_args=(); \
        sift_args=(); \
        for arg in "$@"; do \
            if [ "$arg" = "--cuda" ]; then \
                cargo_args+=(--features cuda); \
                env_args+=("SIFT_DENSE_DEVICE=${SIFT_DENSE_DEVICE:-cpu}"); \
            else \
                sift_args+=("$arg"); \
            fi; \
        done; \
        env "${env_args[@]}" cargo run "${cargo_args[@]}" -- "${sift_args[@]}" \
    ' -- {{args}}

embed-build:
    @set -eu; \
    local_target="{{justfile_directory()}}/target"; \
    manifest="{{justfile_directory()}}/examples/sift-embed/Cargo.toml"; \
    cargo build --manifest-path "$manifest"; \
    profile_dir=debug; \
    source_root="${CARGO_TARGET_DIR:-$local_target}"; \
    source_bin="$source_root/$profile_dir/sift-embed"; \
    dest_dir="$local_target/$profile_dir"; \
    dest_bin="$dest_dir/sift-embed"; \
    if [ ! -f "$source_bin" ]; then \
        echo "expected build artifact not found: $source_bin" >&2; \
        exit 1; \
    fi; \
    mkdir -p "$dest_dir"; \
    if [ "$source_bin" != "$dest_bin" ]; then \
        cp -f "$source_bin" "$dest_bin"; \
        echo "copied $source_bin -> $dest_bin"; \
    fi

embed-sift path query:
    cargo run --manifest-path examples/sift-embed/Cargo.toml -- search '{{path}}' '{{query}}'

embed-sift-here query:
    cargo run --manifest-path examples/sift-embed/Cargo.toml -- search '{{query}}'

flamegraph *args:
    cargo flamegraph --root="-E" -- eval {{args}}

bench:
    cargo bench
