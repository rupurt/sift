{
  description = "sift - Standalone hybrid search (BM25 + Vector) for lightning-fast document retrieval";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    keel = {
      url = "git+ssh://git@github.com/spoke-sh/keel.git";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    keel,
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl" ];
        };
        isLinux = pkgs.stdenv.isLinux;
        keelPkg = keel.packages.${system}.keel;

        sharedInputs = [
          rust
          pkgs.just
          pkgs.cargo-nextest
          pkgs.bzip2
          pkgs.xz
          pkgs.zlib
          keelPkg
        ];

        linuxInputs = pkgs.lib.optionals isLinux [
          pkgs.mold
        ];

        cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
        version = cargoToml.package.version;

        siftPkg = pkgs.rustPlatform.buildRustPackage {
          pname = "sift";
          inherit version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "candle-core-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
            };
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.bzip2 pkgs.xz pkgs.zlib ];
          doCheck = false;
        };

        siftStatic = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
          pname = "sift-static";
          inherit version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "candle-core-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
            };
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.bzip2 pkgs.xz pkgs.zlib ];
          doCheck = false;
        };
      in {
        packages = {
          sift = siftPkg;
          sift-static = siftStatic;
          keel = keelPkg;
          default = siftPkg;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = sharedInputs ++ linuxInputs;

          shellHook = ''
            export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/sift"
          '' + pkgs.lib.optionalString isLinux ''
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
          '';
        };
      });
}
