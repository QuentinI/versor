{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.hello
    pkgs.gcc
    pkgs.glibc
    pkgs.rustc
    pkgs.cargo
    pkgs.pkg-config
    pkgs.openssl.dev


    # keep this line if you use bash
    pkgs.bashInteractive
  ];
}
