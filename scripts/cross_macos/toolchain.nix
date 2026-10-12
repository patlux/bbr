# Task-scoped Linux build closure, never a system/profile activation.
# Reuse the qualified GPUI-Mobile Linux LLVM/Rust input revisions.
let
  nixpkgs = builtins.getFlake "github:NixOS/nixpkgs/89570f24e97e614aa34aa9ab1c927b6578a43775";
  rustOverlay = builtins.getFlake "github:oxalica/rust-overlay/ab777f900c3de943e117eb4814ddff6f645b28ad";
  pkgs = import nixpkgs.outPath {
    system = "x86_64-linux";
    overlays = [ (import rustOverlay.outPath) ];
  };
  rust = pkgs.rust-bin.stable."1.98.1".minimal.override {
    extensions = [ "rustfmt" "clippy" ];
    targets = [ "aarch64-apple-darwin" ];
  };
in pkgs.symlinkJoin {
  name = "bbr-linux-macos-tools";
  paths = [
    rust
    pkgs.llvmPackages.clang-unwrapped
    pkgs.llvmPackages.lld
    pkgs.llvmPackages.llvm
    pkgs.llvmPackages.libclang.lib
    pkgs.rcodesign
  ];
}
