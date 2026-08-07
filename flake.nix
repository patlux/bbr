{
  description = "BBR browser development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "clippy"
              "rust-src"
              "rustfmt"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages =
              with pkgs;
              [
                rustToolchain
                rust-analyzer
                cargo-nextest
                cargo-watch
                clang
                cmake
                coreutils
                git
                just
                llvmPackages.libclang
                ninja
                pkg-config
              ]
              ++ lib.optionals stdenv.isDarwin [
                libiconv
              ];

            env = {
              LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
              RUST_BACKTRACE = "1";
            };

            shellHook = ''
              # Real Apple xcrun must win over the closure's xcbuild fake xcrun:
              # Metal shader compilation (gpui build scripts) needs the system
              # toolchain stub that honors DEVELOPER_DIR.
              export PATH="$PWD/scripts/xcode-shims:$PWD/target/debug:$PWD/target/release:$PATH"

              if [ "$(uname -s)" = "Darwin" ] && [ -d /Applications/Xcode.app/Contents/Developer ]; then
                export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
              fi

              echo "BBR dev shell: $(rustc --version)"
            '';
          };
        }
      );

      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt);
    };
}
