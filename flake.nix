{
  description = "sift - Standalone hybrid search (BM25 + Vector) for lightning-fast document retrieval";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    keel = {
      url = "git+ssh://git@github.com/rupurt/keel.git";
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
        };
        isLinux = pkgs.stdenv.isLinux;
        keelPkg = keel.packages.${system}.keel;

        sharedInputs = [
          rust
          pkgs.cmake
          pkgs.ninja
          pkgs.gnumake
          pkgs.gcc
          pkgs.clang
          pkgs.pkg-config
          pkgs.just
          pkgs.cargo-nextest
          pkgs.zlib
          pkgs.openssl
          pkgs.lz4
          pkgs.zstd
          pkgs.snappy
          pkgs.bzip2
          pkgs.gflags
          pkgs.glog
          pkgs.protobuf
          pkgs.re2
          pkgs.python3
          pkgs.perl
          keelPkg
        ];

        linuxInputs = pkgs.lib.optionals isLinux [
          pkgs.mold
          pkgs.stdenv.cc.cc.lib
        ];
      in {
        packages = {
          keel = keelPkg;
          default = keelPkg;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = sharedInputs ++ linuxInputs;

          shellHook = ''
            export CARGO_TARGET_DIR="$HOME/.cache/cargo-target/sift"
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
          '' + pkgs.lib.optionalString isLinux ''
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
          '';
        };
      });
}
