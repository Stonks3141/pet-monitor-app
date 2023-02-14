{
  description = "A simple and secure pet monitor for Linux.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, utils, fenix, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell
          {
            name = "pet-monitor-app";
            packages = with pkgs; [
              cargo
              rustc
              clippy
              rustfmt
              rust-analyzer
              just
              cargo-flamegraph
              oha
              libclang
              pkg-config
              x264
              nixpkgs-fmt
              nil
            ];
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS = [
              "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include"
              "-I${pkgs.glibc.dev}/include"
            ];
          };

        packages.default = pkgs.rustPlatform.buildRustPackage
          {
            pname = "pet-monitor-app";
            version = "0.3.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = with pkgs; [
              libclang
              pkg-config
            ];
            buildInputs = [ pkgs.x264 ];
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS = [
              "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include"
              "-I${pkgs.glibc.dev}/include"
            ];
          };
      }
    );
}
