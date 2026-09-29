"""Contract tests for the BBR packaging lane (non-GUI, stdlib only).

Covers:
  - deterministic .icns generation (structure, sizes, byte stability);
  - .app bundle layout, token-filled Info.plist, permissions, fixed mtimes;
  - byte-identical rebuilds of the same binary;
  - the bounded launch-smoke contract against fake bundle executables.
"""

from __future__ import annotations

import math
import os
import plistlib
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
import zlib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
REPO = Path(__file__).resolve().parent.parent.parent

from package import build_app, gen_icon, smoke_launch  # noqa: E402

FAKE_BINARY = b"#!/bin/sh\n# fake bbr binary for packaging tests\nexit 0\n"
LIVE_SMOKE_BIN = (
    b"#!/bin/sh\n"
    b'trap "exit 0" TERM\n'
    b"echo $$ > \"$BBR_SMOKE_MARKER\"\n"
    b"while :; do sleep 0.2; done\n"
)
CRASH_SMOKE_BIN = b"#!/bin/sh\necho boom >&2\nexit 3\n"
STUBBORN_SMOKE_BIN = (
    b"#!/bin/sh\n"
    b"trap '' TERM\n"
    b"echo $$ > \"$BBR_SMOKE_MARKER\"\n"
    b"while :; do sleep 0.2; done\n"
)


def make_binary(root: Path, content: bytes = FAKE_BINARY,
                name: str = "binary") -> Path:
    path = root / name
    path.write_bytes(content)
    os.chmod(path, 0o755)
    return path


def make_app(root: Path, *, binary: bytes = FAKE_BINARY) -> Path:
    binary_path = make_binary(root, binary)
    app = root / "BBR.app"
    return build_app.build_app(binary_path, app, skip_codesign=True).app


def parse_png_ihdr(png: bytes) -> tuple[int, int]:
    assert png[:8] == b"\x89PNG\r\n\x1a\n"
    (length,) = struct.unpack(">I", png[8:12])
    assert png[12:16] == b"IHDR" and length == 13
    width, height, depth, color, _, _, _ = struct.unpack(">IIBBBBB", png[16:29])
    assert depth == 8 and color == 6, "expected RGBA8 PNG"
    return width, height


class IconTests(unittest.TestCase):
    def test_icns_structure_and_sizes(self) -> None:
        data = gen_icon.build_icns()
        self.assertEqual(data[:4], b"icns")
        (total,) = struct.unpack(">I", data[4:8])
        self.assertEqual(total, len(data))
        offset = 8
        seen: dict[bytes, int] = {}
        while offset < len(data):
            ctype, length = struct.unpack(">4sI", data[offset:offset + 8])
            self.assertGreaterEqual(length, 8)
            self.assertLessEqual(offset + length, len(data))
            png = data[offset + 8:offset + length]
            width, height = parse_png_ihdr(png)
            self.assertEqual(width, height)
            seen[ctype] = width
            offset += length
        self.assertEqual(offset, len(data))
        self.assertEqual(seen, {c: s for c, s in gen_icon.ICNS_SIZES})

    def test_icns_deterministic(self) -> None:
        self.assertEqual(gen_icon.build_icns(), gen_icon.build_icns())

    def test_render_shape(self) -> None:
        """The Phosphor b (DESIGN §18.1 D8): dark tile, light stem, accent
        arc as the bowl, faint track where the arc is not."""
        size = 128
        rgba = gen_icon.render_rgba(size)

        def pixel(x: float, y: float) -> tuple[int, ...]:
            """Sample at design-space coordinates (0..1024)."""
            px = int(x / gen_icon.BASE * size)
            py = int(y / gen_icon.BASE * size)
            i = (py * size + px) * 4
            return tuple(rgba[i:i + 4])

        # Outside the continuous-corner tile the pixel is transparent.
        self.assertEqual(pixel(4, 4), (0, 0, 0, 0))
        self.assertEqual(pixel(110, 110)[3], 0, "the corner curves away")
        # The tile body is the dark gradient.
        r, g, b, a = pixel(160, 520)
        self.assertEqual(a, 255)
        self.assertLess(max(r, g, b), 60)
        # The stem is the light ink bar.
        r, g, b, _ = pixel(382, 330)
        self.assertGreater(min(r, g, b), 200)
        # The arc's bottom (90°) is the accent: teal, green-dominant.
        cx, cy, radius, _ = gen_icon.BOWL
        r, g, b, _ = pixel(cx, cy + radius)
        self.assertGreater(g, 150)
        self.assertGreater(g, r + 60)
        # Where the arc is not (330°), only the faint track shows.
        angle = math.radians(330)
        r, g, b, _ = pixel(cx + radius * math.cos(angle), cy + radius * math.sin(angle))
        self.assertLess(g, 140)
        # The bowl's hole is dark (halo only), never the arc colour.
        r, g, b, _ = pixel(cx, cy)
        self.assertLess(max(r, g, b), 120)

    def test_mark_stays_legible_at_16_points(self) -> None:
        """Size ladder: at 16 px the stem and the accent bowl still resolve
        to distinct pixels, and the tile corners stay open."""
        rgba = gen_icon.downsample(gen_icon.render_rgba(256), 256, 16)
        pixels = [tuple(rgba[i:i + 4]) for i in range(0, len(rgba), 4)]
        def luminance(p: tuple[int, ...]) -> float:
            def lin(c: int) -> float:
                v = c / 255
                return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4
            return 0.2126 * lin(p[0]) + 0.7152 * lin(p[1]) + 0.0722 * lin(p[2])

        tile = pixels[3 * 16 + 3]  # plain tile body near the top-left
        teal = [p for p in pixels if p[1] > 110 and p[1] > p[0] + 40]
        # The stem: neutral (not teal) pixels at ≥ 3:1 against the tile.
        stem = [p for p in pixels
                if p[3] == 255 and abs(p[1] - p[0]) < 25
                and (luminance(p) + 0.05) / (luminance(tile) + 0.05) >= 3.0]
        self.assertGreaterEqual(len(teal), 6, "the accent bowl reads")
        self.assertGreaterEqual(len(stem), 3, "the stem reads")
        self.assertLess(pixels[0][3], 128, "the tile corner stays open")

    def test_vector_master_matches_the_renderer(self) -> None:
        """The committed SVG master carries the renderer's key geometry."""
        master = (REPO / "packaging" / "icon" / "bbr-icon.svg").read_text()
        cx, cy, radius, stroke = gen_icon.BOWL
        for token in (f'cx="{cx:g}"', f'cy="{cy:g}"', f'r="{radius:g}"',
                      f'stroke-width="{stroke:g}"', "#3DF0CE", "#0A7967",
                      "#1C2127", "#0B0D10", "#ECEEF1"):
            self.assertIn(token, master)


class BundleTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_layout_and_plist(self) -> None:
        make_binary(self.root)
        app = make_app(self.root)
        contents = app / "Contents"
        self.assertTrue((contents / "MacOS" / "bbr").is_file())
        self.assertEqual((contents / "MacOS" / "bbr").stat().st_mode & 0o777, 0o755)
        self.assertTrue((contents / "PkgInfo").is_file())
        self.assertEqual((contents / "PkgInfo").read_bytes(), b"APPL????")
        self.assertTrue((contents / "Resources" / "AppIcon.icns").is_file())

        with (contents / "Info.plist").open("rb") as handle:
            plist = plistlib.load(handle)
        self.assertEqual(plist["CFBundleExecutable"], "bbr")
        self.assertEqual(plist["CFBundleIdentifier"], build_app.DEFAULT_BUNDLE_ID)
        self.assertEqual(plist["CFBundlePackageType"], "APPL")
        self.assertEqual(plist["CFBundleIconFile"], "AppIcon")
        self.assertEqual(plist["CFBundleShortVersionString"], build_app.workspace_version())
        self.assertEqual(plist["CFBundleVersion"], build_app.workspace_version())
        self.assertTrue(plist["NSHighResolutionCapable"])
        # Blueprint D11: WebKit asks macOS for the devices a site was
        # allowed only when the app says why it uses them.
        self.assertIn("camera", plist["NSCameraUsageDescription"])
        self.assertIn("microphone", plist["NSMicrophoneUsageDescription"])

        # Queue item 13: the bundle declares exactly the schemes BBR genuinely
        # handles — http/https, the supported web schemes of the merged
        # contract. A scheme the contract rejects must never be declared (it
        # would make BBR a black hole for the user's links).
        url_types = plist["CFBundleURLTypes"]
        self.assertEqual(len(url_types), 1)
        schemes = url_types[0]["CFBundleURLSchemes"]
        self.assertEqual(sorted(schemes), ["http", "https"])

    def test_deterministic_rebuild(self) -> None:
        binary = make_binary(self.root)
        first = self.root / "a.app"
        second = self.root / "b.app"
        build_app.build_app(binary, first, skip_codesign=True, mtime=1577836800)
        build_app.build_app(binary, second, skip_codesign=True, mtime=1577836800)
        self.assertEqual(
            [(p.relative_to(first), p.read_bytes()) for p in sorted(first.rglob("*")) if p.is_file()],
            [(p.relative_to(second), p.read_bytes()) for p in sorted(second.rglob("*")) if p.is_file()],
        )
        for path in sorted(first.rglob("*")):
            self.assertEqual(int(path.stat().st_mtime), 1577836800, str(path))

    def test_version_override_and_bundle_id(self) -> None:
        make_binary(self.root)
        app = self.root / "v.app"
        build_app.build_app(self.root / "binary", app, skip_codesign=True,
                            version="9.8.7", bundle_id="com.example.test")
        with (app / "Contents" / "Info.plist").open("rb") as handle:
            plist = plistlib.load(handle)
        self.assertEqual(plist["CFBundleShortVersionString"], "9.8.7")
        self.assertEqual(plist["CFBundleIdentifier"], "com.example.test")

    def test_rejects_missing_binary(self) -> None:
        with self.assertRaises(FileNotFoundError):
            build_app.build_app(self.root / "nope", self.root / "x.app",
                                skip_codesign=True)


