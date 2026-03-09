download-scifact out=".cache/eval/scifact":
    cargo run -- eval download --out "{{out}}" scifact

materialize-scifact source=".cache/eval/scifact" out=".cache/eval/scifact-files":
    cargo run -- eval materialize --source "{{source}}" --out "{{out}}" scifact
