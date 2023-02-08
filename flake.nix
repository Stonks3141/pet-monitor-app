{
  description = "A simple and secure pet monitor for Linux.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = inputs@{ self, nixpkgs, utils, fenix, ... }: utils.lib.mkFlake {
    inherit self inputs;
    sharedOverlays = [ fenix.overlays.default ];
    supportedSystems = [ "x86_64-linux" ];
    outputsBuilder = channels:
      let
        pkgs = channels.nixpkgs;
        mkShell = { name, packages }: pkgs.mkShell {
          inherit name;
          packages = with pkgs; [
            libclang
            pkg-config
            x264
          ] ++ packages;
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = [
            "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include"
            "-I${pkgs.glibc.dev}/include"
          ];
        };
      in {
        devShells.default = mkShell {
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
            nixpkgs-fmt
            nil
          ];
        };

        devShells.udeps = mkShell {
          name = "pet-monitor-app-udeps";
          packages = with pkgs; [
            (fenix.packages.x86_64-linux.minimal.withComponents [
              "cargo"
              "rustc"
            ])
            cargo-udeps
          ];
        };

        devShells.publish = mkShell {
          name = "pet-monitor-app-publish";
          packages = with pkgs; [
            cargo
            rustc
            cargo-workspaces
            just
          ];
        };

        packages.default = with import nixpkgs { system = "x86_64-linux"; };
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
