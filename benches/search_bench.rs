use criterion::{Criterion, criterion_group, criterion_main};
use sift::internal::{search::tokenize, vector::dot_product};
use std::hint::black_box;

fn bench_tokenize(c: &mut Criterion) {
    let text = "This is a sample text that we will use to benchmark our tokenizer. It should be long enough to provide meaningful results. Let's add some more words to make it even longer and more representative of a real document that sift might index during a search operation.";

    c.bench_function("tokenize", |b| b.iter(|| tokenize(black_box(text))));
}

fn bench_dot_product(c: &mut Criterion) {
    let a = vec![0.1f32; 384];
    let b = vec![0.2f32; 384];

    c.bench_function("dot_product_384", |bencher| {
        bencher.iter(|| dot_product(black_box(&a), black_box(&b)))
    });
}

criterion_group!(benches, bench_tokenize, bench_dot_product);
criterion_main!(benches);
