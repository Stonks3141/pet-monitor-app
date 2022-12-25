{
  description = "A simple and secure pet monitor for Linux.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    utils.url = "github:gytis-ivaskevicius/flake-utils-plus";
  };
  outputs = inputs@{ self, nixpkgs, utils, ... }: utils.lib.mkFlake {
    inherit self inputs;
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
          ];
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = [
            "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include"
            "-I${pkgs.glibc.dev}/include"
          ];
        };
      };
  };
}
