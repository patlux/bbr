"""Local transport/cleanup composition; no SSH, Tart, keys or native producer."""
from contextlib import contextmanager
import hashlib
import json
from pathlib import Path
import sys
import unittest
from types import SimpleNamespace
from unittest.mock import patch, Mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import test_vm_guest
import vm_cleanup
import vm_control
import vm_evidence
import vm_lifecycle
import vm_transfer
import vm_transport


class TransportContracts(unittest.TestCase):
    def setUp(self):
        fixture = test_vm_guest.GuestContracts()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        self.fixture = fixture
        self.digest = fixture.payload("gallery")
        self.workspace = fixture.root / "transport"
        self.workspace.mkdir(mode=0o700)
        self.control = self.workspace / "control"
        self.control.mkdir(mode=0o700)
        (self.workspace / "vm").mkdir(mode=0o700)
        request = {"schema": 1, "control": str(self.control), "payload_sha256": self.digest,
                   "operation_ms": 10000, "grace_ms": 10000}
        raw = json.dumps(request).encode()
        (self.control / "request.json").write_bytes(raw)
        self.request = hashlib.sha256(raw).hexdigest()
        self.vm = "bbr-visual-123-abcdef012345"
        self.calls = []

    @contextmanager
    def lifecycle(self, share, artifacts, *, binding):
        self.calls.append("boot-fixture")
        vm_evidence.begin(binding)
        clone = SimpleNamespace(vm=binding.vm, artifacts=artifacts, keep_vm=False,
                                tart_process=SimpleNamespace(pid=12345))
        vm_evidence.resources(binding, clone, "192.168.64.2")
        try:
            yield SimpleNamespace(clone=clone)
        finally:
            self.calls.append("cleanup-fixture")
            vm_evidence.complete(binding, {"schema": 1, "vm": binding.vm, "ip": "192.168.64.2",
                "tart_pid": 12345, "tart_exit": 0, "clone_absent": True, "disk_detached": True,
                "private_key_absent": True, "owned_process_reaped": True, "ssh_unreachable": True})

    def exchange(self, session, script, workspace):
        self.calls.append("exchange-fixture")
        self.assertFalse((self.control / "terminal.json").exists())
        self.assertFalse((workspace / "raw").exists())
        raw = self.fixture.root / "guest-raw-fixture"
        raw.mkdir()
        (raw / "guest-receipt.json").write_text('{"synthetic":true}')
        (raw / "fixture--light.png").write_bytes(b"opaque local fixture, NOT PNG")
        vm_transfer.pack(raw, workspace / "return.tar", self.digest)

    def recheck(self, *args):
        self.calls.append("observe-fixture")
        self.assertIn("cleanup-fixture", self.calls)
        self.assertFalse((self.control / "terminal.json").exists())

    def execute(self, *, exchange=None, recheck=None):
        with patch.object(vm_transport.vm_bootstrap, "plan", return_value="synthetic command, never executed"), \
             patch.object(vm_transport.vm_lifecycle, "session", self.lifecycle), \
             patch.object(vm_transport, "_exchange", exchange or self.exchange), \
             patch.object(vm_cleanup, "recheck", recheck or self.recheck):
            return vm_transport.run(self.fixture.share, self.digest, self.fixture.repo, "gallery",
                                    self.workspace, self.vm, self.control, self.request)

    def terminal(self):
        return json.loads((self.control / "terminal.json").read_text())["outcome"]

    def test_success_requires_exchange_archive_checks_cleanup_and_observation_before_raw(self):
        unpack = vm_transfer.unpack
        def checked(*args):
            self.assertEqual(self.calls, ["boot-fixture", "exchange-fixture", "cleanup-fixture", "observe-fixture"])
            self.assertFalse((self.control / "terminal.json").exists())
            return unpack(*args)
        with patch.object(vm_transfer, "unpack", checked):
            value = self.execute()
        self.assertEqual(self.terminal(), "completed")
        self.assertEqual(value["payload_sha256"], self.digest)
        self.assertEqual(len(value["files"]), 2)
        self.assertEqual(self.calls[-1], "observe-fixture")
        self.assertEqual((self.workspace / "raw/fixture--light.png").read_bytes(), b"opaque local fixture, NOT PNG")
        self.assertFalse((self.workspace / "raw/capture.json").exists())

    def test_exchange_failure_and_cancellation_cleanup_before_terminal_without_raw(self):
        def failed(*args): raise vm_transport.TransportError("injected SSH failure")
        with self.assertRaisesRegex(vm_transport.TransportError, "injected"):
            self.execute(exchange=failed)
        self.assertEqual(self.terminal(), "failed")
        self.assertEqual(self.calls, ["boot-fixture", "cleanup-fixture", "observe-fixture"])
        self.assertFalse((self.workspace / "raw").exists())

    def test_cancelled_exchange_is_acknowledged_only_after_observed_cleanup(self):
        def cancelled(*args): raise vm_lifecycle.Cancelled("fixture cancellation")
        with self.assertRaises(vm_lifecycle.Cancelled): self.execute(exchange=cancelled)
        self.assertEqual(self.terminal(), "cancelled")
        self.assertEqual(self.calls, ["boot-fixture", "cleanup-fixture", "observe-fixture"])
        self.assertFalse((self.workspace / "raw").exists())

    def test_truncated_archive_cleans_vm_but_cannot_publish_candidates(self):
        def broken(*args):
            self.exchange(*args)
            (self.workspace / "return.tar").write_bytes(b"truncated")
        with self.assertRaises(Exception): self.execute(exchange=broken)
        self.assertEqual(self.terminal(), "failed")
        self.assertIn("cleanup-fixture", self.calls)
        self.assertFalse((self.workspace / "raw").exists())

    def test_archive_bound_to_another_payload_is_rejected_before_raw(self):
        def swapped(*args):
            self.exchange(*args)
            archive = self.workspace / "return.tar"
            archive.unlink()
            vm_transfer.pack(self.fixture.root / "guest-raw-fixture", archive, "f" * 64)
        with self.assertRaises(vm_transfer.TransferError): self.execute(exchange=swapped)
        self.assertEqual(self.terminal(), "failed")
        self.assertFalse((self.workspace / "raw").exists())

    def test_payload_drift_after_exchange_prevents_candidate_publication(self):
        def changed(*args):
            self.exchange(*args)
            (self.fixture.share / "payload.json").write_text("{}")
        with self.assertRaises(Exception): self.execute(exchange=changed)
        self.assertEqual(self.terminal(), "failed")
        self.assertFalse((self.workspace / "raw").exists())

    def test_resource_verification_failure_retains_unacknowledged_control_and_archive(self):
        def failed(*args): raise vm_evidence.EvidenceError("injected retained resource")
        with self.assertRaisesRegex(vm_evidence.EvidenceError, "retained"):
            self.execute(recheck=failed)
        self.assertFalse((self.control / "terminal.json").exists())
        self.assertTrue((self.workspace / "return.tar").exists())
        self.assertFalse((self.workspace / "raw").exists())

    def test_swapped_request_fails_before_any_boot_or_exchange(self):
        (self.control / "request.json").write_text("{}")
        with self.assertRaisesRegex(vm_transport.TransportError, "request"):
            self.execute()
        self.assertEqual(self.calls, [])
        self.assertFalse((self.control / "terminal.json").exists())

    def test_only_exact_isolated_session_command_and_fixed_archive_are_used(self):
        clone = SimpleNamespace(artifacts=self.workspace / "vm")
        session = SimpleNamespace(clone=clone, ip="192.168.64.2")
        session.ssh = vm_lifecycle.tart_vm.ssh_args(clone.artifacts / "runner_ed25519",
                                                   clone.artifacts / "known_hosts", session.ip, isolated=True)
        with patch.object(vm_transport, "_execute") as execute:
            vm_transport._exchange(session, "fixture", self.workspace)
        first, second = execute.call_args_list
        self.assertEqual(first.args[0][0], "/usr/bin/ssh")
        self.assertEqual(first.args[0][-3:], ["/bin/zsh", "-f", "-s"])
        self.assertEqual(second.args[0][-2:], ["/bin/cat", "/Users/admin/bbr-visual-work/return.tar"])
        self.assertIsNone(second.kwargs["script"])
        session.ssh.insert(1, "-A")
        with self.assertRaisesRegex(vm_transport.TransportError, "isolated"):
            vm_transport._ssh(session)

    def test_bounded_direct_local_stream_and_timeout_no_detached_child(self):
        out, err = self.workspace / "out", self.workspace / "err"
        vm_transport._execute(["/bin/cat"], out, err, script="fixture input", seconds=2, output_limit=128)
        self.assertEqual(out.read_text(), "fixture input")
        self.assertEqual(out.stat().st_mode & 0o777, 0o600)
        child = Mock(returncode=None)
        child.poll.return_value = None
        with patch.object(vm_transport.subprocess, "Popen", return_value=child) as spawn:
            with self.assertRaisesRegex(vm_transport.TransportError, "deadline"):
                vm_transport._execute(["/fixture"], self.workspace / "out2", self.workspace / "err2",
                                      script=None, seconds=0, output_limit=128)
        self.assertNotIn("start_new_session", spawn.call_args.kwargs)
        self.assertEqual(set(spawn.call_args.kwargs["env"]), {"HOME", "PATH", "TZ", "LANG", "LC_ALL"})
        child.terminate.assert_called_once()
        child.wait.assert_called_once_with(timeout=5)
        child.kill.assert_not_called()

    def test_direct_child_gets_term_before_kill_and_hard_kill_is_not_success(self):
        import subprocess
        child = Mock(returncode=None)
        child.poll.return_value = None
        child.wait.side_effect = [subprocess.TimeoutExpired("fixture", 5), 0]
        with patch.object(vm_transport.subprocess, "Popen", return_value=child):
            with self.assertRaisesRegex(vm_transport.TransportError, "remote cleanup requires VM teardown"):
                vm_transport._execute(["/fixture"], self.workspace / "hard-out", self.workspace / "hard-err",
                                      script=None, seconds=0, output_limit=128)
        child.terminate.assert_called_once()
        child.kill.assert_called_once()
        self.assertEqual(child.wait.call_count, 2)

    def test_stream_failure_or_overflow_retains_diagnostics_without_reuse(self):
        for index, command in enumerate((["/usr/bin/printf", "overflow"], ["/usr/bin/false"], ["/bin/sleep", "2"])):
            with self.assertRaises(vm_transport.TransportError):
                vm_transport._execute(command, self.workspace / f"out{index}", self.workspace / f"err{index}",
                                      script=None, seconds=0.1, output_limit=2)
        with self.assertRaisesRegex(vm_transport.TransportError, "fresh"):
            vm_transport._execute(["never-run"], self.workspace / "out0", self.workspace / "err0",
                                  script=None, seconds=1, output_limit=2)


if __name__ == "__main__": unittest.main()
