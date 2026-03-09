download-scifact out=".cache/eval/scifact":
    out_path=$(case "{{out}}" in /*) printf "%s" "{{out}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{out}}" ;; esac); cargo run -- eval download --out "$out_path" scifact

materialize-scifact source=".cache/eval/scifact" out=".cache/eval/scifact-files":
    source_path=$(case "{{source}}" in /*) printf "%s" "{{source}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{source}}" ;; esac); out_path=$(case "{{out}}" in /*) printf "%s" "{{out}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{out}}" ;; esac); cargo run -- eval materialize --source "$source_path" --out "$out_path" scifact
