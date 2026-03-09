all *args:
    @echo "Running comparative evaluations for all strategies..."
    cargo run --release --quiet -- eval all {{args}}

hybrid *args:
    @echo "Running comprehensive evaluation for 'page-index-hybrid' champion..."
    cargo run --release --quiet -- eval quality --strategy "page-index-hybrid" --baseline "bm25" {{args}}
    cargo run --release --quiet -- eval latency --strategy "page-index-hybrid" {{args}}

baseline *args:
    @echo "Running comprehensive evaluation for 'bm25' baseline..."
    cargo run --release --quiet -- eval quality --strategy "bm25" {{args}}
    cargo run --release --quiet -- eval latency --strategy "bm25" {{args}}

quality *args:
    cargo run --release --quiet -- eval quality {{args}}

latency *args:
    cargo run --release --quiet -- eval latency {{args}}
