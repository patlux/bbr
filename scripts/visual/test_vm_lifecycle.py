"""Lifecycle/transport fault injection; tests never call Tart or network tools."""
from pathlib import Path
import os
import signal
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_lifecycle
import vm_policy
import vm_cleanup


class LifecycleContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.payload = self.root / "payload"
        self.payload.mkdir()
        (self.payload / "payload.json").write_text("{}")
        self.artifacts = self.root / "diagnostics"
        self.artifacts.mkdir(mode=0o700)
        self.clone = Mock()
        self.ssh = ["ssh", "admin@192.168.64.9"]
        observation = patch.object(vm_cleanup, "observe", return_value={"schema": 1, "fixture": True})
        self.observation = observation.start()
        self.addCleanup(observation.stop)

    def boot(self, source, name, artifacts, keep, display, shares, **kwargs):
        self.assertEqual(source, "macos-27-golden-gate")
        self.assertTrue(name.startswith("bbr-visual-"))
        self.assertFalse(keep)
        self.assertEqual(display, "3200x1800")
        self.assertEqual(shares, [f"visual:{self.payload}:ro,tag=bbr-visual-payload"])
        self.assertTrue(kwargs["exclusive_key"])
        self.assertIs(kwargs["prepare_data"], vm_policy.harden_offline)
        return self.clone, self.ssh, "fixture session"

    def test_success_owns_one_hardened_clone_and_cleans_before_return(self):
        with patch.object(vm_lifecycle.tart_vm, "boot", self.boot), patch.object(vm_lifecycle, "audit") as audit:
            with vm_lifecycle.session(self.payload, self.artifacts) as session:
                self.assertEqual(session.ip, "192.168.64.9")
                audit.assert_called_once()
                self.clone.cleanup.assert_not_called()
            self.clone.cleanup.assert_called_once()
        self.observation.assert_called_once_with(self.clone, "192.168.64.9")
        self.assertTrue((self.artifacts / "cleanup-observed.json").is_file())

    def test_audit_failure_prevents_producer_and_still_cleans(self):
        with patch.object(vm_lifecycle.tart_vm, "boot", self.boot), \
             patch.object(vm_lifecycle, "audit", side_effect=RuntimeError("unsafe guest")):
            with self.assertRaisesRegex(RuntimeError, "unsafe guest"):
                with vm_lifecycle.session(self.payload, self.artifacts):
                    self.fail("producer must not run")
        self.clone.cleanup.assert_called_once()

    def test_producer_failure_or_timeout_does_not_skip_cleanup(self):
        with patch.object(vm_lifecycle.tart_vm, "boot", self.boot), patch.object(vm_lifecycle, "audit"):
            with self.assertRaisesRegex(TimeoutError, "deadline"):
                with vm_lifecycle.session(self.payload, self.artifacts):
                    raise TimeoutError("deadline")
        self.clone.cleanup.assert_called_once()

    def test_cleanup_failure_propagates_instead_of_success(self):
        self.clone.cleanup.side_effect = RuntimeError("cleanup failed")
        with patch.object(vm_lifecycle.tart_vm, "boot", self.boot), patch.object(vm_lifecycle, "audit"):
            with self.assertRaisesRegex(RuntimeError, "cleanup failed"):
                with vm_lifecycle.session(self.payload, self.artifacts):
                    pass

    def test_cleanup_observation_failure_prevents_completion_after_commands_succeed(self):
        self.observation.side_effect = vm_lifecycle.tart_vm.RunnerError("owned clone still exists")
        with patch.object(vm_lifecycle.tart_vm, "boot", self.boot), patch.object(vm_lifecycle, "audit"):
            with self.assertRaisesRegex(vm_lifecycle.tart_vm.RunnerError, "still exists"):
                with vm_lifecycle.session(self.payload, self.artifacts):
                    pass
        self.clone.cleanup.assert_called_once()
        self.assertFalse((self.artifacts / "cleanup-observed.json").exists())

    def test_signal_cancellation_cleans_and_restores_handlers(self):
        old = {sig: signal.getsignal(sig) for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)}
        with patch.object(vm_lifecycle.tart_vm, "boot", self.boot), patch.object(vm_lifecycle, "audit"):
            with self.assertRaises(vm_lifecycle.Cancelled):
                with vm_lifecycle.session(self.payload, self.artifacts):
                    os.kill(os.getpid(), signal.SIGTERM)
        self.clone.cleanup.assert_called_once()
        for sig, handler in old.items():
            self.assertEqual(signal.getsignal(sig), handler)

    def test_invalid_guest_ip_still_cleans_without_claiming_reachability(self):
        self.ssh[-1] = "admin@203.0.113.9"
        with patch.object(vm_lifecycle.tart_vm, "boot", self.boot):
            with self.assertRaisesRegex(vm_lifecycle.tart_vm.RunnerError, "no validated guest IP"):
                with vm_lifecycle.session(self.payload, self.artifacts):
                    self.fail("invalid IP must not reach a producer")
        self.clone.cleanup.assert_called_once()
        self.observation.assert_not_called()
        self.assertFalse((self.artifacts / "cleanup-observed.json").exists())

    def test_invalid_payload_and_diagnostics_do_not_boot(self):
        with patch.object(vm_lifecycle.tart_vm, "boot") as boot:
            (self.payload / "payload.json").unlink()
            with self.assertRaises(vm_lifecycle.tart_vm.RunnerError):
                with vm_lifecycle.session(self.payload, self.artifacts):
                    self.fail("no completed payload")
            (self.payload / "payload.json").write_text("{}")
            self.artifacts.chmod(0o755)
            with self.assertRaises(vm_lifecycle.tart_vm.RunnerError):
                with vm_lifecycle.session(self.payload, self.artifacts):
                    self.fail("nonprivate diagnostics")
            boot.assert_not_called()

    def test_actual_audit_requires_auth_listener_and_host_reachability_evidence(self):
        effective = "\n".join(f"{key} {value}" for key, value in {**vm_policy.SSH_SETTINGS, "port": "22"}.items())
        listeners = "p1\nclaunchd\nn*:22\n"
        def guest(_ssh, script, **kwargs):
            text = effective if "sshd -T" in script else listeners
            return type("Result", (), {"stdout": text, "stderr": "", "returncode": 0})()
        from contextlib import nullcontext
        def connect(address, **kwargs):
            if address[1] == 22:
                return nullcontext()
            raise ConnectionRefusedError()
        with patch.object(vm_lifecycle.tart_vm, "guest_script", guest), \
             patch.object(vm_lifecycle.socket, "create_connection", connect):
            vm_lifecycle.audit(self.ssh, "192.168.64.9", self.artifacts)
        self.assertIn("5900 reachable=False", (self.artifacts / "network-audit.log").read_text())
        with patch.object(vm_lifecycle.tart_vm, "guest_script", guest), \
             patch.object(vm_lifecycle.socket, "create_connection", return_value=nullcontext()):
            with self.assertRaises(vm_lifecycle.tart_vm.RunnerError):
                vm_lifecycle.audit(self.ssh, "192.168.64.9", self.artifacts)

    def test_isolated_ssh_ignores_personal_config_agent_and_forwarding(self):
        ssh = vm_lifecycle.tart_vm.ssh_args(self.root / "key", self.root / "known", "192.168.64.9", isolated=True)
        self.assertEqual(ssh[:3], ["ssh", "-F", "/dev/null"])
        for option in ["IdentityAgent=none", "ClearAllForwardings=yes", "ControlMaster=no", "ControlPath=none", "ProxyCommand=none"]:
            self.assertIn(option, ssh)


if __name__ == "__main__":
    unittest.main()
