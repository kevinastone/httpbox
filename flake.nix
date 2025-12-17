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
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
      fenix,
      treefmt-nix,
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
        treefmtStack = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs.rustfmt = {
            enable = true;
            package = toolchain;
            edition = "2024";
          };
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
              config.Labels = {
                "org.opencontainers.image.title" = "httpbox";
                "org.opencontainers.image.source" = "https://github.com/kevinatone/httpbox";
                "org.opencontainers.image.description" = ''
                  httpbox is an HTTP test tool that provides a number of endpoints for testing a
                  variety of HTTP features similar to [httpbin](http://httpbin.org).
                '';
              };
            };
        };

        apps.skopeo = {
          type = "app";
          meta = pkgs.skopeo.meta;
          program = "${pkgs.skopeo}/bin/skopeo";
        };

        formatter = treefmtStack.config.build.wrapper;
        devShells.default =
          with pkgs;
          mkShell {
            nativeBuildInputs = [ toolchain ];
            packages = [
              skopeo
              cargo-outdated
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
