{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs@{ nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in rec {
        devShell = pkgs.mkShell {
          buildInputs = [
            pkgs.gcc
            pkgs.glibc
            pkgs.rustc
            pkgs.cargo
            pkgs.llvm_10
            pkgs.pkg-config
            pkgs.openssl.dev
          ];
        };

        defaultPackage = pkgs.rustPlatform.buildRustPackage rec {
          pname = "versor";
          version = "master";

          src = ./.;

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];

          postInstall = ''
            ln -s $out/bin/${pname} $out/bin/${pname}-${version}
          '';

          cargoSha256 = "sha256-SgAgLjlhBZLqUKV6mKRYeCJyjBZ2VErpP86labqLW+E=";
        };
      });
}
