prepare *args:
    @echo "Preparing SciFact dataset..."
    @just eval download-scifact {{args}} > /dev/null
    @just eval materialize-scifact {{args}} > /dev/null

all *args: prepare
    @echo "Running comparative benchmarks for all strategies..."
    @# We can't easily parse named args from *args in just without a lot of shell logic.
    @# For now, let's just pass everything through.
    cargo run --release --quiet -- bench all {{args}}

hybrid *args: prepare
    @echo "Running comprehensive benchmark for 'hybrid' champion..."
    cargo run --release --quiet -- bench quality --strategy "hybrid" --baseline "bm25" {{args}}
    cargo run --release --quiet -- bench latency --strategy "hybrid" {{args}}

baseline *args: prepare
    @echo "Running comprehensive benchmark for 'bm25' baseline..."
    cargo run --release --quiet -- bench quality --strategy "bm25" {{args}}
    cargo run --release --quiet -- bench latency --strategy "bm25" {{args}}
