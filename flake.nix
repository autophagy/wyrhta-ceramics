{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk/master";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs { inherit system; };
          naersk-lib = pkgs.callPackage naersk { };
        in
        rec {
          packages = {
            wyrhta = naersk-lib.buildPackage {
              root = ./api;
              doCheck = true;
            };
            default = packages.wyrhta;
          };

          devShells = {
            api = with pkgs; mkShell {
              buildInputs = [ cargo rustc rustfmt rustPackages.clippy sqlite-interactive ];
              RUST_SRC_PATH = rustPlatform.rustLibSrc;
            };

            frontend = with pkgs; mkShell {
              buildInputs = [ nodejs ];
            };

            server = with pkgs; mkShell {
              buildInputs = [ caddy ];
            };

          };

          formatter = pkgs.nixpkgs-fmt;
        });
}
