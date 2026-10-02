"""Guest-observation parsers over synthetic fixture data only."""
from pathlib import Path
import hashlib
import plistlib
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_observe


class ObservationContracts(unittest.TestCase):
    def test_only_one_unlocked_guest_admin_console_is_accepted(self):
        valid = {"kCGSSessionOnConsoleKey": True, "kCGSSessionUserNameKey": "admin",
                 "CGSSessionScreenIsLocked": False}
        def encode(users):
            return plistlib.dumps([{"IOConsoleUsers": users}])
        self.assertEqual(vm_observe.console(encode([valid]))["console_user"], "admin")
        for users in [[], [valid, valid], [{**valid, "kCGSSessionUserNameKey": "patwoz"}],
                      [{**valid, "CGSSessionScreenIsLocked": True}],
                      [{**valid, "kCGSSessionOnConsoleKey": False}]]:
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.console(encode(users))

    def test_macos27_direct_root_keeps_console_identity_and_lock_checks(self):
        root = {"IORegistryEntryName": "Root", "IOConsoleUsers": [{
            "kCGSSessionOnConsoleKey": True, "kCGSSessionUserNameKey": "admin",
            "CGSSessionScreenIsLocked": False}]}
        self.assertEqual(vm_observe.console(plistlib.dumps(root))["console_user"], "admin")
        for bad in ({**root, "IORegistryEntryName": "Other"}, {}, [root, root],
                    {**root, "IOConsoleUsers": []},
                    {**root, "IOConsoleUsers": [{**root["IOConsoleUsers"][0], "CGSSessionScreenIsLocked": True}]}):
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.console(plistlib.dumps(bad))

    def test_exact_font_bytes_and_no_user_font_or_symlink_fallback(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            fonts = root / "fonts"
            user = root / "user"
            fonts.mkdir()
            user.mkdir()
            file = fonts / "fixture.ttf"
            file.write_bytes(b"synthetic fixture, not actual font pixels")
            found = vm_observe.fonts((fonts,), user, time.monotonic() + 5)
            self.assertEqual(found[str(file)], hashlib.sha256(file.read_bytes()).hexdigest())
            (user / "custom.ttf").write_bytes(b"fixture")
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.fonts((fonts,), user, time.monotonic() + 5)
            (user / "custom.ttf").unlink()
            (fonts / "alias").symlink_to(file)
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.fonts((fonts,), user, time.monotonic() + 5)
            (fonts / "alias").unlink()
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.fonts((fonts,), user, time.monotonic() - 1)

    def test_empty_home_manager_font_metadata_is_not_a_font_or_alias(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            fonts, user = root / "fonts", root / "user"
            fonts.mkdir(); user.mkdir()
            (fonts / "fixture.ttf").write_bytes(b"font fixture")
            (user / "HomeManager").mkdir()
            marker = user / ".home-manager-fonts-version"
            marker.symlink_to("/nix/store/pz885hwbsg4vjxda3d5k33917x1irg5z-home-manager-files/Library/Fonts/.home-manager-fonts-version")
            expected = vm_observe.fonts((fonts,), root / "absent", time.monotonic() + 5)
            self.assertEqual(vm_observe.fonts((fonts,), user, time.monotonic() + 5), expected)
            (user / "HomeManager/custom.ttf").write_bytes(b"unapproved")
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.fonts((fonts,), user, time.monotonic() + 5)
            (user / "HomeManager/custom.ttf").unlink()
            marker.unlink(); marker.symlink_to(fonts / "fixture.ttf")
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.fonts((fonts,), user, time.monotonic() + 5)
            marker.unlink(); (user / "HomeManager").rmdir()
            (user / "HomeManager").symlink_to(fonts)
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.fonts((fonts,), user, time.monotonic() + 5)

    def test_exact_system_font_alias_hashes_target_and_rejects_drift(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            system, library = root / "system", root / "library"
            system.mkdir(); library.mkdir()
            target, alias = system / "Arial.ttf", library / "Arial.ttf"
            target.write_bytes(b"fixture system font")
            alias.symlink_to(target)
            with patch.object(vm_observe, "FONT_ALIASES", {str(alias): str(target)}):
                found = vm_observe.fonts((system, library), root / "user", time.monotonic() + 5)
                self.assertEqual(found[str(alias)], found[str(target)])
                other = system / "Other.ttf"
                other.write_bytes(target.read_bytes())
                alias.unlink(); alias.symlink_to(other)
                with self.assertRaises(vm_observe.ObservationError):
                    vm_observe.fonts((system, library), root / "user", time.monotonic() + 5)
                alias.unlink(); alias.symlink_to(target)
                target.unlink(); target.symlink_to(other)
                with self.assertRaises(vm_observe.ObservationError):
                    vm_observe.fonts((system, library), root / "user", time.monotonic() + 5)

    def test_direct_guest_display_probe_is_bounded_complete_and_not_host_fallback(self):
        import copy
        value = {"origin": "guest-coregraphics-metal", "screens": [{
            "id": 1, "width_px": 3200, "height_px": 1800,
            "width_pt": 3200, "height_pt": 1800}],
            "metal": {"name": "Apple Paravirtual device", "registry_id": 123}}
        self.assertEqual(vm_observe.validate_displays(value), value)
        for field, replacement in (("origin", "host"), ("screens", []), ("metal", {})):
            bad = copy.deepcopy(value); bad[field] = replacement
            with self.assertRaises(vm_observe.ObservationError): vm_observe.validate_displays(bad)
        for field, replacement in (("width_px", 0), ("height_pt", -1), ("id", True)):
            bad = copy.deepcopy(value); bad["screens"][0][field] = replacement
            with self.assertRaises(vm_observe.ObservationError): vm_observe.validate_displays(bad)
        compile(vm_observe.NATIVE_DISPLAY_PROBE, "guest display probe", "exec")
        self.assertIn("CGGetActiveDisplayList", vm_observe.NATIVE_DISPLAY_PROBE)
        self.assertIn("MTLCreateSystemDefaultDevice", vm_observe.NATIVE_DISPLAY_PROBE)
        self.assertNotIn("NSApplication", vm_observe.NATIVE_DISPLAY_PROBE)

    def test_host_cannot_be_reported_as_guest(self):
        with patch.object(Path, "home", return_value=Path("/Users/patwoz")), \
             patch.dict(vm_observe.os.environ, {"TZ": "UTC", "LC_ALL": "C"}), \
             patch.object(vm_observe.subprocess, "run") as run:
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.observe()
            run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
