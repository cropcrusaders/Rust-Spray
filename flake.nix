{
  description = "Rust-Spray build environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = { self, nixpkgs }:
    let pkgs = import nixpkgs { system = "x86_64-linux"; };
    in {
      devShells.default = pkgs.mkShell {
        packages = [ pkgs.rustup pkgs.zig pkgs.llvmPackages_17.lld pkgs.pkg-config ];
      };
    };
}
