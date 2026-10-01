"""Bound cleanup records with mocked resources; never VM/network/key access."""
from pathlib import Path
import hashlib
import json
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_cleanup
import vm_evidence
import vm_install
import vm_lifecycle


class EvidenceContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.artifacts = self.root / "diagnostics"
        self.artifacts.mkdir(mode=0o700)
        self.binding = vm_evidence.Binding(self.artifacts, "bbr-visual-123-abcdef123456", "a" * 64, "b" * 64)
        self.clone = Mock(vm=self.binding.vm, artifacts=self.artifacts, keep_vm=False)
        self.clone.tart_process.pid = 1234
        self.observed = {"schema": 1, "vm": self.binding.vm, "ip": "192.168.64.9",
                         "tart_pid": 1234, "tart_exit": 0, "clone_absent": True,
                         "disk_detached": True, "private_key_absent": True,
                         "owned_process_reaped": True, "ssh_unreachable": True}

    def records(self):
        vm_evidence.begin(self.binding)
        vm_evidence.resources(self.binding, self.clone, self.observed["ip"])
        vm_evidence.complete(self.binding, self.observed)

    def test_complete_records_still_require_independent_observation(self):
        self.records()
        with patch.object(vm_cleanup, "recheck") as recheck:
            value = vm_evidence.verify(self.binding)
        recheck.assert_called_once_with(self.binding.vm, self.artifacts, "192.168.64.9", 1234)
        self.assertEqual(value["request_sha256"], "a" * 64)
        with patch.object(vm_cleanup, "recheck", side_effect=vm_cleanup.CleanupError("retained clone")):
            with self.assertRaisesRegex(vm_cleanup.CleanupError, "retained clone"):
                vm_evidence.verify(self.binding)

    def test_missing_or_swapped_records_fail_before_observation(self):
        self.records()
        for name in ("owned-run.json", "owned-resources.json", "cleanup-observed.json"):
            path = self.artifacts / name
            saved = path.read_bytes()
            mutations = [("request_sha256", "c" * 64), ("payload_sha256", "d" * 64),
                         ("vm", "bbr-visual-9-abcdef123456"), ("schema", True), ("extra", 0)]
            if name != "owned-run.json": mutations.extend([("ip", 1234), ("ip", " 192.168.64.9 ")])
            for field, value in mutations:
                path.write_text(json.dumps({**json.loads(saved), field: value}))
                with patch.object(vm_cleanup, "recheck") as recheck:
                    with self.assertRaises((vm_evidence.EvidenceError, vm_install.InstallError, KeyError)):
                        vm_evidence.verify(self.binding)
                recheck.assert_not_called()
            path.unlink()
            with patch.object(vm_cleanup, "recheck") as recheck:
                with self.assertRaises(FileNotFoundError): vm_evidence.verify(self.binding)
            recheck.assert_not_called()
            path.write_bytes(saved)

    def test_missing_resources_failed_flags_wrong_pid_and_overwrite_cannot_complete(self):
        vm_evidence.begin(self.binding)
        with self.assertRaises(FileNotFoundError): vm_evidence.complete(self.binding, self.observed)
        vm_evidence.resources(self.binding, self.clone, "192.168.64.9")
        for mutation in ({"ssh_unreachable": False}, {"tart_pid": 999}, {"tart_exit": True}, {"tart_exit": 999},
                         {"ip": "192.168.64.10"}, {"owned_process_reaped": 1}):
            with self.assertRaises(vm_evidence.EvidenceError):
                vm_evidence.complete(self.binding, {**self.observed, **mutation})
        self.assertFalse((self.artifacts / "cleanup-observed.json").exists())
        vm_evidence.complete(self.binding, self.observed)
        with self.assertRaises(FileExistsError): vm_evidence.complete(self.binding, self.observed)

    def test_changed_during_observation_links_and_oversized_evidence_fail(self):
        self.records()
        path = self.artifacts / "cleanup-observed.json"
        saved = path.read_bytes()
        def mutate(*_): path.write_text(json.dumps({**json.loads(saved), "tart_exit": 2}))
        with patch.object(vm_cleanup, "recheck", mutate):
            with self.assertRaisesRegex(vm_evidence.EvidenceError, "changed"):
                vm_evidence.verify(self.binding)
        path.write_bytes(saved)
        import os
        alias = self.root / "alias"
        os.link(path, alias)
        with self.assertRaises(vm_install.InstallError): vm_evidence.verify(self.binding)
        alias.unlink()
        path.write_bytes(b" " * 16385)
        with self.assertRaises(vm_install.InstallError): vm_evidence.verify(self.binding)

    def test_bound_lifecycle_records_before_producer_and_completes_after_cleanup(self):
        payload = self.root / "payload"
        payload.mkdir()
        (payload / "payload.json").write_text("{}")
        binding = vm_evidence.Binding(self.artifacts, self.binding.vm, "a" * 64,
                                      hashlib.sha256(b"{}").hexdigest())
        def boot(_source, name, artifacts, *_args, **_kwargs):
            self.assertEqual(name, binding.vm)
            self.assertTrue((artifacts / "owned-run.json").exists())
            return self.clone, ["ssh", "admin@192.168.64.9"], "synthetic session"
        with patch.object(vm_lifecycle.tart_vm, "boot", boot), patch.object(vm_lifecycle, "audit"), \
             patch.object(vm_cleanup, "observe", return_value=self.observed):
            with vm_lifecycle.session(payload, self.artifacts, binding=binding):
                self.assertTrue((self.artifacts / "owned-resources.json").exists())
                self.assertFalse((self.artifacts / "cleanup-observed.json").exists())
                self.clone.cleanup.assert_not_called()
        self.clone.cleanup.assert_called_once()
        with patch.object(vm_cleanup, "recheck"):
            self.assertEqual(vm_evidence.verify(binding)["payload_sha256"], binding.payload_sha256)

    def test_wrong_payload_and_partial_boot_never_claim_complete_cleanup(self):
        payload = self.root / "payload"
        payload.mkdir()
        (payload / "payload.json").write_text("{}")
        with patch.object(vm_lifecycle.tart_vm, "boot") as boot:
            with self.assertRaisesRegex(vm_lifecycle.tart_vm.RunnerError, "payload differs"):
                with vm_lifecycle.session(payload, self.artifacts, binding=self.binding): pass
        boot.assert_not_called()
        binding = vm_evidence.Binding(self.artifacts, self.binding.vm, "a" * 64,
                                      hashlib.sha256(b"{}").hexdigest())
        with patch.object(vm_lifecycle.tart_vm, "boot", side_effect=RuntimeError("partial boot")):
            with self.assertRaisesRegex(RuntimeError, "partial boot"):
                with vm_lifecycle.session(payload, self.artifacts, binding=binding): pass
        self.assertTrue((self.artifacts / "owned-run.json").exists())
        with patch.object(vm_cleanup, "recheck") as recheck:
            with self.assertRaises(FileNotFoundError): vm_evidence.verify(binding)
        recheck.assert_not_called()


if __name__ == "__main__": unittest.main()