class SignedBundleTests(unittest.TestCase):
    """Signed-build invariants; skipped where ad-hoc codesign is unavailable."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    @unittest.skipUnless(
        sys.platform == "darwin" and shutil.which("codesign") is not None,
        "ad-hoc codesign requires macOS with codesign",
    )
    def test_signed_bundle_verifies_with_fixed_mtimes(self) -> None:
        # codesign only accepts a real Mach-O; the tiny universal /usr/bin/true
        # is present on every macOS install and ad-hoc signs cleanly.
        system_true = shutil.which("true")
        if system_true is None:
            self.skipTest("system `true` binary not found")
        app = self.root / "signed.app"
        result = build_app.build_app(Path(system_true), app, mtime=1577836800)
        self.assertTrue(result.signed)
        self.assertTrue((app / "Contents" / "_CodeSignature").is_dir())
        verify = subprocess.run(
            ["codesign", "--verify", "--strict", str(app)],
            capture_output=True, text=True)
        self.assertEqual(verify.returncode, 0, verify.stderr)
        # codesign touches the executable and bundle directories with
        # wall-clock mtimes; the assembler must pin every path back to the
        # fixed epoch so the signed bundle keeps the same invariant.
        for path in sorted(app.rglob("*"), reverse=True):
            self.assertEqual(int(path.stat().st_mtime), 1577836800, str(path))
        self.assertEqual(int(app.stat().st_mtime), 1577836800, str(app))


class SmokeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def bundle_with(self, script: bytes) -> Path:
        binary = make_binary(self.root, script, name="fake-bbr")
        app = self.root / "BBR.app"
        build_app.build_app(binary, app, skip_codesign=True)
        return app

    def test_alive_then_terminated(self) -> None:
        app = self.bundle_with(LIVE_SMOKE_BIN)
        result = smoke_launch.probe_app(app, observe=1.0, launch_timeout=10,
                                        verify_signed=False)
        self.assertTrue(result.ok, result.message)
        self.assertIn("stayed alive", result.message)
        self.assertIn("terminated", result.message)

    def test_early_crash_fails_with_stderr(self) -> None:
        app = self.bundle_with(CRASH_SMOKE_BIN)
        result = smoke_launch.probe_app(app, observe=1.0, launch_timeout=10,
                                        verify_signed=False)
        self.assertFalse(result.ok)
        self.assertEqual(result.exit_code, 3)
        self.assertIn("boom", result.stderr_tail)

    def test_missing_executable_fails(self) -> None:
        make_binary(self.root)
        app = self.root / "empty.app"
        (app / "Contents" / "MacOS").mkdir(parents=True)
        shutil.copy(self.root / "binary", app / "Contents" / "MacOS" / "bbr")
        # Deliberately drop the executable entry from the plist.
        (app / "Contents").joinpath("Info.plist").write_text(
            "<plist version=\"1.0\"><dict></dict></plist>")
        result = smoke_launch.probe_app(app, observe=1.0, launch_timeout=10,
                                        verify_signed=False)
        self.assertFalse(result.ok)
        self.assertIn("CFBundleExecutable", result.message)

    def test_stubborn_process_is_reaped(self) -> None:
        app = self.bundle_with(STUBBORN_SMOKE_BIN)
        marker = self.root / "pid.txt"
        env = {**os.environ, "BBR_SMOKE_MARKER": str(marker)}
        result = smoke_launch.probe_app(app, observe=1.0, launch_timeout=10,
                                        verify_signed=False, env=env)
        self.assertTrue(result.ok, result.message)
        self.assertEqual(result.termination, "killed")
        pid = int(marker.read_text().strip())
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)  # the stubborn process group must be gone


if __name__ == "__main__":
    unittest.main()
