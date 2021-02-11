{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs@{ nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          devShell = pkgs.mkShell {
            buildInputs = [
              pkgs.gcc
              pkgs.glibc
              pkgs.rustc
              pkgs.cargo
              pkgs.pkg-config
              pkgs.openssl.dev
              pkgs.llvm_10
            ];
          };
        }
      );
}
