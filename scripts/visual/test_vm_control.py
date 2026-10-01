"""Protocol parser/publication tests; no guest/producer/network operations."""
from pathlib import Path
import hashlib
import json
import signal
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_control


class ControlContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.control = self.root / "control"
        self.control.mkdir(mode=0o700)
        self.request = {"schema": 1, "control": str(self.control), "payload_sha256": "a" * 64,
                        "operation_ms": 10000, "grace_ms": 1000}
        self.digest = self.write_request(self.request)

    def write_request(self, value):
        raw = json.dumps(value).encode()
        (self.control / "request.json").write_bytes(raw)
        return hashlib.sha256(raw).hexdigest()

    def test_acknowledgement_follows_verification_and_restores_signal_handlers(self):
        previous = {sig: signal.getsignal(sig) for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)}
        order = []
        def operation():
            order.append("operation")
            return "value"
        def verify():
            self.assertFalse((self.control / "terminal.json").exists())
            order.append("cleanup verified")
        self.assertEqual(vm_control.run(self.control, self.digest, operation, verify), "value")
        self.assertEqual(order, ["operation", "cleanup verified"])
        terminal = json.loads((self.control / "terminal.json").read_text())
        self.assertEqual(terminal, {"schema": 1, "request_sha256": self.digest, "outcome": "completed"})
        for sig, handler in previous.items(): self.assertEqual(signal.getsignal(sig), handler)
        tripwire = Mock()
        with self.assertRaises(vm_control.ControlError):
            vm_control.run(self.control, self.digest, tripwire, tripwire)
        tripwire.assert_not_called()

    def test_invalid_identity_fields_and_paths_do_not_call_operation(self):
        for key, value in [("schema", True), ("control", str(self.root)), ("payload_sha256", "bad"),
                           ("operation_ms", 0), ("grace_ms", True), ("extra", "field")]:
            with self.subTest(key=key):
                digest = self.write_request({**self.request, key: value})
                tripwire = Mock()
                with self.assertRaises(vm_control.ControlError):
                    vm_control.run(self.control, digest, tripwire, tripwire)
                tripwire.assert_not_called()
                self.assertFalse((self.control / "terminal.json").exists())
        self.write_request(self.request)
        with self.assertRaises(vm_control.ControlError):
            vm_control.run(self.control, "f" * 64, self.fail, self.fail)
        alias = self.root / "alias"
        alias.symlink_to(self.control)
        with self.assertRaises(vm_control.ControlError):
            vm_control.run(alias, self.digest, self.fail, self.fail)

    def test_cleanup_failure_and_changed_request_never_acknowledge(self):
        def failure(): raise RuntimeError("cleanup failed")
        with self.assertRaisesRegex(RuntimeError, "cleanup failed"):
            vm_control.run(self.control, self.digest, lambda: None, failure)
        self.assertFalse((self.control / "terminal.json").exists())
        def mutate(): self.write_request({**self.request, "payload_sha256": "b" * 64})
        with self.assertRaisesRegex(vm_control.ControlError, "changed"):
            vm_control.run(self.control, self.digest, mutate, lambda: None)
        self.assertFalse((self.control / "terminal.json").exists())

    def test_failed_operation_is_acknowledged_only_after_verified_teardown(self):
        def failure(): raise RuntimeError("producer failed")
        verified = Mock()
        with self.assertRaisesRegex(RuntimeError, "producer failed"):
            vm_control.run(self.control, self.digest, failure, verified)
        verified.assert_called_once()
        self.assertEqual(json.loads((self.control / "terminal.json").read_text())["outcome"], "failed")

    def test_publication_is_no_replace_and_duplicate_json_is_rejected(self):
        path = self.control / "terminal.json"
        path.write_text("preserved")
        with self.assertRaises(FileExistsError): vm_control._publish(path, {"new": "value"})
        self.assertEqual(path.read_text(), "preserved")
        path.write_text('{"schema":1,"schema":1}')
        with self.assertRaisesRegex(vm_control.ControlError, "duplicate"):
            vm_control._read(path)
        self.assertEqual(sorted(p.name for p in self.control.iterdir()), ["request.json", "terminal.json"])


if __name__ == "__main__":
    unittest.main()
