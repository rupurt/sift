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
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl" ];
        };
        isLinux = pkgs.stdenv.isLinux;
        cudaToolkit = pkgs.cudatoolkit;
        keelPkg = keel.packages.${system}.keel;

        sharedInputs = [
          rust
          pkgs.just
          pkgs.cargo-nextest
          pkgs.cargo-flamegraph
          pkgs.bzip2
          pkgs.xz
          pkgs.zlib
          keelPkg
        ];

        linuxInputs = pkgs.lib.optionals isLinux [
          pkgs.mold
          cudaToolkit
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
              "candle-nn-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-transformers-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-kernels-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-ug-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
            };
          };
          nativeBuildInputs = [ pkgs.pkg-config ] ++ pkgs.lib.optionals isLinux [ cudaToolkit ];
          buildInputs = [ pkgs.bzip2 pkgs.xz pkgs.zlib ] ++ pkgs.lib.optionals isLinux [ cudaToolkit ];
          doCheck = false;

          CUDA_HOME = pkgs.lib.optionalString isLinux "${cudaToolkit}";
          CUDA_PATH = pkgs.lib.optionalString isLinux "${cudaToolkit}";
          CUDA_ROOT = pkgs.lib.optionalString isLinux "${cudaToolkit}";
          CUDA_TOOLKIT_ROOT_DIR = pkgs.lib.optionalString isLinux "${cudaToolkit}";
          CUDA_COMPUTE_CAP = "80";
          NVCC_PREPEND_FLAGS = pkgs.lib.optionalString isLinux "-I${cudaToolkit}/include";
        };

        siftStatic = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
          pname = "sift-static";
          inherit version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "candle-core-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-nn-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-transformers-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-kernels-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
              "candle-ug-0.9.2" = "sha256-GeU7yc4vqN0hy3tJAq0LDhwnpO4XDeVVmxaBchKWkWg=";
            };
          };
          nativeBuildInputs = [ pkgs.pkg-config ] ++ pkgs.lib.optionals isLinux [ cudaToolkit ];
          buildInputs = [ pkgs.bzip2 pkgs.xz pkgs.zlib ] ++ pkgs.lib.optionals isLinux [ cudaToolkit ];
          doCheck = false;

          CUDA_HOME = pkgs.lib.optionalString isLinux "${cudaToolkit}";
          CUDA_PATH = pkgs.lib.optionalString isLinux "${cudaToolkit}";
          CUDA_ROOT = pkgs.lib.optionalString isLinux "${cudaToolkit}";
          CUDA_TOOLKIT_ROOT_DIR = pkgs.lib.optionalString isLinux "${cudaToolkit}";
          CUDA_COMPUTE_CAP = "80";
          NVCC_PREPEND_FLAGS = pkgs.lib.optionalString isLinux "-I${cudaToolkit}/include";
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
            export SIFT_CACHE="$HOME/.cache/sift"
          '' + pkgs.lib.optionalString isLinux ''
            export CUDA_HOME="${cudaToolkit}"
            export CUDA_PATH="${cudaToolkit}"
            export CUDA_ROOT="${cudaToolkit}"
            export CUDA_TOOLKIT_ROOT_DIR="${cudaToolkit}"
            export NVCC_PREPEND_FLAGS="-I${cudaToolkit}/include"
            cuda_driver_rpath=""
            for candidate in \
              /run/opengl-driver/lib \
              /usr/lib/x86_64-linux-gnu \
              /usr/lib/wsl/lib
            do
              if [ -f "$candidate/libcuda.so.1" ]; then
                cuda_driver_rpath="$candidate"
                break
              fi
            done

            linux_link_args="-C link-arg=-fuse-ld=mold"
            if [ -n "$cuda_driver_rpath" ]; then
              linux_link_args="$linux_link_args -C link-arg=-Wl,-rpath,$cuda_driver_rpath"
            fi

            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS }$linux_link_args"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS }$linux_link_args"
          '';
        };
      });
}
