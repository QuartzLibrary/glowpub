{
  description = "Dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (system: {
      devShells.default = nixpkgs.legacyPackages.${system}.mkShell (
        import ./shell.nix {
          pkgs = nixpkgs.legacyPackages.${system};
          rust-toolchain = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
        }
      );
      formatter = nixpkgs.legacyPackages.${system}.nixfmt-rfc-style;
    });
}
