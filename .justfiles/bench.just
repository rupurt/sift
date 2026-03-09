quality corpus=".cache/eval/scifact-files" qrels=".cache/eval/scifact/qrels/test.tsv" engine="hybrid" baseline="bm25":
    cargo run --release -- bench quality --engine "{{engine}}" --baseline "{{baseline}}" --corpus "{{corpus}}" --qrels "{{qrels}}"

latency corpus=".cache/eval/scifact-files" queries=".cache/eval/scifact-files/test-queries.tsv" engine="hybrid":
    cargo run --release -- bench latency --engine "{{engine}}" --corpus "{{corpus}}" --queries "{{queries}}"
