# Optimize Artifact Portability - SDD

> Implement static executable support in the release pipeline.

## Architecture Overview

Static executables on Linux are achieved by compiling against `musl` instead of `glibc`. This eliminates dependencies on system libraries, making the binary more portable across different Linux distributions.

## Components

### `Cargo.toml`
The `targets` list will be updated to include `x86_64-unknown-linux-musl`.

### `.github/workflows/release.yml`
The build matrix will be expanded to include the `musl` target, and the runner will be configured to install the `musl` tools.

## Data Flow

1. Trigger on tag.
2. Build matrix includes `x86_64-unknown-linux-musl`.
3. Artifacts uploaded to GitHub Release.
