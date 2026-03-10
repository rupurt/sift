# Add Homebrew Platform Support - SDD

> Implement Homebrew support in the cargo-dist pipeline.

## Architecture Overview

Homebrew support is handled as an additional "installer" in `cargo-dist`. It generates a Ruby formula that points to the released tarballs on GitHub.

## Components

### `Cargo.toml`
Update the `installers` list to include `homebrew`. Configure the tap repository if necessary.

## Data Flow

1. `cargo dist build` generates the formula.
2. `cargo dist host github` uploads the formula along with other artifacts.
