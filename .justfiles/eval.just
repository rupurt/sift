download-scifact *args:
    cargo run --release -- eval download scifact {{args}}

materialize-scifact *args:
    cargo run --release -- eval materialize scifact {{args}}
