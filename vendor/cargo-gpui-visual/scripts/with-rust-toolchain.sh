#!/usr/bin/env bash
# Use the pinned host-only Nix Rust distribution; mbx owns build storage.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
toolchain="$(nix build --no-link --print-out-paths "${ROOT_DIR}#rust-toolchain")"
test -x "${toolchain}/bin/rustc"
test -x "${toolchain}/bin/cargo"
if [ "$(uname -s)" = Darwin ]; then
  # Nix dev-shell dependencies may prepend cc-wrapper even with mkShellNoCC.
  # Use selected Xcode for multi-target C/ObjC; keep target-specific NDK vars.
  native_tools="$(dirname "$(xcrun --find clang)")"
  test -x "${native_tools}/clang"
  test -x "${native_tools}/clang++"
  export CC="${native_tools}/clang"
  export CXX="${native_tools}/clang++"
  export PATH="${native_tools}:${PATH}"
fi
export PATH="${toolchain}/bin:${PATH}"
export RUSTC="${toolchain}/bin/rustc"
export RUSTDOC="${toolchain}/bin/rustdoc"
exec "$@"
