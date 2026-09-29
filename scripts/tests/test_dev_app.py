"""Contract tests for the incremental development bundle (`just run`).

Non-GUI, stdlib only: bundle identity and layout, incremental updates, the
cached icon, and the one-time WebKit site-data copy.
"""

from __future__ import annotations

import os
import plistlib
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from package import dev_app, gen_icon  # noqa: E402

FAKE_BINARY = b"#!/bin/sh\nexit 0\n"
FAKE_ICNS = b"icns-fixture"


class DevBundleTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.binary = self.root / "bbr"
        self.binary.write_bytes(FAKE_BINARY)
        os.chmod(self.binary, 0o755)
        self.app = self.root / "out" / "BBR.app"
        # Rendering the real 1024 px icon takes seconds; the renderer has its
        # own tests in test_packaging.py.
        patcher = mock.patch.object(gen_icon, "build_icns", return_value=FAKE_ICNS)
        self.build_icns = patcher.start()
        self.addCleanup(patcher.stop)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def build(self) -> bool:
        return dev_app.build_dev_app(self.binary, self.app, version="1.2.3")

    def test_bundle_has_a_distinct_dev_identity_and_the_app_icon(self) -> None:
        self.assertTrue(self.build())
        contents = self.app / "Contents"
        executable = contents / "MacOS" / "bbr"
        self.assertEqual(executable.read_bytes(), FAKE_BINARY)
        self.assertEqual(executable.stat().st_mode & 0o777, 0o755)
        self.assertEqual((contents / "PkgInfo").read_bytes(), b"APPL????")
        self.assertEqual((contents / "Resources" / "AppIcon.icns").read_bytes(), FAKE_ICNS)
        with (contents / "Info.plist").open("rb") as handle:
            plist = plistlib.load(handle)
        self.assertEqual(plist["CFBundleIdentifier"], dev_app.DEV_BUNDLE_ID)
        self.assertNotEqual(plist["CFBundleIdentifier"], "dev.patwoz.bbr")
        self.assertEqual(plist["CFBundleName"], "BBR")
        self.assertEqual(plist["CFBundleExecutable"], "bbr")
        self.assertEqual(plist["CFBundleIconFile"], "AppIcon")
        self.assertEqual(plist["CFBundleShortVersionString"], "1.2.3")
        # A regular Dock app with a menu bar, never an agent/background app.
        self.assertNotIn("LSUIElement", plist)
        self.assertNotIn("LSBackgroundOnly", plist)

    def test_rebuild_is_incremental(self) -> None:
        self.build()
        self.assertFalse(self.build(), "unchanged metadata reports no change")
        self.assertEqual(self.build_icns.call_count, 1, "the icon renders once")

    def test_rebuild_replaces_the_binary(self) -> None:
        self.build()
        self.binary.write_bytes(b"#!/bin/sh\nexit 1\n")
        self.assertFalse(self.build())
        self.assertEqual((self.app / "Contents" / "MacOS" / "bbr").read_bytes(),
                         b"#!/bin/sh\nexit 1\n")

    def test_icon_rerenders_when_the_renderer_changes(self) -> None:
        self.build()
        with mock.patch.object(dev_app, "icon_fingerprint", return_value="changed"):
            self.assertTrue(self.build())
        self.assertEqual(self.build_icns.call_count, 2)

    def test_rejects_missing_binary(self) -> None:
        with self.assertRaises(FileNotFoundError):
            dev_app.build_dev_app(self.root / "missing", self.app)


class RunnerTests(unittest.TestCase):
    """`just run` uses the script as Cargo's runner: `dev_app.py --exec
    <binary> <program args...>`."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.debug = Path(self.tmp.name) / "debug"
        self.debug.mkdir()
        self.binary = self.debug / "bbr"
        self.binary.write_bytes(FAKE_BINARY)
        os.chmod(self.binary, 0o755)
        patcher = mock.patch.object(gen_icon, "build_icns", return_value=FAKE_ICNS)
        patcher.start()
        self.addCleanup(patcher.stop)
        patcher = mock.patch.object(dev_app, "refresh_launch_services")
        self.refresh = patcher.start()
        self.addCleanup(patcher.stop)

    def test_execs_the_bundle_executable_with_the_program_arguments(self) -> None:
        with mock.patch.object(dev_app.os, "execv") as execv:
            code = dev_app.main(["dev_app.py", "--no-seed", "--exec", str(self.binary),
                                 "--bbr-initial-url", "https://example.com"])
        self.assertEqual(code, 0)
        executable = str(self.debug.resolve() / "BBR.app" / "Contents" / "MacOS" / "bbr")
        execv.assert_called_once_with(
            executable, [executable, "--bbr-initial-url", "https://example.com"])

    def test_blocked_copy_never_launches(self) -> None:
        blocked = dev_app.SeedBlocked("busy")
        with mock.patch.object(dev_app, "seed_webkit_data", side_effect=blocked), \
                mock.patch.object(dev_app.os, "execv") as execv:
            code = dev_app.main(["dev_app.py", "--exec", str(self.binary)])
        self.assertEqual(code, 3)
        execv.assert_not_called()

    def test_launch_services_refresh_only_on_metadata_change(self) -> None:
        argv = ["dev_app.py", "--no-seed", str(self.binary)]
        self.assertEqual(dev_app.main(argv), 0)
        self.assertEqual(dev_app.main(argv), 0)
        self.assertEqual(self.refresh.call_count, 1 if sys.platform == "darwin" else 0)


class SeedWebKitDataTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.webkit = Path(self.tmp.name)
        self.source = self.webkit / "bbr"
        cookies = self.source / "WebsiteDataStore" / "profile" / "Cookies"
        cookies.mkdir(parents=True)
        (cookies / "Cookies.binarycookies").write_bytes(b"session")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def seed(self, running: bool = False) -> str | None:
        return dev_app.seed_webkit_data("dev.example.bbr", webkit_root=self.webkit,
                                        is_running=lambda: running)

    def test_copies_the_unbundled_site_data_once_and_keeps_the_source(self) -> None:
        self.assertIn("copied", self.seed())
        copied = (self.webkit / "dev.example.bbr" / "WebsiteDataStore" / "profile"
                  / "Cookies" / "Cookies.binarycookies")
        self.assertEqual(copied.read_bytes(), b"session")
        self.assertTrue((self.source / "WebsiteDataStore").is_dir(), "source untouched")
        self.assertFalse(any(p.name.endswith(".seeding") for p in self.webkit.iterdir()))
        copied.write_bytes(b"newer")
        self.assertIsNone(self.seed(), "an existing target is never overwritten")
        self.assertEqual(copied.read_bytes(), b"newer")

    def test_refuses_while_bbr_runs(self) -> None:
        with self.assertRaises(dev_app.SeedBlocked):
            self.seed(running=True)
        self.assertFalse((self.webkit / "dev.example.bbr").exists())

    def test_nothing_to_copy_without_source(self) -> None:
        empty = tempfile.TemporaryDirectory()
        self.addCleanup(empty.cleanup)
        self.assertIsNone(dev_app.seed_webkit_data(
            "dev.example.bbr", webkit_root=Path(empty.name), is_running=lambda: True))


if __name__ == "__main__":
    unittest.main()
