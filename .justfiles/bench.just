prepare:
    just eval download-scifact
    just eval materialize-scifact

all corpus=".cache/eval/scifact-files" qrels=".cache/eval/scifact/qrels/test.tsv": prepare
    corpus_path=$(case "{{corpus}}" in /*) printf "%s" "{{corpus}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{corpus}}" ;; esac); \
    qrels_path=$(case "{{qrels}}" in /*) printf "%s" "{{qrels}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{qrels}}" ;; esac); \
    cargo run --release -- bench all --corpus "$corpus_path" --qrels "$qrels_path"

hybrid corpus=".cache/eval/scifact-files" qrels=".cache/eval/scifact/qrels/test.tsv" queries=".cache/eval/scifact-files/test-queries.tsv": prepare
    corpus_path=$(case "{{corpus}}" in /*) printf "%s" "{{corpus}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{corpus}}" ;; esac); \
    qrels_path=$(case "{{qrels}}" in /*) printf "%s" "{{qrels}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{qrels}}" ;; esac); \
    queries_path=$(case "{{queries}}" in /*) printf "%s" "{{queries}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{queries}}" ;; esac); \
    cargo run --release -- bench quality --strategy "hybrid" --baseline "bm25" --corpus "$corpus_path" --qrels "$qrels_path"; \
    cargo run --release -- bench latency --strategy "hybrid" --corpus "$corpus_path" --queries "$queries_path"

baseline corpus=".cache/eval/scifact-files" qrels=".cache/eval/scifact/qrels/test.tsv" queries=".cache/eval/scifact-files/test-queries.tsv": prepare
    corpus_path=$(case "{{corpus}}" in /*) printf "%s" "{{corpus}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{corpus}}" ;; esac); \
    qrels_path=$(case "{{qrels}}" in /*) printf "%s" "{{qrels}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{qrels}}" ;; esac); \
    queries_path=$(case "{{queries}}" in /*) printf "%s" "{{queries}}" ;; *) printf "%s/%s" "{{justfile_directory()}}" "{{queries}}" ;; esac); \
    cargo run --release -- bench quality --strategy "bm25" --corpus "$corpus_path" --qrels "$qrels_path"; \
    cargo run --release -- bench latency --strategy "bm25" --corpus "$corpus_path" --queries "$queries_path"
