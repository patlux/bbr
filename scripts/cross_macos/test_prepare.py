"""Portable integrity tests; real upstream generation is tested by verify.py."""

import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from scripts.cross_macos import prepare, verify


class PreparationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.source = self.root / "source"
        self.source.mkdir()
        self.metadata = self.root / "repo/scripts/cross_macos"
        self.metadata.mkdir(parents=True)
        self.relative = "crates/gpui_macos/build.rs"
        self.input = self.source / self.relative
        self.input.parent.mkdir(parents=True)
        self.input.write_bytes(b"original source\n")
        self.manifest = {"revision": prepare.REVISION, "files": {
            self.relative: hashlib.sha256(self.input.read_bytes()).hexdigest()}}
        (self.metadata / "upstream.json").write_text(json.dumps(self.manifest))
        (self.metadata / "probe.lock").write_text("test-lock\n")
        (self.metadata / "gpui-generators.patch").write_text("unused baseline patch\n")
        self.addCleanup(patch.stopall)
        patch.object(prepare, "HERE", self.metadata).start()
        self.git = patch.object(prepare.subprocess, "run", return_value=
                                subprocess.CompletedProcess([], 0, prepare.REVISION + "\n", "")).start()

    def test_exact_input_bytes_and_receipt_are_preserved(self):
        out = self.root / "output"
        receipt = prepare.prepare(self.source, out, baseline=True)
        self.assertEqual((out / self.relative).read_bytes(), self.input.read_bytes())
        self.assertEqual(verify.verify_sources(out), receipt)
        self.assertEqual(receipt["variant"], "baseline")
        self.assertIsNone(receipt["patch_sha256"])
        self.assertEqual(self.git.call_count, 1)

    def test_wrong_revision_writes_nothing(self):
        self.git.return_value.stdout = "0" * 40
        self.assert_refused("expected GPUI-CE")

    def test_modified_input_writes_nothing(self):
        self.input.write_text("modified")
        self.assert_refused("differs from pin")

    def test_missing_input_writes_nothing(self):
        self.input.unlink()
        self.assert_refused("regular non-symlink")

    def test_symlink_input_writes_nothing(self):
        actual = self.root / "actual"
        self.input.rename(actual)
        self.input.symlink_to(actual)
        self.assert_refused("regular non-symlink")

    def test_symlink_parent_is_refused(self):
        actual = self.source / "actual"
        self.input.parent.rename(actual)
        self.input.parent.symlink_to(actual, target_is_directory=True)
        self.assert_refused("regular non-symlink")

    def test_existing_output_is_preserved(self):
        out = self.root / "output"
        out.mkdir()
        marker = out / "preserved"
        marker.write_text("keep")
        with self.assertRaisesRegex(ValueError, "new path"):
            prepare.prepare(self.source, out)
        self.assertEqual(marker.read_text(), "keep")
        self.git.assert_not_called()

    def test_output_inside_upstream_is_refused(self):
        with self.assertRaisesRegex(ValueError, "outside upstream"):
            prepare.prepare(self.source, self.source / "output")
        self.assertFalse((self.source / "output").exists())

    def test_changed_manifest_revision_is_refused(self):
        self.manifest["revision"] = "0" * 40
        (self.metadata / "upstream.json").write_text(json.dumps(self.manifest))
        self.assert_refused("unexpected upstream revision")

    def test_source_mutation_after_preparation_is_detected(self):
        out = self.root / "output"
        prepare.prepare(self.source, out, baseline=True)
        (out / self.relative).write_text("changed after export")
        with self.assertRaisesRegex(ValueError, "changed after preparation"):
            verify.verify_sources(out)

    def test_source_symlink_after_preparation_is_detected(self):
        out = self.root / "output"
        prepare.prepare(self.source, out, baseline=True)
        path = out / self.relative
        path.unlink()
        path.symlink_to(self.input)
        with self.assertRaisesRegex(ValueError, "changed after preparation"):
            verify.verify_sources(out)

    def test_patch_is_checked_before_application(self):
        out = self.root / "output"
        prepare.prepare(self.source, out)
        self.assertEqual(self.git.call_args_list[1].args[0], ["git", "apply", "--check", "-"])
        self.assertEqual(self.git.call_args_list[2].args[0], ["git", "apply", "-"])

    def assert_refused(self, message):
        out = self.root / "output"
        with self.assertRaisesRegex(ValueError, message):
            prepare.prepare(self.source, out)
        self.assertFalse(out.exists())


class ArtifactReceiptTests(unittest.TestCase):
    def test_missing_or_extra_executables_are_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "build.jsonl"
            path.write_text('{"reason":"build-finished","success":true}\n')
            with self.assertRaisesRegex(ValueError, "exactly these"):
                verify.artifact_paths(path)

    def test_cargo_json_receipt_selects_built_executables(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "build.jsonl"
            path.write_text("\n".join(json.dumps({
                "reason": "compiler-artifact", "target": {"name": name},
                "executable": "/tmp/" + name,
            }) for name in ("shader-generator", "media-generator")))
            self.assertEqual(set(verify.artifact_paths(path)),
                             {"shader-generator", "media-generator"})


if __name__ == "__main__":
    unittest.main()
