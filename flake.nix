{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk/master";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        packages = rec {
          httpbox = naersk-lib.buildPackage ./.;
          default = httpbox;
        };
        checks = {
          check = naersk-lib.buildPackage {
            src = ./.;
            mode = "check";
          };
          clippy = naersk-lib.buildPackage {
            src = ./.;
            mode = "clippy";
          };
          test = naersk-lib.buildPackage {
            src = ./.;
            mode = "test";
          };
        };
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
