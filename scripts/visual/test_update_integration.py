"""Exercise the BBR update router against the UNCHANGED real CLI in a temp repo."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import update
from support import run


@unittest.skipUnless(os.environ.get("BBR_VISUAL_TEST_TOOL"), "real CLI supplied by integration recipe")
class UpdateIntegration(unittest.TestCase):
    def test_two_suite_selection_accepts_only_named_and_rejects_stale_sources(self):
        binary = Path(os.environ["BBR_VISUAL_TEST_TOOL"])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            subprocess.run(["git", "init", "-q", str(root)], check=True, timeout=10)
            (root / "source.rs").write_text("// fixture source\n")
            policy = {"scopes": ["source.rs"], "suffixes": [".rs"]}
            (root / "policy.json").write_text(json.dumps(policy))
            sources = json.loads(run([binary, "sources", "--repo", root, "--policy", "policy.json"],
                                     seconds=10, capture=True).stdout)
            batch = root / "reports/visual/batches/123-456"
            # Tiny valid RGBA PNG encoded with stdlib; no renderer dependency.
            import struct
            import zlib
            def chunk(kind, data):
                return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
            png = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0))
                   + chunk(b"IDAT", zlib.compress(b"\x00\xff\x00\x00\xff")) + chunk(b"IEND", b""))
            cases = ["fixture--light", "fixture--dark"]
            for suite, config in update.SUITES.items():
                (root / config).write_text(
                    f'schema = 1\ncapture = ["unused"]\nbaselines = "{suite}-baselines"\n'
                    'report = "unused-report"\nruns = "unused-runs"\n[source_policy]\n'
                    'scopes = ["source.rs"]\nsuffixes = [".rs"]\n')
                output = batch / suite / "run"
                output.mkdir(parents=True)
                for case in cases:
                    (output / f"{case}.png").write_bytes(png)
                (output / "capture.json").write_text(json.dumps({
                    "schema": 1, "cases": cases, "environment": {"layer": "gpui"},
                    "images": {case: hashlib.sha256(png).hexdigest() for case in cases},
                    "sources": sources, "source_policy": policy,
                }))
                result = run([binary, "compare", "--repo", root, "--config", config,
                              "--run", output, "--report", batch / suite / "report"],
                             seconds=10, check=False)
                self.assertEqual(result.returncode, 1)

            def in_temp(argv, **kwargs):
                return run([*argv, "--repo", root], **kwargs)

            selected = {suite: ["fixture--light"] for suite in update.SUITES}
            with patch("update.ROOT", root), patch("update.run", side_effect=in_temp):
                self.assertEqual(update.update(binary, batch, selected), 0)
                for suite in selected:
                    baseline = root / f"{suite}-baselines"
                    self.assertEqual((baseline / "fixture--light.png").read_bytes(), png)
                    self.assertFalse((baseline / "fixture--dark.png").exists())
                    metadata = json.loads((baseline / "baseline.json").read_text())
                    self.assertEqual(set(metadata["cases"]), {"fixture--light"})
                # Old reports cannot authorize a second mutation after acceptance.
                with self.assertRaisesRegex(ValueError, "baselines changed"):
                    update.update(binary, batch, {"gallery": ["fixture--dark"]})
                for suite, config in update.SUITES.items():
                    run([binary, "compare", "--repo", root, "--config", config,
                         "--run", batch / suite / "run", "--report", batch / suite / "report"],
                        seconds=10, check=False)
                (root / "source.rs").write_text("// changed after capture\n")
                with self.assertRaises(subprocess.CalledProcessError):
                    update.update(binary, batch, {"gallery": ["fixture--dark"]})
                self.assertFalse((root / "gallery-baselines/fixture--dark.png").exists())


if __name__ == "__main__":
    from support import install_signal_handlers, tool_binary
    install_signal_handlers()
    if not os.environ.get("BBR_VISUAL_TEST_TOOL"):
        os.environ["BBR_VISUAL_TEST_TOOL"] = str(tool_binary())
        UpdateIntegration.__unittest_skip__ = False
    unittest.main()
