"""Read-only cleanup observations using fixtures, never Tart/network access."""
from pathlib import Path
import json
import plistlib
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_cleanup


class CleanupObservationContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.name = "bbr-visual-123-abcdef123456"
        self.clone = Mock(vm=self.name, cleaned=True, keep_vm=False, base_disk=None, artifacts=self.root)
        self.clone.tart_process.poll.return_value = 0
        self.clone.tart_process.returncode = 0
        self.clone.tart_process.pid = 1234
        self.inventory = [{"Name": "macos-27-golden-gate", "Source": "local", "Running": False, "State": "stopped"}]
        self.mounts = {"images": []}
        self.calls = []

    def command(self, argv, **kwargs):
        self.calls.append(argv)
        self.assertEqual(kwargs["timeout"], 15)
        if argv == ["tart", "list", "--source", "local", "--format", "json"]:
            return Mock(stdout=json.dumps(self.inventory))
        self.assertEqual(argv, ["hdiutil", "info", "-plist"])
        return Mock(stdout=plistlib.dumps(self.mounts).decode())

    def observe(self, network=ConnectionRefusedError()):
        with patch.object(Path, "home", return_value=self.root), \
             patch.object(vm_cleanup.tart_vm, "run", self.command), \
             patch.object(vm_cleanup.socket, "create_connection", side_effect=network):
            return vm_cleanup.observe(self.clone, "192.168.64.9")

    def test_success_requires_all_resource_observations(self):
        observed = self.observe()
        self.assertEqual(observed["vm"], self.name)
        self.assertTrue(observed["disk_detached"])
        self.assertTrue(observed["owned_process_reaped"])
        self.assertTrue(observed["ssh_unreachable"])
        self.assertEqual(len(self.calls), 2)

    def test_live_process_or_incomplete_cleanup_fails_before_commands(self):
        for field, value in [("cleaned", False), ("keep_vm", True), ("base_disk", "/dev/disk999"), ("vm", "foreign")]:
            with self.subTest(field=field):
                original = getattr(self.clone, field)
                setattr(self.clone, field, value)
                with self.assertRaises(vm_cleanup.CleanupError): self.observe()
                setattr(self.clone, field, original)
        self.clone.tart_process.poll.return_value = None
        with self.assertRaises(vm_cleanup.CleanupError): self.observe()
        self.assertEqual(self.calls, [])

    def test_private_key_or_clone_directory_presence_is_not_clean(self):
        # Empty marker only, no actual key material read or written.
        marker = self.root / "runner_ed25519"
        marker.touch()
        with self.assertRaisesRegex(vm_cleanup.CleanupError, "key remains"): self.observe()
        marker.unlink()
        directory = self.root / ".tart/vms" / self.name
        directory.mkdir(parents=True)
        with self.assertRaisesRegex(vm_cleanup.CleanupError, "directory remains"): self.observe()
        self.assertEqual(self.calls, [])

    def test_stopped_but_undeleted_clone_and_attached_disk_are_rejected(self):
        self.inventory.append({**self.inventory[0], "Name": self.name})
        with self.assertRaisesRegex(vm_cleanup.CleanupError, "still in Tart"): self.observe()
        self.inventory.pop()
        self.mounts["images"] = [{"image-path": str(self.root / ".tart/vms" / self.name / "disk.img")}]
        with self.assertRaisesRegex(vm_cleanup.CleanupError, "still attached"): self.observe()

    def test_unknown_inventory_or_network_error_cannot_prove_absence(self):
        for inventory in ({}, [{}], [self.inventory[0], self.inventory[0]]):
            with self.assertRaises(vm_cleanup.CleanupError):
                vm_cleanup.validate_inventory(json.dumps(inventory), self.name)
        for mounts in ({}, {"images": [{}]}):
            with self.assertRaises(vm_cleanup.CleanupError):
                vm_cleanup.validate_mounts(plistlib.dumps(mounts).decode(), self.root / "disk.img")
        with self.assertRaises(OSError): self.observe(OSError("no routing evidence"))
        from contextlib import nullcontext
        with patch.object(Path, "home", return_value=self.root), \
             patch.object(vm_cleanup.tart_vm, "run", self.command), \
             patch.object(vm_cleanup.socket, "create_connection", return_value=nullcontext()):
            with self.assertRaisesRegex(vm_cleanup.CleanupError, "still reachable"):
                vm_cleanup.observe(self.clone, "192.168.64.9")


if __name__ == "__main__":
    unittest.main()
