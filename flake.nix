{
  description = "A simple and secure pet monitor for Linux.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
  };
  outputs = inputs@{ self, nixpkgs, utils, ... }: utils.lib.mkFlake {
    inherit self inputs;
    supportedSystems = [ "x86_64-linux" "arm-linux" ];
    outputsBuilder = channels:
      let pkgs = channels.nixpkgs; in {
        devShell = pkgs.mkShell {
          name = "pet-monitor-app";
          packages = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
            rust-analyzer
            cargo-udeps
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
        defaultPackage = with import nixpkgs { system = "x86_64-linux"; };
          rustPlatform.buildRustPackage {
            pname = "pet-monitor-app";
            version = "0.3.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [
              libclang
              pkg-config
              x264
            ];
            buildInputs = [ pkg-config x264 ];

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS = [
              "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include"
              "-I${pkgs.glibc.dev}/include"
            ];
          };
      };
  };
}
