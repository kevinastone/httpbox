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
      inputs.nixpkgs.follows = "nixpkgs";
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
        lib = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
      in
      {
        packages = rec {
          httpbox = lib.buildPackage ./.;
          bin = httpbox;
          default = httpbox;

          check = lib.buildPackage {
            src = ./.;
            mode = "check";
            release = false;
          };
          clippy = lib.buildPackage {
            src = ./.;
            mode = "clippy";
            release = false;
          };
          test = lib.buildPackage {
            src = ./.;
            mode = "test";
            release = false;
          };

          image =
            with pkgs;
            dockerTools.buildImage {
              name = "httpbox";
              config.Env = [ "PORT=80" ];
              config.Entrypoint = [ "${httpbox}/bin/httpbox" ];
            };
        };

        apps.skopeo = {
          type = "app";
          program = "${pkgs.skopeo}/bin/skopeo";
        };

        formatter = lib.buildPackage {
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
            packages = [
              skopeo
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            # RUSTC_VERSION = overrides.toolchain.channel;
          };
      }
    );
}
