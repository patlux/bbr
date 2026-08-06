#!/usr/bin/env python3
"""Contract tests for scripts/with_gui_lock.py."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

SCRIPT = Path(__file__).with_name("with_gui_lock.py")


class GuiLockTests(unittest.TestCase):
    def environment(self, directory: str, timeout: str = "2") -> dict[str, str]:
        return {
            **os.environ,
            "BBR_GUI_LOCK_DIR": directory,
            "BBR_GUI_LOCK_KEY": "contract-test",
            "BBR_GUI_LOCK_TIMEOUT_SECONDS": timeout,
        }

    def run_wrapper(
        self, directory: str, *command: str, timeout: str = "2"
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--", *command],
            env=self.environment(directory, timeout),
            capture_output=True,
            text=True,
            check=False,
        )

    def test_forwards_command_exit_status(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = self.run_wrapper(
                directory, sys.executable, "-c", "raise SystemExit(23)"
            )

        self.assertEqual(result.returncode, 23)
        self.assertIn("BBR GUI gate acquired", result.stderr)

    def test_serializes_commands_using_the_same_key(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "marker.txt"
            first = subprocess.Popen(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--",
                    sys.executable,
                    "-c",
                    (
                        "from pathlib import Path; import time; "
                        f"p=Path({str(marker)!r}); p.write_text('held'); "
                        "time.sleep(0.6)"
                    ),
                ],
                env=self.environment(directory),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            deadline = time.monotonic() + 2
            while not marker.exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue(marker.exists(), "first command never acquired the lock")

            started = time.monotonic()
            second = self.run_wrapper(directory, sys.executable, "-c", "pass")
            elapsed = time.monotonic() - started
            first_stdout, first_stderr = first.communicate(timeout=2)

        self.assertEqual(first.returncode, 0, (first_stdout, first_stderr))
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertGreaterEqual(elapsed, 0.45)

    def test_times_out_with_holder_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "marker.txt"
            first = subprocess.Popen(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--",
                    sys.executable,
                    "-c",
                    (
                        "from pathlib import Path; import time; "
                        f"p=Path({str(marker)!r}); p.write_text('held'); "
                        "time.sleep(1)"
                    ),
                ],
                env=self.environment(directory),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            deadline = time.monotonic() + 2
            while not marker.exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue(marker.exists(), "first command never acquired the lock")

            second = self.run_wrapper(
                directory, sys.executable, "-c", "pass", timeout="0.1"
            )
            first.communicate(timeout=2)

        self.assertEqual(second.returncode, 75)
        self.assertIn("current holder", second.stderr)
        self.assertIn("contract-test.lock", second.stderr)


if __name__ == "__main__":
    unittest.main()
