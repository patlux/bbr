"""Guest composition with real file transport and synthetic producer bytes.

No native binary or actual environment probe is executed. These validate ordering
and evidence binding, not Metal pixels, real guest bootstrap or GUI correctness.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_guest
import vm_install
import vm_transfer


class GuestContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.share, self.repo, self.work = (self.root / name for name in ("share", "repo", "work"))
        self.share.mkdir()
        self.environment = {"schema": 1, "origin": "tart-guest", "os": "fixture", "os_build": "fixture",
                            "arch": "arm64", "model": "VirtualMac-fixture", "displays": [{"synthetic": True}],
                            "fonts": {"/System/Library/Fonts/fixture.ttf": "f" * 64}, "console_user": "admin",
                            "on_console": True, "screen_locked": False, "window_server": True,
                            "timezone": "UTC", "locale": "C"}
        self.calls = []

    def payload(self, suite):
        files = {"source/source.rs": ("source.rs", b"synthetic source", False),
                 "executables/0": ("scripts/visual/target/release/bbr-visual", b"synthetic consumer", True)}
        if suite == "gallery":
            files["executables/1"] = ("target/release/bbr-gallery", b"synthetic gallery", True)
        else:
            files["executables/1"] = ("target/release/bbr", b"synthetic app", True)
            files["executables/2"] = ("target/release/deps/e2e_visual_shell_capture-fixture", b"synthetic test", True)
        payload = {"schema": 1, "suite": suite, "repo": str(self.repo), "sources": {}, "files": []}
        for name, (guest, data, executable) in files.items():
            path = self.share / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            path.chmod(0o500 if executable else 0o400)
            digest = hashlib.sha256(data).hexdigest()
            payload["files"].append({"payload": name, "guest": str(self.repo / guest),
                                     "bytes": len(data), "sha256": digest, "executable": executable})
            if not executable: payload["sources"][guest] = digest
        raw = json.dumps(payload).encode()
        (self.share / "payload.json").write_bytes(raw)
        return hashlib.sha256(raw).hexdigest()

    def producer(self, consumer, suite, action, repo, share, digest, work, seconds):
        self.calls.append(action)
        self.assertEqual(consumer, repo / "scripts/visual/target/release/bbr-visual")
        self.assertTrue(vm_install.verify_installed(share, digest, repo, suite))
        if action == "inventory":
            (work / "live-inventory.tsv").write_text("synthetic live inventory\n")
        else:
            raw = work / "raw"
            raw.mkdir()
            for mode in ("light", "dark"):
                (raw / f"fixture--{mode}.png").write_bytes(b"synthetic opaque transfer fixture, NOT PNG")
            if suite == "gallery": (raw / "coverage.tsv").write_text("synthetic coverage\n")

    def test_gallery_install_observe_execute_receipt_return_is_one_bound_chain(self):
        digest = self.payload("gallery")
        with patch.object(vm_guest.vm_observe, "observe", return_value=self.environment), \
             patch.object(vm_guest, "_producer", self.producer):
            result = vm_guest.run(self.share, digest, self.repo, "gallery", self.work)
        self.assertEqual(self.calls, ["inventory", "capture"])
        self.assertEqual(result["payload_sha256"], digest)
        returned = self.root / "returned"
        vm_transfer.unpack(self.work / "return.tar", returned, digest)
        receipt = json.loads((returned / "guest-receipt.json").read_text())
        self.assertEqual(receipt["installed"], vm_install.verify_installed(self.share, digest, self.repo, "gallery"))
        self.assertEqual(receipt["environment_before"], self.environment)
        self.assertEqual(receipt["environment_before"], receipt["environment_after"])
        self.assertFalse((returned / "capture.json").exists())
        self.assertTrue((returned / "live-inventory.tsv").exists())

    def test_browser_uses_fixed_capture_only_and_preserves_transport_bytes(self):
        digest = self.payload("browser")
        with patch.object(vm_guest.vm_observe, "observe", return_value=self.environment), \
             patch.object(vm_guest, "_producer", self.producer):
            vm_guest.run(self.share, digest, self.repo, "browser", self.work)
        self.assertEqual(self.calls, ["capture"])
        vm_transfer.inspect(self.work / "return.tar", digest)

    def test_environment_drift_prevents_receipt_and_return(self):
        digest = self.payload("gallery")
        after = {**self.environment, "os_build": "changed"}
        with patch.object(vm_guest.vm_observe, "observe", side_effect=[self.environment, after]), \
             patch.object(vm_guest, "_producer", self.producer):
            with self.assertRaisesRegex(vm_guest.GuestError, "changed"):
                vm_guest.run(self.share, digest, self.repo, "gallery", self.work)
        self.assertFalse((self.work / "raw/guest-receipt.json").exists())
        self.assertFalse((self.work / "return.tar").exists())

    def test_installed_mutation_prevents_receipt_and_return(self):
        digest = self.payload("browser")
        def mutate(*args):
            self.producer(*args)
            path = self.repo / "source.rs"
            path.chmod(0o600)
            path.write_text("mutated")
            path.chmod(0o400)
        with patch.object(vm_guest.vm_observe, "observe", return_value=self.environment), \
             patch.object(vm_guest, "_producer", mutate):
            with self.assertRaises(vm_install.InstallError):
                vm_guest.run(self.share, digest, self.repo, "browser", self.work)
        self.assertFalse((self.work / "raw/guest-receipt.json").exists())
        self.assertFalse((self.work / "return.tar").exists())

    def test_host_observation_and_unknown_payload_refused_before_installation(self):
        digest = self.payload("gallery")
        for mutation in ({"model": "Mac16,9"}, {"console_user": "patwoz"}, {"screen_locked": True}):
            with patch.object(vm_guest.vm_observe, "observe", return_value={**self.environment, **mutation}):
                with self.assertRaises(vm_guest.GuestError):
                    vm_guest.run(self.share, digest, self.repo, "gallery", self.work)
            self.assertFalse(self.repo.exists())
        with patch.object(vm_guest.vm_observe, "observe", return_value=self.environment):
            with self.assertRaises(vm_install.InstallError):
                vm_guest.run(self.share, "f" * 64, self.repo, "gallery", self.work)
        self.assertFalse(self.work.exists())

    def test_producer_failure_never_observes_after_or_packs(self):
        digest = self.payload("gallery")
        with patch.object(vm_guest.vm_observe, "observe", return_value=self.environment) as observe, \
             patch.object(vm_guest, "_producer", side_effect=vm_guest.GuestError("injected producer failure")):
            with self.assertRaisesRegex(vm_guest.GuestError, "producer failure"):
                vm_guest.run(self.share, digest, self.repo, "gallery", self.work)
        observe.assert_called_once()
        self.assertFalse((self.work / "return.tar").exists())

    def test_changed_shared_manifest_prevents_receipt_and_return(self):
        digest = self.payload("browser")
        def mutate(*args):
            self.producer(*args)
            (self.share / "payload.json").write_text("{}")
        with patch.object(vm_guest.vm_observe, "observe", return_value=self.environment), \
             patch.object(vm_guest, "_producer", mutate):
            with self.assertRaises(vm_install.InstallError):
                vm_guest.run(self.share, digest, self.repo, "browser", self.work)
        self.assertFalse((self.work / "raw/guest-receipt.json").exists())
        self.assertFalse((self.work / "return.tar").exists())

    def test_overlap_or_reused_output_refused_without_installation(self):
        digest = self.payload("gallery")
        self.work.mkdir()
        for work in (self.work, self.share / "output", self.repo / "output"):
            with patch.object(vm_guest.vm_observe, "observe", return_value=self.environment), \
                 patch.object(vm_guest, "_producer") as producer:
                with self.assertRaises(vm_guest.GuestError):
                    vm_guest.run(self.share, digest, self.repo, "gallery", work)
            producer.assert_not_called()
            self.assertFalse(self.repo.exists())

    def test_cooperative_consumer_timeout_reaps_without_hard_kill(self):
        from unittest.mock import Mock
        self.work.mkdir()
        child = Mock(returncode=None)
        child.poll.return_value = None
        child.wait.return_value = 2
        with patch.object(vm_guest.subprocess, "Popen", return_value=child):
            with self.assertRaisesRegex(vm_guest.GuestError, "deadline"):
                vm_guest._producer(self.root / "fixture", "gallery", "capture", self.repo, self.share,
                                   "a" * 64, self.work, 0)
        child.terminate.assert_called_once()
        child.wait.assert_called_once_with(timeout=vm_guest.CONSUMER_GRACE)
        child.kill.assert_not_called()

    def test_direct_consumer_timeout_gets_term_before_hard_kill(self):
        from unittest.mock import Mock
        self.work.mkdir()
        child = Mock(returncode=None)
        child.poll.return_value = None
        child.wait.side_effect = [subprocess.TimeoutExpired("fixture", 15), 0]
        with patch.object(vm_guest.subprocess, "Popen", return_value=child) as spawn:
            with self.assertRaisesRegex(vm_guest.GuestError, "hard terminated"):
                vm_guest._producer(self.root / "fixture", "gallery", "capture", self.repo, self.share,
                                   "a" * 64, self.work, 0)
        child.terminate.assert_called_once()
        child.kill.assert_called_once()
        self.assertEqual(child.wait.call_count, 2)
        self.assertNotIn("start_new_session", spawn.call_args.kwargs)
        self.assertEqual(spawn.call_args.kwargs["env"]["HOME"], "/Users/admin")


if __name__ == "__main__":
    unittest.main()
