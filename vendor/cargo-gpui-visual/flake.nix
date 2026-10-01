{
  description = "Pinned portable GPUI Visual development tools";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" "x86_64-linux" "aarch64-linux" ];
      forAll = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      toolchainFor = system: (pkgsFor system).rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      packages = forAll (system: { rust-toolchain = toolchainFor system; });
      devShells = forAll (system:
        let pkgs = pkgsFor system;
        in {
          default = pkgs.mkShellNoCC {
            packages = [ (toolchainFor system) pkgs.git ];
          };
        });
    };
}
