"""Fault-injection tests of the reused Tart lifecycle; never invokes Tart/SSH."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import tart_vm


class CleanupContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        # Synthetic lifecycle marker, not key material.
        self.key = self.root / "runner_ed25519"
        self.key.touch()
        self.clone = tart_vm.Clone("bbr-visual-fixture", self.root, False)
        self.calls = []

    def runner(self, failures=()):
        def run(argv, **kwargs):
            self.calls.append(argv[:])
            self.assertLessEqual(kwargs.get("timeout", 600), 30)
            if argv[0:2] in failures:
                raise tart_vm.RunnerError("injected command timeout")
            return subprocess.CompletedProcess(argv, 0, "", "")
        return run

    def test_detach_failure_does_not_skip_stop_delete_or_evidence(self):
        self.clone.base_disk = "/dev/disk999"
        with patch.object(tart_vm, "run", self.runner((["hdiutil", "detach"],))):
            with self.assertRaises(tart_vm.RunnerError):
                self.clone.cleanup()
        self.assertIn(["tart", "stop", self.clone.vm], self.calls)
        self.assertIn(["tart", "delete", self.clone.vm], self.calls)
        self.assertEqual(self.clone.base_disk, "/dev/disk999")
        self.assertFalse(self.key.exists())
        self.assertIn("detach_error=", (self.root / "cleanup.log").read_text())

    def test_stop_timeout_still_reaps_owned_process_and_deletes_clone(self):
        child = Mock()
        child.poll.return_value = None
        self.clone.tart_process = child
        with patch.object(tart_vm, "run", self.runner((["tart", "stop"],))):
            with self.assertRaises(tart_vm.RunnerError):
                self.clone.cleanup()
        child.terminate.assert_called_once()
        child.wait.assert_called_once_with(timeout=20)
        self.assertIn(["tart", "delete", self.clone.vm], self.calls)
        self.assertFalse(self.key.exists())

    def test_owned_process_timeout_escalates_and_is_reaped(self):
        child = Mock()
        child.poll.return_value = None
        child.wait.side_effect = [subprocess.TimeoutExpired("tart", 20), 0]
        self.clone.tart_process = child
        with patch.object(tart_vm, "run", self.runner()):
            self.clone.cleanup()
        child.kill.assert_called_once()
        self.assertEqual(child.wait.call_count, 2)
        self.assertFalse(self.key.exists())

    def test_failed_delete_preserves_key_and_never_records_clean(self):
        def run(argv, **kwargs):
            self.calls.append(argv)
            return subprocess.CompletedProcess(argv, 1 if argv[1] == "delete" else 0, "", "")
        with patch.object(tart_vm, "run", run):
            with self.assertRaises(tart_vm.RunnerError):
                self.clone.cleanup()
        self.assertTrue(self.key.exists())
        log = (self.root / "cleanup.log").read_text()
        self.assertIn("delete_rc=1", log)
        self.assertNotIn("private_key=deleted", log)

    def test_successful_cleanup_is_idempotent_and_preserves_first_record(self):
        with patch.object(tart_vm, "run", self.runner()):
            self.clone.cleanup()
            first = (self.root / "cleanup.log").read_bytes()
            self.clone.cleanup()
        self.assertEqual(len(self.calls), 2)
        self.assertEqual((self.root / "cleanup.log").read_bytes(), first)
        self.assertIn(b"private_key=deleted", first)

    def test_log_failure_is_not_silent_success(self):
        with patch.object(tart_vm, "run", self.runner()), patch.object(Path, "write_text", side_effect=OSError("injected log failure")):
            with self.assertRaises(tart_vm.RunnerError):
                self.clone.cleanup()
        self.assertFalse(self.key.exists())

    def test_keep_vm_does_not_delete_clone_or_key(self):
        self.clone.keep_vm = True
        with patch.object(tart_vm, "run", self.runner()):
            self.clone.cleanup()
        self.assertEqual(self.calls, [["tart", "stop", self.clone.vm]])
        self.assertTrue(self.key.exists())
        self.assertIn("delete=skipped_keep_vm", (self.root / "cleanup.log").read_text())


if __name__ == "__main__":
    unittest.main()
