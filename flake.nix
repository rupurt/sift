{
  description = "sift - Standalone hybrid search (BM25 + Vector) for lightning-fast document retrieval";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs-cmake.url = "github:NixOS/nixpkgs/nixos-25.05";
    keel = {
      url = "github:rupurt/keel?rev=2b14d578c1771b247275ad5ba295e2cf4a0f1fec";
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
    nixpkgs-cmake,
    keel,
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        compatPkgs = import nixpkgs-cmake { inherit system; };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
        isLinux = pkgs.stdenv.isLinux;
        keelPkg = keel.packages.${system}.keel;

        sharedInputs = [
          rust
          compatPkgs.cmake
          pkgs.ninja
          pkgs.gnumake
          compatPkgs.stdenv.cc
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
          compatPkgs.stdenv.cc.cc.lib
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
            export CC="${compatPkgs.stdenv.cc}/bin/cc"
            export CXX="${compatPkgs.stdenv.cc}/bin/c++"
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
          '' + pkgs.lib.optionalString isLinux ''
            export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
          '';
        };
      });
}
