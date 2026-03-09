all corpus=".cache/eval/scifact-files" qrels=".cache/eval/scifact/qrels/test.tsv":
    cargo run --release -- bench all --corpus "{{corpus}}" --qrels "{{qrels}}"

quality corpus=".cache/eval/scifact-files" qrels=".cache/eval/scifact/qrels/test.tsv" strategy="hybrid" baseline="bm25":
    cargo run --release -- bench quality --strategy "{{strategy}}" --baseline "{{baseline}}" --corpus "{{corpus}}" --qrels "{{qrels}}"

latency corpus=".cache/eval/scifact-files" queries=".cache/eval/scifact-files/test-queries.tsv" strategy="hybrid":
    cargo run --release -- bench latency --strategy "{{strategy}}" --corpus "{{corpus}}" --queries "{{queries}}"
