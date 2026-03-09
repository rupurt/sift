all corpus=".cache/eval/scifact-files" qrels=".cache/eval/scifact/qrels/test.tsv":
    cargo run --release -- bench all --corpus "{{corpus}}" --qrels "{{qrels}}"

hybrid corpus=".cache/eval/scifact-files" qrels=".cache/eval/scifact/qrels/test.tsv" queries=".cache/eval/scifact-files/test-queries.tsv":
    cargo run --release -- bench quality --strategy "hybrid" --baseline "bm25" --corpus "{{corpus}}" --qrels "{{qrels}}"
    cargo run --release -- bench latency --strategy "hybrid" --corpus "{{corpus}}" --queries "{{queries}}"

baseline corpus=".cache/eval/scifact-files" qrels=".cache/eval/scifact/qrels/test.tsv" queries=".cache/eval/scifact-files/test-queries.tsv":
    cargo run --release -- bench quality --strategy "bm25" --corpus "{{corpus}}" --qrels "{{qrels}}"
    cargo run --release -- bench latency --strategy "bm25" --corpus "{{corpus}}" --queries "{{queries}}"
