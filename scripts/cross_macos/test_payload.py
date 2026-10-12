"""Exact-artifact transfer/VM plan contracts; never boots a guest."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from scripts.cross_macos import apple_sdk, payload, verify_vm


class TransferTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.source = self.root / "snapshot"
        (self.source / "bbr").mkdir(parents=True)
        (self.source / "bbr/Cargo.toml").write_text("fixture")
        prepared = {"source": {"bbr_revision": "a" * 40},
                    "inputs": {"bbr/Cargo.toml": apple_sdk.digest(self.source / "bbr/Cargo.toml")}}
        (self.source / "prepared.json").write_text(json.dumps(prepared))
        self.build = self.root / "build"; self.build.mkdir()
        self.relative = "target/aarch64-apple-darwin/debug/deps/e2e_smoke-abcdef"
        self.exe = self.source / "bbr" / self.relative
        self.exe.parent.mkdir(parents=True); self.exe.write_bytes(b"inert test fixture"); self.exe.chmod(0o755)
        self.receipt = {"target": "aarch64-apple-darwin",
            "prepared_sha256": apple_sdk.digest(self.source / "prepared.json"), "bundle": {},
            "executables": {self.relative: {"sha256": apple_sdk.digest(self.exe), "test": True, "target": "e2e_smoke"}}}
        (self.build / "build-receipt.json").write_text(json.dumps(self.receipt))

    def staged(self):
        dest = self.root / "payload"
        payload.stage(self.source, [self.build], dest)
        # Fixture-only portable compiled path. No program in this test executes.
        p = dest / "payload.json"; data = json.loads(p.read_text())
        data["repo"] = "/tmp/bbr-cross-fixture/bbr"; p.write_text(json.dumps(data))
        return dest, apple_sdk.digest(p)

    def test_roundtrip_hashes_and_guest_host_guard(self):
        dest, sha = self.staged()
        manifest = verify_vm.inspect(dest, sha)
        self.assertEqual(manifest["tests"], {"e2e_smoke": self.relative})
        script = verify_vm.bootstrap(manifest)
        self.assertLess(script.index("VirtualMac"), script.index("mount_virtiofs"))
        self.assertNotIn("otool", script)
        self.assertNotIn("xcrun", script)
        self.assertIn("/usr/sbin/ioreg", script)
        self.assertIn("shasum", script)
        self.assertNotIn("grep -q", script)
        with self.assertRaisesRegex(ValueError, "independent expectation"):
            verify_vm.inspect(dest, "0" * 64)

    def test_source_and_artifact_mutation_precede_output(self):
        self.exe.write_bytes(b"modified")
        dest = self.root / "payload"
        with self.assertRaisesRegex(ValueError, "stale"):
            payload.stage(self.source, [self.build], dest)
        self.assertFalse(dest.exists())

    def test_archive_and_checksum_list_drift_refused(self):
        dest, sha = self.staged()
        checksum = dest / "SHA256SUMS"; original = checksum.read_text()
        checksum.write_text(original + "injected\n")
        with self.assertRaisesRegex(ValueError, "checksum list"):
            verify_vm.inspect(dest, sha)
        checksum.write_text(original)
        archive = dest / "payload.tar.gz"; archive.write_bytes(archive.read_bytes() + b"changed")
        with self.assertRaisesRegex(ValueError, "archive hash"):
            verify_vm.inspect(dest, sha)

    def test_compiled_path_and_test_binding_refused(self):
        dest, sha = self.staged()
        p = dest / "payload.json"; original = p.read_text()
        for repo in ("/Users/patwoz/dev/bbr", "/tmp/bbr-cross-x/../bbr", "/tmp/bbr-cross-x/bbr;echo"):
            m = json.loads(original); m["repo"] = repo; p.write_text(json.dumps(m))
            with self.assertRaisesRegex(ValueError, "compiled source"):
                verify_vm.inspect(dest, apple_sdk.digest(p))
        m = json.loads(original); m["tests"]["e2e_smoke"] = "Cargo.toml"; p.write_text(json.dumps(m))
        with self.assertRaisesRegex(ValueError, "executable binding"):
            verify_vm.inspect(dest, apple_sdk.digest(p))

    def test_zero_ignored_failed_or_ambiguous_execution_refused(self):
        valid = 'test result: ok. 2 passed; 0 failed; 0 ignored; 1 filtered out;'
        self.assertEqual(verify_vm.validate_test_output(valid), 2)
        for invalid in ('', valid.replace('2 passed', '0 passed'), valid + valid,
                        valid.replace('0 ignored', '2 ignored'), valid.replace('0 failed', '1 failed')):
            with self.assertRaises(ValueError):
                verify_vm.validate_test_output(invalid)

    def test_recipe_plan_preserves_order_and_deferral(self):
        plan = verify_vm.plan_for([], True)
        self.assertEqual(plan[0][1], "e2e_smoke")
        self.assertEqual(plan[-1][1], "bbr-gallery")
        self.assertEqual(len(plan), 59)
        self.assertTrue(all("perf" not in p[0] for p in plan))
        for names, functional in (([], False), (["e2e-smoke"], True), (["e2e-perf"], False)):
            with self.assertRaises(ValueError):
                verify_vm.plan_for(names, functional)
        self.assertEqual(verify_vm.plan_for(["e2e-smoke"], False)[0][2], ["--ignored", "--nocapture"])


if __name__ == "__main__":
    unittest.main()
