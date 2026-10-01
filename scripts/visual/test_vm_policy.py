"""Clone-policy fixtures; no VM, socket, actual key or host config access."""
from pathlib import Path
import plistlib
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_policy
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import tart_vm


class PolicyContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.data = self.root / "clone-data"
        (self.data / "Users/admin").mkdir(parents=True)
        self.config = self.data / "private/etc/ssh/sshd_config"
        self.config.parent.mkdir(parents=True)
        self.config.write_text("PasswordAuthentication yes\nInclude unsafe.conf\n")
        self.disabled = self.data / "private/var/db/com.apple.xpc.launchd/disabled.plist"
        self.disabled.parent.mkdir(parents=True)
        self.disabled.write_bytes(plistlib.dumps({"preserve.other": True}))

    def effective(self):
        return "\n".join(f"{k} {v}" for k, v in {**vm_policy.SSH_SETTINGS, "port": "22"}.items())

    def test_hardening_is_clone_only_and_replaces_conflicting_config(self):
        sibling = self.root / "source-config"
        sibling.write_text("source image unchanged")
        vm_policy.harden_offline(self.data)
        config = self.config.read_text()
        self.assertNotIn("Include", config)
        self.assertNotIn("PasswordAuthentication yes", config)
        for key, value in vm_policy.SSH_SETTINGS.items():
            self.assertIn(f"{key} {value}\n", config)
        jobs = plistlib.loads(self.disabled.read_bytes())
        self.assertTrue(jobs["preserve.other"])
        self.assertTrue(all(jobs[job] for job in vm_policy.DISABLED_JOBS))
        self.assertEqual(sibling.read_text(), "source image unchanged")
        self.assertEqual(self.config.stat().st_mode & 0o777, 0o600)

    def test_bad_mounts_and_symlink_config_fail_before_writes(self):
        for root in [Path("/"), Path("/System/Volumes/Data"), Path("relative"), self.root]:
            with self.assertRaises(vm_policy.PolicyError):
                vm_policy.harden_offline(root)
        sibling = self.root / "source-config"
        sibling.write_text("unchanged")
        self.config.unlink()
        self.config.symlink_to(sibling)
        with self.assertRaises(vm_policy.PolicyError):
            vm_policy.harden_offline(self.data)
        self.assertEqual(sibling.read_text(), "unchanged")

    def test_effective_authentication_requires_every_safe_setting(self):
        valid = self.effective()
        vm_policy.validate_sshd(valid)
        for key in vm_policy.SSH_SETTINGS:
            with self.assertRaises(vm_policy.PolicyError):
                vm_policy.validate_sshd("\n".join(line for line in valid.splitlines() if not line.startswith(key + " ")))
            with self.assertRaises(vm_policy.PolicyError):
                vm_policy.validate_sshd(valid + f"\n{key} unsafe")
        with self.assertRaises(vm_policy.PolicyError):
            vm_policy.validate_sshd(valid.replace("passwordauthentication no", "passwordauthentication yes"))

    def test_listener_audit_rejects_unrelated_services_or_failed_observations(self):
        valid = "p1\nclaunchd\nf3\nn*:22\np2\ncfixture\nn127.0.0.1:3456\nn[::1]:7890\n"
        vm_policy.validate_listeners(valid)
        for invalid in ["", "permission denied", valid + "p3\ncscreensharing\nn*:5900\n",
                        valid + "p3\nckdc\nn[::]:88\n", "p1\ncevil\nn*:22\n",
                        "p1\nclaunchd\nnno-port", valid + "unexpected"]:
            with self.assertRaises(vm_policy.PolicyError):
                vm_policy.validate_listeners(invalid)

    def test_only_single_private_nat_ip_is_accepted(self):
        for ip in ["192.168.64.3", "10.0.0.2", "172.16.1.2"]:
            self.assertEqual(vm_policy.validate_ip(ip), ip)
        for ip in ["127.0.0.1", "8.8.8.8", "169.254.2.3", "::1", "192.168.64.3\n192.168.64.4", "-oProxyCommand=bad"]:
            with self.assertRaises(vm_policy.PolicyError):
                vm_policy.validate_ip(ip)

    def test_public_boot_ip_is_refused_before_initial_ssh(self):
        from unittest.mock import Mock
        clone = Mock()
        with patch.object(tart_vm, "Clone", return_value=clone), \
             patch.object(tart_vm, "run"), patch.object(tart_vm, "inject_key", return_value=(self.root / "key", "fingerprint")), \
             patch.object(tart_vm.subprocess, "Popen"), patch.object(tart_vm, "wait_until", return_value="8.8.8.8"), \
             patch.object(tart_vm, "ssh_args") as ssh:
            with self.assertRaises(tart_vm.RunnerError):
                tart_vm.boot("golden", "clone", self.root, False, "3200x1800", [], exclusive_key=True)
        ssh.assert_not_called()
        clone.cleanup.assert_called_once()

    def test_offline_hook_and_exclusive_key_precede_unmount(self):
        clone = tart_vm.Clone("bbr-visual-fixture", self.root, False)
        clone.base_disk = "/dev/disk999"
        key = self.root / "runner_ed25519"
        # Public test value only; no private key is created/read in this fixture.
        key.with_suffix(".pub").write_text("ssh-ed25519 SYNTHETIC-TEST-PUBLIC")
        auth = self.data / "Users/admin/.ssh/authorized_keys"
        auth.parent.mkdir()
        auth.write_text("old-public-key")
        calls = []

        def run(argv, **kwargs):
            calls.append(argv)
            if argv[0:2] == ["diskutil", "unmount"]:
                self.assertIn("disableforwarding yes", self.config.read_text())
                self.assertNotIn("old-public-key", auth.read_text())
            return type("Result", (), {"stdout": "public fingerprint", "returncode": 0})()

        with patch.object(tart_vm, "run", run), patch.object(tart_vm, "_attach_data_volume", return_value=self.data), \
             patch.object(tart_vm.os, "chown"), patch.object(tart_vm.os, "chmod"):
            tart_vm.inject_key(clone, exclusive_key=True, prepare_data=vm_policy.harden_offline)
        self.assertIsNone(clone.base_disk)
        self.assertEqual(auth.read_text(), "ssh-ed25519 SYNTHETIC-TEST-PUBLIC\n")


if __name__ == "__main__":
    unittest.main()
