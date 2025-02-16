{
  description = "The main monorepo of the VALESC digital artisans group";

  inputs = {
    # Use `shallow=1` to avoid insane slow download times
    # TRACK: https://github.com/NixOS/nix/issues/10683
    nixpkgs.url = "github:NixOS/nixpkgs?shallow=1&ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    allSystems = [
      "x86_64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];

    # This functions defines the package for all the targets defined at the list `allSystems`.
    forAllSystems = callback:
      nixpkgs.lib.genAttrs allSystems
      (system: callback {pkgs = import nixpkgs {inherit system;};});
  in {
    formatter = forAllSystems ({pkgs}: pkgs.alejandra);

    devShells = forAllSystems ({pkgs}: {
      default = pkgs.mkShell {
        name = "VALESC";
        nativeBuildInputs = with pkgs; [
          addlicense
          nushell
          ripgrep
        ];
      };
    });
  };
}
