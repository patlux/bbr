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

    def test_host_cannot_be_reported_as_guest(self):
        with patch.object(Path, "home", return_value=Path("/Users/patwoz")), \
             patch.dict(vm_observe.os.environ, {"TZ": "UTC", "LC_ALL": "C"}), \
             patch.object(vm_observe.subprocess, "run") as run:
            with self.assertRaises(vm_observe.ObservationError):
                vm_observe.observe()
            run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
