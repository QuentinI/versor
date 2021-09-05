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
        devShell = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            gcc
            cargo
            rustc
            rustfmt
            llvm_10
            pkg-config
            openssl.dev
          ];
          LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "" buildInputs;
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
