{
  description = "A simple and secure pet monitor for Linux.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
  };
  outputs = inputs@{ self, nixpkgs, utils, ... }: utils.lib.mkFlake {
    inherit self inputs;
    # supportedSystems = [ "x86_64-linux" "arm-linux" ];
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
            cargo-flamegraph
            libclang
            pkg-config
            x264
            nodePackages.pnpm
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
          # let
          #   pma-client = stdenvNoCC.mkDerivation {
          #     pname = "pet-monitor-app-client";
          #     version = "0.1.0";
          #     src = ./client;

          #     nativeBuildInputs = [ nodePackages.pnpm ];
          #     buildPhase = ''
          #       pnpm install
          #       pnpm build
          #     '';
          #     installPhase = ''
          #       mkdir -p $out/share
          #       cp -r ./build $out/share
          #     '';
          #   };
          # in
          rustPlatform.buildRustPackage {
            pname = "pet-monitor-app";
            version = "0.1.0";
            src = ./.;
            cargoLock = { lockFile = ./Cargo.lock; };

            nativeBuildInputs = [
              libclang
              pkg-config
              x264
              openssl
              # pma-client
            ];
            buildInputs = [ pkg-config x264 ];

            preBuild = ''
              # cp -r $\{pma-client}/share/build ./pet-monitor-app
            '';

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS = [
              "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include"
              "-I${pkgs.glibc.dev}/include"
            ];
            PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";
          };
      };
  };
}
