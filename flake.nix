{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [
            cargo rustc rustfmt pre-commit rustPackages.clippy alsa-lib wayland libxkbcommon vulkan-loader
          ];
          nativeBuildInputs = [ pkg-config cmake ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;

          shellHook = ''
            export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${wayland}/lib:${libxkbcommon}/lib:${vulkan-loader}/lib
          '';
        };
      }
    );
}
