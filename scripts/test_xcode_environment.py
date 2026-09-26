"""Check the actual default just recipe without launching the browser."""

import json
import os
import pty
import select
import signal
import time
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_XCODE = "/Applications/Xcode-27.0.0-Beta.6.app/Contents/Developer"
KEYS = ("DEVELOPER_DIR", "SDKROOT", "CC", "CXX",
        "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER",
        "CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER")


@unittest.skipUnless(os.uname().sysname == "Darwin", "requires installed Xcode")
class XcodeEnvironmentTests(unittest.TestCase):
    def test_default_keeps_terminal_foreground(self):
        with tempfile.TemporaryDirectory() as directory:
            probe = Path(directory) / "mbx"
            probe.write_text(
                '#!/usr/bin/env python3\n'
                'import os\n'
                'print("BBR_FOREGROUND=" + str(os.getpgrp() == os.tcgetpgrp(0)), flush=True)\n'
            )
            probe.chmod(0o755)
            pid, terminal = pty.fork()
            if pid == 0:
                os.chdir(ROOT)
                os.environ["PATH"] = directory + os.pathsep + os.environ["PATH"]
                os.execvp("just", ["just"])
            output = bytearray()
            exited = False
            try:
                deadline = time.monotonic() + 15
                while time.monotonic() < deadline:
                    if select.select([terminal], [], [], 0.1)[0]:
                        try:
                            data = os.read(terminal, 4096)
                        except OSError:
                            break
                        if not data:
                            break
                        output.extend(data)
                    if os.waitpid(pid, os.WNOHANG)[0]:
                        exited = True
                        break
                self.assertIn(b"BBR_FOREGROUND=True", output, output.decode(errors="replace"))
            finally:
                os.close(terminal)
                if not exited:
                    try:
                        os.kill(pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    os.waitpid(pid, 0)

    def run_default(self, developer_dir=None):
        with tempfile.TemporaryDirectory() as directory:
            probe = Path(directory) / "mbx"
            probe.write_text(
                '#!/usr/bin/env python3\n'
                'import json, os, sys\n'
                f'print(json.dumps({{"env": {{k: os.environ.get(k) for k in {KEYS!r}}}, '
                '"args": sys.argv[1:]}))\n'
            )
            probe.chmod(0o755)
            env = os.environ.copy()
            env.pop("BBR_XCODE_DEVELOPER_DIR", None)
            env.update({key: "/stale/nix/toolchain" for key in KEYS})
            env["PATH"] = directory + os.pathsep + env["PATH"]
            if developer_dir:
                env["BBR_XCODE_DEVELOPER_DIR"] = developer_dir
            result = subprocess.run(
                ["just"], cwd=ROOT, env=env, text=True,
                capture_output=True, timeout=30,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            return json.loads(result.stdout)

    def assert_toolchain(self, developer_dir):
        result = self.run_default(developer_dir)
        selected = developer_dir or DEFAULT_XCODE
        env = os.environ.copy()
        env["DEVELOPER_DIR"] = selected

        def xcrun(*args):
            return subprocess.check_output(
                ["/usr/bin/xcrun", *args], env=env, text=True, timeout=10,
            ).strip()

        self.assertEqual(result["args"], ["run", "-p", "bbr"])
        self.assertEqual(result["env"], {
            "DEVELOPER_DIR": selected,
            "SDKROOT": xcrun("--sdk", "macosx", "--show-sdk-path"),
            "CC": xcrun("--find", "clang"),
            "CXX": xcrun("--find", "clang++"),
            "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER": xcrun("--sdk", "macosx", "--find", "clang"),
            "CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER": xcrun("--sdk", "macosx", "--find", "clang"),
        })

    def test_default_replaces_inherited_toolchain(self):
        self.assert_toolchain(None)

    def test_explicit_xcode_override_selects_matching_tools(self):
        alternate = "/Applications/Xcode-26.2.0.app/Contents/Developer"
        if not Path(alternate).is_dir():
            self.skipTest("alternate Xcode is not installed")
        self.assert_toolchain(alternate)


if __name__ == "__main__":
    unittest.main()
