"""Portable contracts for BBR's adapter, not a copy of comparator tests."""
import fcntl
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import tomllib
import unittest
from unittest.mock import patch

from accept import acceptance_args
import capture
from capture import output_directory, write_inventory
from environment import environment
from support import CASES, ROOT, build, executable, run, verify_tool


class AdapterTests(unittest.TestCase):
    def test_pinned_tool_is_unmodified(self):
        self.assertEqual(verify_tool()["version"], "0.1.1")

    def test_tool_tampering_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch("support.TOOL", Path(directory)):
                with self.assertRaisesRegex(ValueError, "Missing/non-regular"):
                    verify_tool()

    def test_extra_vendor_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            tool = Path(directory)
            (tool / 'injected.rs').write_text('// not in pinned revision')
            with patch('support.TOOL', tool), patch('support.json.loads', return_value={'files': {}}):
                with self.assertRaisesRegex(ValueError, 'Unexpected pinned tool source'):
                    verify_tool()

    def test_evicted_mbx_target_link_is_not_vendor_source(self):
        with tempfile.TemporaryDirectory() as directory:
            tool = Path(directory)
            (tool / 'target').symlink_to(tool / 'evicted-managed-target', target_is_directory=True)
            with patch('support.TOOL', tool), patch('support.json.loads', return_value={'files': {}}):
                self.assertEqual(verify_tool(), {'files': {}})
            (tool / 'target').unlink()
            (tool / 'target').write_text('unexpected regular file')
            with patch('support.TOOL', tool), patch('support.json.loads', return_value={'files': {}}):
                with self.assertRaisesRegex(ValueError, 'Unexpected pinned tool source'):
                    verify_tool()

    def test_capture_timeout_reaps_group_before_releasing_gui_lock(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            gate = root / 'gui.lock'
            runner = root / 'runner'
            pidfile = root / 'child.pid'
            runner.write_text(
                f'#!{sys.executable}\n'
                'import pathlib, signal, subprocess, sys, time\n'
                'signal.signal(signal.SIGTERM, signal.SIG_IGN)\n'
                "child = subprocess.Popen([sys.executable, '-c', "
                "'import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(60)'])\n"
                f'pathlib.Path({str(pidfile)!r}).write_text(str(child.pid))\n'
                'time.sleep(60)\n'
            )
            runner.chmod(0o700)

            def bounded_run(*args, **kwargs):
                try:
                    return run(*args, **kwargs)
                finally:
                    with gate.open('a+') as contender:
                        with self.assertRaises(BlockingIOError):
                            fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    pid = int(pidfile.read_text())
                    state = subprocess.run(['ps', '-p', str(pid), '-o', 'stat='],
                                           capture_output=True, text=True).stdout.strip()
                    self.assertTrue(not state or state.startswith('Z'), 'child survived lock owner')

            with patch('capture.with_gui_lock.lock_path', return_value=gate), \
                 patch('capture.CAPTURE_SECONDS', 1), patch('capture.run', side_effect=bounded_run):
                with self.assertRaises(subprocess.TimeoutExpired):
                    capture.run_suite(runner, os.environ.copy())
            with gate.open('a+') as contender:
                fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_inventory_has_complete_light_dark_pairs(self):
        self.assertEqual(len(CASES), 28)
        required = json.loads((ROOT / 'scripts/visual/catalogs/browser.json').read_text())
        self.assertEqual(set(CASES), {
            f"{row['base']}--{mode}"
            for row in required['snapshots'] if row['state'] == 'required'
            for mode in ('light', 'dark')
        })
        for case in CASES:
            base, _ = case.rsplit("--", 1)
            self.assertIn(base + "--light", CASES)
            self.assertIn(base + "--dark", CASES)

    def test_inventory_rejects_missing_and_extra_pngs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for case in CASES[:-1]:
                (root / f"{case}.png").write_bytes(b"fixture")
            with self.assertRaisesRegex(ValueError, "Incomplete"):
                write_inventory(root)
            self.assertFalse((root / "inventory.json").exists())
            (root / f"{CASES[-1]}.png").write_bytes(b"fixture")
            write_inventory(root)
            self.assertEqual(json.loads((root / "inventory.json").read_text()), list(CASES))
            (root / "unexpected.png").write_bytes(b"fixture")
            with self.assertRaisesRegex(ValueError, "extra"):
                write_inventory(root)

    def test_inventory_rejects_non_regular_or_empty_pngs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for case in CASES:
                (root / f'{case}.png').write_bytes(b'fixture')
            invalid = root / f'{CASES[0]}.png'
            invalid.write_bytes(b'')
            with self.assertRaisesRegex(ValueError, 'Non-regular or empty'):
                write_inventory(root)
            invalid.unlink()
            invalid.mkdir()
            with self.assertRaisesRegex(ValueError, 'Non-regular or empty'):
                write_inventory(root)
            invalid.rmdir()
            invalid.symlink_to(root / f'{CASES[1]}.png')
            with self.assertRaisesRegex(ValueError, 'Non-regular or empty'):
                write_inventory(root)

    def test_capture_output_is_fresh_and_confined(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            with patch("capture.ROOT", root):
                path = root / "reports/visual/runs/fresh"
                self.assertEqual(output_directory(str(path)), path)
                path.mkdir(parents=True)
                for invalid in [str(path), str(root / "reports/visual/runs"),
                                str(root / "elsewhere"), "relative", str(path / "../escape")]:
                    with self.assertRaises(ValueError, msg=invalid):
                        output_directory(invalid)
                link = root / "reports/visual/runs/link"
                link.symlink_to(root, target_is_directory=True)
                with self.assertRaisesRegex(ValueError, "symlink"):
                    output_directory(str(link / "fresh"))

    def test_acceptance_names_only_explicit_reviewed_cases(self):
        self.assertEqual(acceptance_args(["run", CASES[0], CASES[1]]),
                         ["accept", "--run", "run", "--case", CASES[0], "--case", CASES[1]])
        for args in [[], ["run"], ["run", "*"], ["run", "--adopt-environment"],
                     ["run", CASES[0], CASES[0]]]:
            with self.assertRaises(ValueError):
                acceptance_args(args)

    def test_executable_selection_rejects_ambiguous_or_missing_build(self):
        artifact = {"executable": sys.executable, "target": {"name": "bbr"},
                    "profile": {"test": False}}
        self.assertEqual(executable([artifact], "bbr"), Path(sys.executable))
        for artifacts in [[], [artifact, artifact]]:
            with self.assertRaises(ValueError):
                executable(artifacts, "bbr")
        with self.assertRaises(ValueError):
            executable([artifact], "bbr", test=True)

    def test_config_seals_tool_renderer_assets_and_capture_inputs(self):
        config = tomllib.loads((ROOT / "visual.toml").read_text())
        self.assertEqual(config["capture"], ["python3", "scripts/visual/capture.py"])
        self.assertIn("vendor/cargo-gpui-visual", config["source_policy"]["scopes"])
        self.assertIn("vendor/gpui-component", config["source_policy"]["scopes"])
        self.assertIn("crates", config["source_policy"]["scopes"])
        self.assertNotIn("tests/visual/baselines", config["source_policy"]["scopes"])
        self.assertEqual(config["runs"], "reports/visual/runs")

    def test_independent_cli_keeps_xcode_host_artifacts_unstripped(self):
        result = subprocess.CompletedProcess([], 0, '{"reason":"build-finished"}\n')
        with patch("support.run", return_value=result) as invoke:
            self.assertEqual(build(ROOT / "vendor/cargo-gpui-visual/Cargo.toml"), [])
        argv = invoke.call_args.args[0]
        self.assertIn("profile.release.build-override.strip=false", argv)
        self.assertEqual(argv[:2], ["mbx", "build"])

    def test_environment_uses_upstream_layer_and_explicit_native_exclusions(self):
        with patch("environment.platform.system", return_value="Darwin"), \
             patch("environment.output", return_value="test-host"):
            value = environment()
        self.assertEqual(value["layer"], "gpui")
        self.assertEqual(value["capture_scope"], "production-chrome-only")
        self.assertEqual(value["scale"], 2)
        self.assertEqual(value["contrast"], "standard")
        self.assertIn("WKWebView", value["native_pixels"])

    def test_process_exit_and_stdout_are_forwarded(self):
        result = run([sys.executable, "-c", "print('ok'); raise SystemExit(7)"],
                     capture=True, check=False)
        self.assertEqual((result.returncode, result.stdout), (7, "ok\n"))

    def test_timeout_kills_descendants(self):
        with tempfile.TemporaryDirectory() as directory:
            pidfile = Path(directory) / "child.pid"
            code = (
                "import pathlib, subprocess, sys, time; "
                "p = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)']); "
                f"pathlib.Path({str(pidfile)!r}).write_text(str(p.pid)); "
                "time.sleep(60)"
            )
            with self.assertRaises(subprocess.TimeoutExpired):
                run([sys.executable, "-c", code], seconds=1)
            pid = int(pidfile.read_text())
            until = time.monotonic() + 3
            while True:
                state = subprocess.run(["ps", "-p", str(pid), "-o", "stat="],
                                       capture_output=True, text=True).stdout.strip()
                if not state or state.startswith("Z"):
                    break
                self.assertLess(time.monotonic(), until, "descendant survived timeout")


if __name__ == "__main__":
    unittest.main()
