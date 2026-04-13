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
        cargoBuildPackage = rustPlatform: pname: cudaEnabled:
          rustPlatform.buildRustPackage {
            inherit pname version;
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = cargoOutputHashes;
            };
            nativeBuildInputs = [ pkgs.pkg-config ] ++ pkgs.lib.optionals (isLinux && cudaEnabled) [ cudaToolkit ];
            buildInputs = [ pkgs.bzip2 pkgs.xz pkgs.zlib ] ++ pkgs.lib.optionals (isLinux && cudaEnabled) [ cudaToolkit ];
            cargoBuildFeatures = pkgs.lib.optionals (isLinux && cudaEnabled) [ "cuda" ];
            doCheck = false;

            CUDA_HOME = pkgs.lib.optionalString (isLinux && cudaEnabled) "${cudaToolkit}";
            CUDA_PATH = pkgs.lib.optionalString (isLinux && cudaEnabled) "${cudaToolkit}";
            CUDA_ROOT = pkgs.lib.optionalString (isLinux && cudaEnabled) "${cudaToolkit}";
            CUDA_TOOLKIT_ROOT_DIR = pkgs.lib.optionalString (isLinux && cudaEnabled) "${cudaToolkit}";
            CUDA_COMPUTE_CAP = pkgs.lib.optionalString (isLinux && cudaEnabled) "80";
            NVCC_PREPEND_FLAGS = pkgs.lib.optionalString (isLinux && cudaEnabled) "-I${cudaToolkit}/include";
          };
        baseShellHook = ''
          export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/sift"
          export SIFT_CACHE="$HOME/.cache/sift"
        '';
        moldShellHook = pkgs.lib.optionalString isLinux ''
          linux_link_args="-C link-arg=-fuse-ld=mold"
          export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS }$linux_link_args"
          export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="''${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS:+$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS }$linux_link_args"
        '';
        cudaShellHook = pkgs.lib.optionalString isLinux ''
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

          if [ -n "$cuda_driver_rpath" ]; then
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="$CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS -C link-arg=-Wl,-rpath,$cuda_driver_rpath"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS -C link-arg=-Wl,-rpath,$cuda_driver_rpath"
          fi
        '';

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
        ];
        cudaInputs = pkgs.lib.optionals isLinux [ cudaToolkit ];

        cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
        version = cargoToml.package.version;
        cargoOutputHashes = {
          "candle-core-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
          "candle-nn-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
          "candle-transformers-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
          "candle-kernels-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
          "candle-ug-0.9.2" = "sha256-Oa62yRA95P/MsGUG2u10a/jgcRtUdVFOQIoykqmv4Bs=";
          "metamorph-0.1.0" = "sha256-sGl4+khLHI2k4gX/jikg9ZcVDknQNKXYWHuV2uZtnCc=";
        };

        siftPkg = cargoBuildPackage pkgs.rustPlatform "sift" false;
        siftCudaPkg = cargoBuildPackage pkgs.rustPlatform "sift-cuda" true;
        siftStatic = cargoBuildPackage pkgs.pkgsStatic.rustPlatform "sift-static" false;
      in {
        packages = {
          sift = siftPkg;
          sift-static = siftStatic;
          keel = keelPkg;
          default = siftPkg;
        } // pkgs.lib.optionalAttrs isLinux {
          sift-cuda = siftCudaPkg;
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = sharedInputs ++ linuxInputs;
            shellHook = baseShellHook + moldShellHook;
          };
        } // pkgs.lib.optionalAttrs isLinux {
          cuda = pkgs.mkShell {
            buildInputs = sharedInputs ++ linuxInputs ++ cudaInputs;
            shellHook = baseShellHook + moldShellHook + cudaShellHook;
          };
        };
      });
}
