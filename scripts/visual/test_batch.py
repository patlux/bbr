"""Combined snapshot orchestration preserves the upstream acceptance boundary."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import batch
import gallery_capture
from gallery_environment import environment
import update


class BatchContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.batches = self.root / "reports/visual/batches"
        self.batch = self.batches / "123-456"

    def test_selection_requires_explicit_batch_suite_and_unique_cases(self):
        with patch("batch.ROOT", self.root), patch("batch.BATCHES", self.batches):
            self.assertEqual(update.selection(["123-456", "browser:ordinary-full--light"]),
                             (self.batch, {"browser": ["ordinary-full--light"]}))
            for args in ([], ["123-456"], ["latest", "browser:a--light"],
                         ["123-456", "*"], ["123-456", "gallery:*"],
                         ["123-456", "other:a--light"], ["123-456", "browser:../a--light"],
                         ["123-456", "browser:a--light", "browser:a--light"],
                         ["../123-456", "browser:a--light"]):
                with self.subTest(args=args), self.assertRaises(ValueError):
                    update.selection(args)

    def test_gallery_renames_identity_not_png_content_and_requires_pairs(self):
        output = self.root / "capture"
        output.mkdir()
        old = "gallery-00-state-00-standard-normal"
        for mode in ("light", "dark"):
            (output / f"{old}--{mode}-page-00.png").write_bytes(mode.encode())
        ids = gallery_capture.prepare_inventory(output)
        self.assertEqual(ids, [f"{old}-page-00--dark", f"{old}-page-00--light"])
        for mode in ("light", "dark"):
            self.assertEqual((output / f"{old}-page-00--{mode}.png").read_bytes(), mode.encode())
        orphan = self.root / "orphan"
        orphan.mkdir()
        png = orphan / f"{old}--light-page-00.png"
        png.write_bytes(b"unchanged")
        with self.assertRaises(ValueError):
            gallery_capture.prepare_inventory(orphan)
        self.assertEqual(png.read_bytes(), b"unchanged")
        self.assertFalse((orphan / "inventory.json").exists())

    def test_gallery_raw_output_is_fresh_confined_and_not_symlinked(self):
        with patch("gallery_capture.ROOT", self.root):
            path = self.batches / "123-456/gallery/run"
            self.assertEqual(gallery_capture.output_directory(str(path)), path)
            path.mkdir()
            for invalid in (path, self.root / "elsewhere", Path("relative"), path / "../escape"):
                with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                    gallery_capture.output_directory(str(invalid))
            link = self.batches / "linked"
            link.symlink_to(self.root, target_is_directory=True)
            with self.assertRaises(ValueError):
                gallery_capture.output_directory(str(link / "run"))

    def test_gallery_environment_is_real_offscreen_and_explicit_about_limits(self):
        with patch("gallery_environment.host_environment", return_value={"layer": "gpui"}):
            value = environment()
        self.assertFalse(value["native_window"])
        self.assertIn("MetalHeadlessRenderer", value["renderer"])
        self.assertEqual(value["scale"], 2)
        self.assertIn("pending", value["coverage_limits"])

    def test_all_suites_run_on_review_failure_or_invalid_capture(self):
        for gallery_exit in (1, 2):
            base = self.batches / str(gallery_exit)
            calls = []

            def invoke(argv, **_kwargs):
                suite = "gallery" if "visual-gallery.toml" in argv else "browser"
                calls.append(suite)
                report = Path(argv[argv.index("--report") + 1])
                report.mkdir(parents=True)
                (report / "index.html").write_text("upstream report")
                (report / "review.json").write_text(json.dumps({
                    "results": [{"status": "missing-baseline"}], "errors": [],
                    "environment_mismatch": None,
                }))
                return subprocess.CompletedProcess(argv, gallery_exit if suite == "gallery" else 0, "log")

            with patch("batch.BATCHES", base), patch("batch.run", side_effect=invoke), \
                 patch("batch.tool_binary", return_value=Path("comparator")):
                self.assertEqual(batch.check(Path("comparator")), gallery_exit)
            self.assertEqual(calls, ["gallery", "browser"])
            created = next(base.iterdir())
            self.assertIn("browser/report/index.html", (created / "index.html").read_text())
            self.assertEqual(set(json.loads((created / "batch.json").read_text())["suites"]),
                             {"gallery", "browser"})

    def test_failed_update_stops_without_retrying_or_rolling_back(self):
        with patch("update.preflight"), patch("update.run", return_value=
                subprocess.CompletedProcess([], 2)) as invoke:
            self.assertEqual(update.update(Path("cli"), self.batch,
                                           {"gallery": ["g--light"], "browser": ["b--light"]}), 2)
            self.assertEqual(invoke.call_count, 1)

    def test_real_configs_keep_baseline_namespaces_and_source_seals_separate(self):
        import tomllib
        from support import ROOT
        configs = [tomllib.loads((ROOT / name).read_text()) for name in batch.SUITES.values()]
        self.assertEqual(len({config["baselines"] for config in configs}), 2)
        for config in configs:
            for name in batch.SUITES.values():
                self.assertIn(name, config["source_policy"]["scopes"])
            self.assertIn("scripts/visual", config["source_policy"]["scopes"])
            self.assertIn("crates", config["source_policy"]["scopes"])
            self.assertTrue(config["environment_command"])

    def test_invalid_later_suite_prevents_all_updates(self):
        with patch("update.preflight", side_effect=[None, ValueError("stale")]), \
             patch("update.run") as invoke:
            with self.assertRaises(ValueError):
                update.update(Path("cli"), self.batch,
                              {"gallery": ["g--light"], "browser": ["b--light"]})
            invoke.assert_not_called()

    def test_update_routes_only_explicit_cases_without_capture_or_adoption(self):
        with patch("update.preflight") as verify, patch("update.run", return_value=
                subprocess.CompletedProcess([], 0)) as invoke:
            self.assertEqual(update.update(Path("cli"), self.batch,
                                           {"browser": ["b--light", "b--dark"]}), 0)
            verify.assert_called_once()
            argv = invoke.call_args.args[0]
            self.assertEqual(argv[1], "accept")
            self.assertEqual(argv[-4:], ["--case", "b--light", "--case", "b--dark"])
            self.assertNotIn("--adopt-environment", argv)

    def test_preflight_rejects_changed_baselines_capture_invalid_review_and_environment(self):
        suite = self.batch / "browser"
        (suite / "run").mkdir(parents=True)
        (suite / "report").mkdir()
        (self.root / "visual.toml").write_text('baselines = "baselines"\n')
        capture = suite / "run/capture.json"
        capture.write_text(json.dumps({"cases": ["b--light"], "environment": {"layer": "gpui"}}))
        valid = {"schema": 1, "capture_hash": hashlib.sha256(capture.read_bytes()).hexdigest(),
                 "baseline_state": {}, "environment": {"layer": "gpui"}, "errors": [],
                 "environment_mismatch": None, "results": [{"case": "b--light", "status": "changed"}]}
        for change in ({"capture_hash": "bad"}, {"baseline_state": {"unknown.png": "bad"}},
                       {"errors": ["invalid"]}, {"environment_mismatch": {}},
                       {"results": [{"case": "b--light", "status": "same"}]}):
            (suite / "report/review.json").write_text(json.dumps({**valid, **change}))
            with patch("update.ROOT", self.root), patch("update.run") as invoke:
                with self.subTest(change=change), self.assertRaises(ValueError):
                    update.preflight(Path("cli"), self.batch, "browser", ["b--light"])
                invoke.assert_not_called()
        (suite / "report/review.json").write_text(json.dumps(valid))
        with patch("update.ROOT", self.root), patch("update.run") as invoke:
            update.preflight(Path("cli"), self.batch, "browser", ["b--light"])
            self.assertEqual(invoke.call_args.args[0][1], "validate")


if __name__ == "__main__":
    unittest.main()
