download *args:
    cargo run --release -- dataset download {{args}}

materialize *args:
    cargo run --release -- dataset materialize {{args}}

prepare *args:
    @echo "Preparing SciFact dataset..."
    @just dataset download scifact {{args}} > /dev/null
    @just dataset materialize scifact {{args}} > /dev/null
