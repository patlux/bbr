"""Selected request bytes across fixed transport/bootstrap/guest seams.

All resource/producer observations below are local synthetic fixtures, not VM
or pixel evidence. Rust integration independently validates returned receipts.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import test_vm_bootstrap
import test_vm_guest
import test_vm_transport
import vm_bootstrap
import vm_bootstrap_guest
import vm_cleanup
import vm_guest
import vm_selection
import vm_transport


def request(digest):
    text = json.dumps({"schema": 1, "payload_sha256": digest, "selection": {
        "catalog": "fixture.json", "catalog_hash": "c" * 64,
        "stories": [{"id": "gallery-00", "label": "quoted '\"\\ newline\nfixture", "cases": ["fixture"]}]}})
    return text, hashlib.sha256(text.encode()).hexdigest()


class SelectedTransportContracts(unittest.TestCase):
    def guest(self):
        f = test_vm_guest.GuestContracts()
        f.setUp()
        self.addCleanup(f.doCleanups)
        return f, f.payload("gallery")

    def test_request_hash_payload_suite_bounds_and_links_fail_closed(self):
        f, digest = self.guest()
        text, sha = request(digest)
        self.assertEqual(vm_selection.decode(text, sha, digest, "gallery"), text.encode())
        for value, expected, payload, suite in [(text, None, digest, "gallery"), (None, sha, digest, "gallery"),
                (text, "f"*64, digest, "gallery"), (text, sha, "f"*64, "gallery"),
                (text, sha, digest, "browser"), (" "* (vm_selection.LIMIT+1), sha, digest, "gallery")]:
            with self.assertRaises(Exception): vm_selection.decode(value, expected, payload, suite)
        f.work.mkdir()
        vm_selection.write(f.work, text, sha, digest, "gallery")
        self.assertEqual(vm_selection.read(f.work, sha, digest, "gallery"), text)
        with self.assertRaises(FileExistsError): vm_selection.write(f.work, text, sha, digest, "gallery")
        path = f.work / vm_selection.NAME
        copy = f.root / "copy"
        path.rename(copy)
        for link in (os.link, os.symlink):
            link(copy, path)
            with self.assertRaises(Exception): vm_selection.read(f.work, sha, digest, "gallery")
            path.unlink()
        for mutation in ({"schema": True}, {"extra": 1}):
            bad = json.loads(text); bad.update(mutation); raw = json.dumps(bad)
            with self.assertRaises(ValueError):
                vm_selection.decode(raw, hashlib.sha256(raw.encode()).hexdigest(), digest, "gallery")

    def test_guest_uses_only_selected_action_with_exact_request_and_independent_hash(self):
        f, digest = self.guest()
        text, sha = request(digest)
        def selected(consumer, suite, action, repo, share, payload, work, seconds, expected):
            self.assertEqual(action, "capture-selected")
            self.assertEqual(expected, sha)
            self.assertEqual((work / vm_selection.NAME).read_bytes(), text.encode())
            f.producer(consumer, suite, action, repo, share, payload, work, seconds)
            (work / "live-inventory.tsv").write_text("synthetic selected inventory\n")
            (work / "raw/selection.json").write_text(json.dumps(json.loads(text)["selection"]))
        with patch.object(vm_guest.vm_observe, "observe", return_value=f.environment), patch.object(vm_guest, "_producer", selected):
            value = vm_guest.run(f.share, digest, f.repo, "gallery", f.work, text, sha)
        self.assertEqual(f.calls, ["capture-selected"])
        self.assertIn("selection.json", value["files"])
        self.assertNotIn(vm_selection.NAME, value["files"])
        self.assertFalse((f.work / "raw/capture.json").exists())

    def test_guest_bad_request_refuses_before_install_and_mutation_refuses_receipt(self):
        f, digest = self.guest()
        text, sha = request(digest)
        with patch.object(vm_guest.vm_observe, "observe", return_value=f.environment), patch.object(vm_guest, "_producer") as producer:
            with self.assertRaises(Exception): vm_guest.run(f.share, digest, f.repo, "gallery", f.work, text, "f"*64)
        producer.assert_not_called()
        self.assertFalse(f.repo.exists())
        def mutate(*args): (f.work / vm_selection.NAME).write_text("{}")
        with patch.object(vm_guest.vm_observe, "observe", return_value=f.environment), patch.object(vm_guest, "_producer", mutate):
            with self.assertRaises(Exception): vm_guest.run(f.share, digest, f.repo, "gallery", f.work, text, sha)
        self.assertFalse((f.work / "return.tar").exists())
        self.assertFalse((f.work / "raw/guest-receipt.json").exists())

    def test_selected_direct_consumer_argv_has_hash_and_same_owned_child_policy(self):
        f, digest = self.guest()
        text, sha = request(digest)
        f.work.mkdir()
        vm_selection.write(f.work, text, sha, digest, "gallery")
        child = Mock(returncode=0); child.poll.return_value = 0
        with patch.object(vm_guest.subprocess, "Popen", return_value=child) as spawn:
            vm_guest._producer(f.root / "consumer", "gallery", "capture-selected", f.repo, f.share, digest, f.work, 660, sha)
        self.assertEqual(spawn.call_args.args[0], [str(f.root / "consumer"), "guest-execute", "gallery", "capture-selected", str(f.repo), str(f.share), digest, str(f.work), sha])
        self.assertNotIn("start_new_session", spawn.call_args.kwargs)
        self.assertEqual(spawn.call_args.kwargs["env"]["HOME"], "/Users/admin")

    def test_selected_bootstrap_is_quoted_source_bound_and_tail_preserves_exact_bytes(self):
        f = test_vm_bootstrap.BootstrapContracts(); f.setUp(); self.addCleanup(f.doCleanups)
        text, sha = request(f.digest)
        with patch.object(vm_bootstrap, "REPO", f.fixture.repo):
            script = vm_bootstrap.plan(f.fixture.share, f.digest, f.fixture.repo, "gallery", text, sha)
            with self.assertRaises(Exception): vm_bootstrap.plan(f.fixture.share, f.digest, f.fixture.repo, "gallery", text, "f"*64)
        result = subprocess.run(["/bin/zsh", "-n"], input=script, text=True, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(sha, script)
        with patch.object(vm_bootstrap_guest, "REPO", f.fixture.repo), patch.object(vm_bootstrap_guest, "identity"), \
             patch.object(vm_bootstrap_guest, "prepare_parent"), patch.object(vm_bootstrap_guest.vm_guest, "main") as main:
            vm_bootstrap_guest.run(f.fixture.share, f.digest, f.fixture.repo, "gallery", f.fixture.work, text, sha)
        self.assertEqual(main.call_args.args[0][-2:], [text, sha])
        self.assertFalse(f.fixture.work.exists())

    def test_frozen_loader_conveys_exact_selected_bytes_without_file_reread(self):
        f = test_vm_bootstrap.BootstrapContracts(); f.setUp(); self.addCleanup(f.doCleanups)
        text, sha = request(f.digest)
        binding = f.binding(); binding.update(selected=text, selected_sha256=sha)
        name = "vm_bootstrap_guest"
        path = f.fixture.share / "source/scripts/visual" / (name + ".py")
        code = f"def run(*args):\n    assert args[-2:] == ({text!r}, {sha!r})\n"
        path.chmod(0o600); path.write_text(code)
        binding["modules"][name] = hashlib.sha256(code.encode()).hexdigest()
        result = subprocess.run([sys.executable, "-I", "-B", "-c", f.loader_code(), json.dumps(binding)], text=True, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        null_binding = dict(binding, selected=None, selected_sha256=None)
        result = subprocess.run([sys.executable, "-I", "-B", "-c", f.loader_code(), json.dumps(null_binding)], text=True, capture_output=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid bootstrap binding", result.stderr)
        del binding["selected_sha256"]
        result = subprocess.run([sys.executable, "-I", "-B", "-c", f.loader_code(), json.dumps(binding)], text=True, capture_output=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid bootstrap binding", result.stderr)

    def test_transport_forwards_original_bytes_and_bad_hash_never_boots(self):
        f = test_vm_transport.TransportContracts(); f.setUp(); self.addCleanup(f.doCleanups)
        text, sha = request(f.digest)
        vm_selection.write(f.workspace, text, sha, f.digest, "gallery")
        with patch.object(vm_transport.vm_bootstrap, "plan", return_value="fixture") as plan, \
             patch.object(vm_transport.vm_lifecycle, "session", f.lifecycle), \
             patch.object(vm_transport, "_exchange", f.exchange), \
             patch.object(vm_cleanup, "recheck", f.recheck):
            with self.assertRaises(Exception):
                vm_transport.run(f.fixture.share, f.digest, f.fixture.repo, "gallery", f.workspace, f.vm, f.control, f.request, "f"*64)
            self.assertEqual(f.calls, [])
            vm_transport.run(f.fixture.share, f.digest, f.fixture.repo, "gallery", f.workspace, f.vm, f.control, f.request, sha)
        self.assertEqual(plan.call_args.args[-2:], (text, sha))
        self.assertEqual(f.terminal(), "completed")


if __name__ == "__main__": unittest.main()
