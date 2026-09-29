"""Incremental development `.app` wrapper, used as the `just run` Cargo runner.

A bare Mach-O has no bundle identity. macOS then shows the generic
executable icon, names the application menu `bbr`, and — because GPUI only
switches the process to the regular activation policy inside
`didFinishLaunching` — does not activate it when it is launched from a
terminal: the terminal keeps the menu bar while the BBR window is on screen.
The same binary run from inside a bundle launches as a regular app with the
BBR name, the Phosphor-b Dock icon and the BBR menu bar.

`just run` sets this script as Cargo's runner, so `mbx run -p bbr` builds as
before and Cargo hands the fresh binary (plus any program arguments) here;
`--exec` then replaces this process with the bundle executable, keeping the
terminal's foreground process group, logs and Ctrl-C.

The release lane (`build_app.py`) stays the deterministic packaging
authority. This wrapper reuses its Info.plist template, PkgInfo and icon
renderer but is incremental so a development launch stays instant:

- the binary is cloned (APFS `clonefile`, falling back to a copy);
- the icon is rendered once and re-rendered only when the renderer changes;
- LaunchServices is refreshed only when the plist or icon changed.

The development bundle uses its own identifier so it never shares
LaunchServices, TCC or WebKit state with a packaged release. WebKit keys a
bundle-less process's data by executable name (`~/Library/WebKit/bbr`); the
first bundled launch clones that directory to the new identifier once, so
existing logins and site data survive the switch. The source is never
modified.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import subprocess
import sys
from collections.abc import Callable
from pathlib import Path

# Make the sibling `package` package importable when run as a script.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from package import gen_icon  # noqa: E402
from package.build_app import EXECUTABLE, PKGINFO, render_plist, workspace_version  # noqa: E402

DEV_BUNDLE_ID = "dev.patwoz.bbr.dev"
BUNDLE_NAME = "BBR.app"
# WebKit's data directory name for the bundle-less `bbr` process.
UNBUNDLED_WEBKIT_NAME = EXECUTABLE
LSREGISTER = Path(
    "/System/Library/Frameworks/CoreServices.framework/Frameworks/"
    "LaunchServices.framework/Support/lsregister"
)


def _write_if_changed(path: Path, data: bytes, mode: int = 0o644) -> bool:
    if path.is_file() and path.read_bytes() == data:
        return False
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_bytes(data)
    os.chmod(tmp, mode)
    os.replace(tmp, path)
    return True


def _clone(source: Path, destination: Path, *, directory: bool = False) -> None:
    """Copy-on-write clone on APFS (`cp -c`), plain copy elsewhere."""
    flags = ["-c", "-R"] if directory else ["-c"]
    result = subprocess.run(["cp", *flags, str(source), str(destination)],
                            capture_output=True)
    if result.returncode == 0:
        return
    if destination.exists() or destination.is_symlink():
        if directory:
            shutil.rmtree(destination)
        else:
            destination.unlink()
    if directory:
        shutil.copytree(source, destination, symlinks=True)
    else:
        shutil.copy2(source, destination)


def install_binary(binary: Path, executable: Path) -> None:
    """Replace the bundle executable atomically with a clone of `binary`."""
    tmp = executable.with_name(executable.name + ".tmp")
    if tmp.exists():
        tmp.unlink()
    _clone(binary, tmp)
    os.chmod(tmp, 0o755)
    os.replace(tmp, executable)


def icon_fingerprint() -> str:
    """Changes whenever the icon renderer (and therefore the icon) changes."""
    return hashlib.sha256(Path(gen_icon.__file__).read_bytes()).hexdigest()


def ensure_icon(icns: Path, stamp: Path) -> bool:
    """Render `AppIcon.icns` unless the stamp proves it is current."""
    fingerprint = icon_fingerprint()
    if icns.is_file() and stamp.is_file() and stamp.read_text().strip() == fingerprint:
        return False
    _write_if_changed(icns, gen_icon.build_icns())
    stamp.write_text(fingerprint + "\n")
    return True


def bbr_is_running() -> bool:
    """True when any process named `bbr` runs (it may be writing WebKit data)."""
    result = subprocess.run(["pgrep", "-x", EXECUTABLE], capture_output=True)
    return result.returncode == 0


class SeedBlocked(RuntimeError):
    """The site data cannot be copied safely right now."""


def seed_webkit_data(bundle_id: str, *, webkit_root: Path | None = None,
                     is_running: Callable[[], bool] = bbr_is_running) -> str | None:
    """Clone the bundle-less process's WebKit data to `bundle_id` once.

    Returns a status line when data was copied, `None` when there is nothing
    to copy. Raises [`SeedBlocked`] while a `bbr` process runs: its SQLite
    stores may be mid-write, and launching anyway would create the target
    directory and forfeit the copy for good.
    """
    root = webkit_root or Path.home() / "Library" / "WebKit"
    source = root / UNBUNDLED_WEBKIT_NAME
    target = root / bundle_id
    if target.exists() or not source.is_dir():
        return None
    if is_running():
        raise SeedBlocked(
            f"a bbr process is still running and may be writing {source}; "
            f"quit it so its site data can move to {target}, then run again")
    staging = root / f".{bundle_id}.seeding"
    if staging.exists():
        shutil.rmtree(staging)
    _clone(source, staging, directory=True)
    os.replace(staging, target)
    return f"copied WebKit site data {source} -> {target}"


def refresh_launch_services(app: Path) -> None:
    """Make Finder/Dock pick up a changed plist or icon (best effort)."""
    os.utime(app)
    if LSREGISTER.is_file():
        subprocess.run([str(LSREGISTER), "-f", str(app)], capture_output=True)


def build_dev_app(binary: Path, output: Path, *, bundle_id: str = DEV_BUNDLE_ID,
                  version: str | None = None) -> bool:
    """Create or update the development bundle; True when its metadata changed."""
    binary = Path(binary)
    output = Path(output)
    if not binary.is_file():
        raise FileNotFoundError(f"binary not found: {binary}")
    if not os.access(binary, os.X_OK):
        raise ValueError(f"binary is not executable: {binary}")
    contents = output / "Contents"
    macos_dir = contents / "MacOS"
    resources = contents / "Resources"
    for directory in (macos_dir, resources):
        directory.mkdir(parents=True, exist_ok=True)

    install_binary(binary, macos_dir / EXECUTABLE)
    plist = render_plist(bundle_id, version or workspace_version()).encode("utf-8")
    changed = _write_if_changed(contents / "Info.plist", plist)
    changed |= _write_if_changed(contents / "PkgInfo", PKGINFO.read_bytes())
    stamp = output.parent / f".{output.stem}.icon-sha256"
    changed |= ensure_icon(resources / "AppIcon.icns", stamp)
    return changed


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Wrap a built bbr binary in a development BBR.app (Cargo runner)")
    parser.add_argument("--output", type=Path, default=None,
                        help=f"bundle path (default: {BUNDLE_NAME} next to the binary)")
    parser.add_argument("--bundle-id", default=DEV_BUNDLE_ID,
                        help=f"CFBundleIdentifier (default {DEV_BUNDLE_ID})")
    parser.add_argument("--no-seed", action="store_true",
                        help="do not copy the bundle-less WebKit site data")
    parser.add_argument("--exec", action="store_true",
                        help="replace this process with the bundle executable")
    parser.add_argument("binary", type=Path, help="built bbr executable")
    parser.add_argument("args", nargs=argparse.REMAINDER,
                        help="program arguments passed through with --exec")
    args = parser.parse_args(argv[1:])

    output = args.output or args.binary.resolve().parent / BUNDLE_NAME
    changed = build_dev_app(args.binary, output, bundle_id=args.bundle_id)
    if changed and sys.platform == "darwin":
        refresh_launch_services(output)
    if not args.no_seed:
        try:
            status = seed_webkit_data(args.bundle_id)
        except SeedBlocked as error:
            print(f"dev_app: {error}", file=sys.stderr)
            return 3
        if status:
            print(f"dev_app: {status}", file=sys.stderr)
    if args.exec:
        executable = str(output / "Contents" / "MacOS" / EXECUTABLE)
        sys.stdout.flush()
        sys.stderr.flush()
        os.execv(executable, [executable, *args.args])
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
