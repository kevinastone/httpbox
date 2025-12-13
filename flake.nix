{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.fenix.follows = "fenix";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
      fenix,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        stable = fenix.packages.${system}.stable;
        toolchain = fenix.packages.${system}.combine [
          stable.cargo
          stable.rustc
          stable.rustfmt
          stable.clippy
        ];
        naersk-lib = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
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
        formatter = naersk-lib.buildPackage {
          src = ./.;
          mode = "fmt";
        };
        devShells.default =
          with pkgs;
          mkShell {
            nativeBuildInputs = [ toolchain ];
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            # RUSTC_VERSION = overrides.toolchain.channel;
          };
      }
    );
}
