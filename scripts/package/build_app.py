"""Deterministic macOS .app bundling for the BBR release binary.

Consumes the already-built `bbr` executable generically (no app-code or
Cargo changes) and emits a standard bundle:

    <output>/BBR.app/
      Contents/Info.plist              token-filled from packaging template
      Contents/PkgInfo                 APPL????
      Contents/MacOS/bbr               the release binary, mode 0755
      Contents/Resources/AppIcon.icns  generated placeholder (gen_icon)

Every file written here gets one fixed mtime (SOURCE_DATE_EPOCH or a stable
default) and files are written in a fixed order, so identical inputs produce
byte-identical bundles. Unless --skip-codesign is given, the bundle is
ad-hoc signed with `codesign --force --sign - --timestamp=none` and verified
with `codesign --verify --strict`; after signing, every file and directory
in the bundle is pinned back to the fixed mtime because codesign touches the
executable and bundle directories with wall-clock timestamps.

This lane deliberately does not integrate with the later GPUI anchor or any
app code; it packages whatever Mach-O binary is handed to it.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import struct
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

# Make the sibling `package` package importable when run as a script.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from package.gen_icon import write_icns  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
EXECUTABLE = "bbr"
BUNDLE_NAME = "BBR.app"
DEFAULT_BUNDLE_ID = "dev.patwoz.bbr"
DEFAULT_MIN_SYSTEM_VERSION = "13.0"
DEFAULT_COPYRIGHT = "Copyright \u00a9 2026 BBR. All rights reserved."
TEMPLATE = REPO_ROOT / "packaging" / "Info.plist.template"
PKGINFO = REPO_ROOT / "packaging" / "PkgInfo"
DEFAULT_MTIME = int(os.environ.get("SOURCE_DATE_EPOCH", "1577836800"))  # 2020-01-01T00:00:00Z

# Magic bytes for Mach-O thin (64/32-bit, either endian) and universal/fat.
_MACHO_MAGICS = {b"\xcf\xfa\xed\xfe", b"\xce\xfa\xed\xfe",
                 b"\xfe\xed\xfa\xcf", b"\xfe\xed\xfa\xce",
                 b"\xca\xfe\xba\xbe", b"\xbe\xba\xfe\xca"}


@dataclass
class BuildResult:
    app: Path
    bundle_id: str
    version: str
    executable: str
    signed: bool
    files: list[Path] = field(default_factory=list)


def workspace_version() -> str:
    """Read the workspace package version from the root Cargo.toml."""
    text = (REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
    return match.group(1) if match else "0.1.0"


def render_plist(bundle_id: str, version: str) -> str:
    text = TEMPLATE.read_text(encoding="utf-8")
    tokens = {
        "__EXECUTABLE__": EXECUTABLE,
        "__BUNDLE_ID__": bundle_id,
        "__SHORT_VERSION__": version,
        "__BUILD_VERSION__": version,
        "__MIN_SYSTEM_VERSION__": DEFAULT_MIN_SYSTEM_VERSION,
        "__COPYRIGHT__": DEFAULT_COPYRIGHT,
    }
    for token, value in tokens.items():
        if token not in text:
            raise ValueError(f"template missing token {token}")
        text = text.replace(token, value)
    return text


def _set_mtime(path: Path, mtime: int) -> None:
    os.utime(path, (mtime, mtime))


def _require_macho(binary: Path) -> None:
    if sys.platform != "darwin":
        return  # the Mach-O check only makes sense on macOS
    with binary.open("rb") as handle:
        magic = handle.read(4)
    if magic not in _MACHO_MAGICS:
        raise ValueError(f"{binary} is not a Mach-O binary (magic {magic!r})")


def _run(command: list[str], label: str) -> None:
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode != 0:
        detail = (result.stderr or result.stdout or "").strip()
        raise RuntimeError(f"{label} failed (exit {result.returncode}): {detail}")


def _codesign(app: Path, bundle_id: str, mtime: int) -> None:
    _run(["codesign", "--force", "--sign", "-", "--timestamp=none",
          "--identifier", bundle_id, str(app)], "codesign")
    _run(["codesign", "--verify", "--strict", str(app)], "codesign --verify")
    # codesign rewrites the executable's signature envelope and touches the
    # bundle directories with wall-clock mtimes; pin every file and directory
    # (deepest first) back to the fixed epoch so the signed bundle keeps the
    # same fixed-mtime invariant as the unsigned one.
    for path in sorted(app.rglob("*"), reverse=True):
        _set_mtime(path, mtime)
    _set_mtime(app, mtime)


def build_app(binary: Path, output: Path, *, bundle_id: str = DEFAULT_BUNDLE_ID,
              version: str | None = None, skip_codesign: bool = False,
              mtime: int = DEFAULT_MTIME) -> BuildResult:
    """Assemble, sign (unless skipped), and verify the .app bundle."""
    binary = Path(binary)
    output = Path(output)
    if not binary.is_file():
        raise FileNotFoundError(f"binary not found: {binary}")
    if not os.access(binary, os.X_OK):
        raise ValueError(f"binary is not executable: {binary}")
    if not skip_codesign:
        _require_macho(binary)
    version = version or workspace_version()

    # Wipe any previous bundle so stale files cannot leak into the output.
    if output.exists():
        shutil.rmtree(output)
    contents = output / "Contents"
    macos_dir = contents / "MacOS"
    resources = contents / "Resources"
    for directory in (output, contents, macos_dir, resources):
        directory.mkdir(parents=True, exist_ok=True)

    executable = macos_dir / EXECUTABLE
    shutil.copy2(binary, executable)
    os.chmod(executable, 0o755)
    _set_mtime(executable, mtime)

    plist = contents / "Info.plist"
    plist.write_text(render_plist(bundle_id, version), encoding="utf-8")
    os.chmod(plist, 0o644)
    _set_mtime(plist, mtime)

    pkginfo = contents / "PkgInfo"
    pkginfo.write_bytes(PKGINFO.read_bytes())
    os.chmod(pkginfo, 0o644)
    _set_mtime(pkginfo, mtime)

    icns = resources / "AppIcon.icns"
    write_icns(icns)
    os.chmod(icns, 0o644)
    _set_mtime(icns, mtime)

    for directory in (output, contents, macos_dir, resources):
        _set_mtime(directory, mtime)

    signed = False
    if not skip_codesign:
        _codesign(output, bundle_id, mtime)
        signed = True

    return BuildResult(
        app=output, bundle_id=bundle_id, version=version,
        executable=EXECUTABLE, signed=signed,
        files=[executable, plist, pkginfo, icns],
    )


def _human_size(path: Path) -> str:
    size = path.stat().st_size
    for unit in ("B", "KiB", "MiB"):
        if size < 1024 or unit == "MiB":
            return f"{size} {unit}"
        size //= 1024
    return f"{size} MiB"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Package the BBR release binary into a signed .app")
    parser.add_argument("--binary", required=True, type=Path, help="built bbr Mach-O executable")
    parser.add_argument("--output", type=Path, default=Path("target/bbr-app") / BUNDLE_NAME,
                        help="output .app bundle path (default target/bbr-app/BBR.app)")
    parser.add_argument("--bundle-id", default=DEFAULT_BUNDLE_ID,
                        help=f"CFBundleIdentifier (default {DEFAULT_BUNDLE_ID})")
    parser.add_argument("--version", default=None,
                        help="bundle version; default reads the workspace Cargo.toml")
    parser.add_argument("--skip-codesign", action="store_true",
                        help="skip ad-hoc codesigning and verification (tests, fake binaries)")
    parser.add_argument("--mtime", type=int, default=DEFAULT_MTIME,
                        help="fixed mtime epoch for bundle files (default SOURCE_DATE_EPOCH or 2020-01-01)")
    args = parser.parse_args(argv[1:])

    result = build_app(args.binary, args.output, bundle_id=args.bundle_id,
                       version=args.version, skip_codesign=args.skip_codesign,
                       mtime=args.mtime)
    signature = "ad-hoc, verified" if result.signed else "skipped"
    print(f"Packaged {result.app} (bundle id {result.bundle_id}, version {result.version}, "
          f"signature: {signature})")
    for path in result.files:
        print(f"  {path.relative_to(result.app)}  ({_human_size(path)})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
