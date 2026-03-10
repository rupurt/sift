# sift-embed

`sift-embed` is the canonical runnable embedding reference for consuming
`sift` as a library from another Rust crate.

This example stays on the supported public surface:

- It depends on `sift` through the crate root.
- It builds a standalone `sift-embed` executable.
- It renders `SearchResponse` locally instead of using `sift::internal`.

## Build And Run

From the repository root:

```bash
just embed-build
just embed-search tests/fixtures/rich-docs "architecture decision"
```

You can also run the example directly through Cargo:

```bash
cargo run --manifest-path examples/sift-embed/Cargo.toml -- search "hybrid search"
cargo run --manifest-path examples/sift-embed/Cargo.toml -- search tests/fixtures/rich-docs "architecture decision"
```

## Command Shape

```bash
sift-embed search [OPTIONS] [PATH] <QUERY>
```

If `PATH` is omitted, `sift-embed search "<term>"` searches the current
directory.
