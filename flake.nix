{
  description = "A simple and secure pet monitor for Linux.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, utils, fenix, ... }:
    utils.lib.eachSystem (with utils.lib.system; [ x86_64-linux ]) (system:
      let
        # system = utils.lib.system.x86_64-linux;
        pkgs = nixpkgs.legacyPackages.${system};
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
      in
      {
        overlays.default = _self: _super: fenix.overlays.default;

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

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "pet-monitor-app";
          version = "0.3.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            libclang
            pkg-config
            x264
          ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = [
            "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include"
            "-I${pkgs.glibc.dev}/include"
          ];
        };
      }
    );
}
