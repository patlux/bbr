"""Guest-only rendering observations; no build-host environment fallback.

Invoked by the future installed-payload guest runner, not the host CLI. Callers
must validate the receipt and exact before/after equality in the Rust consumer.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import plistlib
import subprocess
import sys
import time


# Stock macOS alias observed in the disposable golden clone. No generic alias
# following: the exact destination must also be directly inventoried below.
FONT_ALIASES = {"/Library/Fonts/Arial Unicode.ttf":
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf"}


class ObservationError(RuntimeError):
    pass


def console(plist: bytes) -> dict:
    roots = plistlib.loads(plist)
    # macOS 27's name-selected XML output is a direct root dictionary. Older
    # ioreg versions wrap that same root in a singleton array; neither shape
    # relaxes the requirement for one actual unlocked admin console.
    if isinstance(roots, dict):
        if roots.get("IORegistryEntryName") != "Root":
            raise ObservationError("named IORegistry root required")
        root = roots
    elif isinstance(roots, list) and len(roots) == 1 and isinstance(roots[0], dict):
        root = roots[0]
    else:
        raise ObservationError("one IORegistry root required")
    users = root.get("IOConsoleUsers", [])
    if not isinstance(users, list) or any(not isinstance(user, dict) for user in users):
        raise ObservationError("invalid guest console records")
    active = [u for u in users if u.get("kCGSSessionOnConsoleKey") is True]
    if len(active) != 1 or active[0].get("kCGSSessionUserNameKey") != "admin":
        raise ObservationError("one admin guest console required")
    if active[0].get("CGSSessionScreenIsLocked", False) is not False:
        raise ObservationError("guest screen locked")
    return {"console_user": "admin", "on_console": True, "screen_locked": False}


def fonts(roots: tuple[Path, ...], user_root: Path, deadline: float) -> dict[str, str]:
    if user_root.is_symlink():
        raise ObservationError("symlink guest user font root")
    if user_root.exists():
        for entry in user_root.iterdir():
            # Actual golden-image metadata, not fonts. Do not follow this hidden
            # version-marker link or allow a populated/aliased font directory.
            if (entry.name == "HomeManager" and not entry.is_symlink()
                    and entry.is_dir() and not any(entry.iterdir())):
                continue
            if (entry.name == ".home-manager-fonts-version" and entry.is_symlink()
                    and os.readlink(entry) == "/nix/store/pz885hwbsg4vjxda3d5k33917x1irg5z-home-manager-files/Library/Fonts/.home-manager-fonts-version"):
                continue
            raise ObservationError("user-installed guest fonts are not part of the canonical environment: " + entry.name)
    result = {}
    unsupported = []
    for root in roots:
        if not root.is_dir() or root.is_symlink():
            raise ObservationError("missing/symlink guest font root")
        for directory, subdirs, files in os.walk(root, followlinks=False):
            for name in subdirs:
                path = Path(directory) / name
                if path.is_symlink():
                    unsupported.append({"path": str(path), "target": os.readlink(path)})
                    if len(unsupported) > 64:
                        raise ObservationError("too many unsupported font paths")
            for name in files:
                path = Path(directory) / name
                if time.monotonic() >= deadline:
                    raise ObservationError("guest font inventory deadline")
                actual = path
                if path.is_symlink():
                    expected = FONT_ALIASES.get(str(path))
                    if (expected is not None and os.readlink(path) == expected
                            and Path(expected).resolve() == Path(expected) and Path(expected).is_file()):
                        actual = Path(expected)
                    else:
                        unsupported.append({"path": str(path), "target": os.readlink(path)})
                        if len(unsupported) > 64:
                            raise ObservationError("too many unsupported font paths")
                        continue
                elif not path.is_file():
                    raise ObservationError("nonregular guest font entry: " + str(path))
                # Hash regular bytes, including the exact system alias target.
                # User aliases and arbitrary destination chains remain forbidden.
                if actual.stat().st_size > 256 * 1024 * 1024:
                    raise ObservationError("guest font file exceeds bound")
                digest = hashlib.sha256()
                with actual.open("rb") as handle:
                    while block := handle.read(65536):
                        if time.monotonic() >= deadline:
                            raise ObservationError("guest font hashing deadline")
                        digest.update(block)
                result[str(path)] = digest.hexdigest()
    for alias, target in FONT_ALIASES.items():
        if alias in result and result.get(target) != result[alias]:
            raise ObservationError("font alias target missing or changed during inventory")
    if unsupported:
        raise ObservationError("unsupported guest font entries: " + json.dumps(unsupported, sort_keys=True))
    if not result:
        raise ObservationError("empty guest font inventory")
    return dict(sorted(result.items()))


# Executed only by the pinned guest interpreter in a deadline-bound child. No
# AppKit application/window or renderer is created. system_profiler on Tart's
# macOS 27 image reports an empty GPU array despite the active virtual display.
NATIVE_DISPLAY_PROBE = r'''
import ctypes as c, json
cg = c.CDLL('/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics')
metal = c.CDLL('/System/Library/Frameworks/Metal.framework/Metal')
objc = c.CDLL('/usr/lib/libobjc.A.dylib')
def bind(lib, name, result, args):
    fn = getattr(lib, name); fn.restype = result; fn.argtypes = args; return fn
active = bind(cg, 'CGGetActiveDisplayList', c.c_int32, [c.c_uint32, c.POINTER(c.c_uint32), c.POINTER(c.c_uint32)])
copy_mode = bind(cg, 'CGDisplayCopyDisplayMode', c.c_void_p, [c.c_uint32])
release_mode = bind(cg, 'CGDisplayModeRelease', None, [c.c_void_p])
get_class = bind(objc, 'objc_getClass', c.c_void_p, [c.c_char_p])
selector = bind(objc, 'sel_registerName', c.c_void_p, [c.c_char_p])
def message(receiver, name, result=c.c_void_p):
    return c.CFUNCTYPE(result, c.c_void_p, c.c_void_p)(('objc_msgSend', objc))(receiver, selector(name))
pool = message(message(get_class(b'NSAutoreleasePool'), b'alloc'), b'init')
assert pool, 'missing autorelease pool'
try:
    ids, count = (c.c_uint32 * 16)(), c.c_uint32()
    assert active(16, ids, c.byref(count)) == 0 and 0 < count.value < 16, 'active display list unavailable/overflow'
    screens = []
    for display in list(ids)[:count.value]:
        mode = copy_mode(display)
        assert mode, 'display mode unavailable'
        try:
            row = {'id': display}
            for key, name in [('width_px', 'CGDisplayModeGetPixelWidth'), ('height_px', 'CGDisplayModeGetPixelHeight'), ('width_pt', 'CGDisplayModeGetWidth'), ('height_pt', 'CGDisplayModeGetHeight')]:
                row[key] = bind(cg, name, c.c_size_t, [c.c_void_p])(mode)
            screens.append(row)
        finally: release_mode(mode)
    device = bind(metal, 'MTLCreateSystemDefaultDevice', c.c_void_p, [])()
    assert device, 'Metal device unavailable'
    try:
        name = message(message(device, b'name'), b'UTF8String', c.c_char_p)
        assert name, 'Metal device name unavailable'
        gpu = {'name': name.decode('utf-8'), 'registry_id': message(device, b'registryID', c.c_uint64)}
    finally: message(device, b'release', None)
    print(json.dumps({'origin': 'guest-coregraphics-metal', 'screens': sorted(screens, key=lambda s:s['id']), 'metal': gpu}, sort_keys=True))
finally: message(pool, b'drain', None)
'''


def validate_displays(value: dict) -> dict:
    if not isinstance(value, dict) or set(value) != {"origin", "screens", "metal"} or value["origin"] != "guest-coregraphics-metal":
        raise ObservationError("direct guest display identity required")
    screens, metal = value["screens"], value["metal"]
    if not isinstance(screens, list) or not 0 < len(screens) < 16:
        raise ObservationError("bounded active guest displays required")
    ids = set()
    for screen in screens:
        if (not isinstance(screen, dict) or set(screen) != {"id", "width_px", "height_px", "width_pt", "height_pt"}
                or any(type(v) is not int or not 0 < v <= (2**32-1 if k == "id" else 32768) for k, v in screen.items())
                or screen["id"] in ids):
            raise ObservationError("invalid guest display geometry")
        ids.add(screen["id"])
    if (not isinstance(metal, dict) or set(metal) != {"name", "registry_id"}
            or not isinstance(metal["name"], str) or not 0 < len(metal["name"]) <= 256
            or type(metal["registry_id"]) is not int or not 0 < metal["registry_id"] < 2**64):
        raise ObservationError("actual guest Metal device required")
    return value


def observe() -> dict:
    """Actual commands must execute in the throwaway guest's admin session."""
    if os.environ.get("TZ") != "UTC" or os.environ.get("LC_ALL") != "C":
        raise ObservationError("explicit UTC/C guest process environment required")
    if Path.home() != Path("/Users/admin"):
        raise ObservationError("not the disposable guest admin home")
    deadline = time.monotonic() + 60

    def command(*argv: str) -> bytes:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise ObservationError("guest environment deadline")
        result = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                check=False, timeout=remaining)
        if result.returncode or len(result.stdout) > 16 * 1024 * 1024:
            raise ObservationError(f"guest probe failed: {argv[0]}")
        return result.stdout

    def text(*argv: str) -> str:
        value = command(*argv).decode().strip()
        if not value:
            raise ObservationError("empty guest probe observation")
        return value

    state = console(command("/usr/sbin/ioreg", "-a", "-n", "Root", "-d1"))
    command("/usr/bin/pgrep", "-x", "WindowServer")
    displays = validate_displays(json.loads(command(sys.executable, "-I", "-B", "-c", NATIVE_DISPLAY_PROBE)))
    return {
        "schema": 1, "origin": "tart-guest",
        "os": text("/usr/bin/sw_vers", "-productVersion"),
        "os_build": text("/usr/bin/sw_vers", "-buildVersion"),
        "arch": text("/usr/bin/uname", "-m"),
        "model": text("/usr/sbin/sysctl", "-n", "hw.model"),
        "displays": [displays],
        "fonts": fonts((Path("/System/Library/Fonts"), Path("/Library/Fonts")),
                       Path.home() / "Library/Fonts", deadline),
        **state, "window_server": True, "timezone": "UTC", "locale": "C",
    }
