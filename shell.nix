{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = [ pkgs.rustup pkgs.zig pkgs.llvmPackages_17.lld pkgs.pkg-config ];
}
