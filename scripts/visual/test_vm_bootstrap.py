"""Pure bootstrap generation, frozen-source fixtures and actual host refusal."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import textwrap
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_bootstrap
import vm_bootstrap_guest
import vm_bootstrap_loader
import test_vm_runtime


class BootstrapContracts(unittest.TestCase):
    def setUp(self):
        helper = test_vm_runtime.RuntimeContracts()
        self.fixture, self.digest, self.lock = helper.setup_payload()
        self.addCleanup(helper.doCleanups)
        payload_path = self.fixture.share / "payload.json"
        payload = json.loads(payload_path.read_text())
        for name in (*vm_bootstrap_loader.MODULES, "vm_bootstrap_loader", "vm_bootstrap", "vm_runtime"):
            relative = "scripts/visual/" + name + ".py"
            source = Path(__file__).parent / (name + ".py")
            raw = source.read_bytes()
            target = self.fixture.share / "source" / relative
            target.write_bytes(raw)
            target.chmod(0o400)
            digest = hashlib.sha256(raw).hexdigest()
            payload["sources"][relative] = digest
            payload["files"].append({"payload": "source/" + relative, "guest": str(self.fixture.repo / relative),
                                     "sha256": digest, "bytes": len(raw), "executable": False})
        raw = json.dumps(payload).encode()
        payload_path.write_bytes(raw)
        self.digest = hashlib.sha256(raw).hexdigest()
        self.modules = {name: payload["sources"]["scripts/visual/" + name + ".py"] for name in vm_bootstrap_loader.MODULES}

    def plan(self):
        # Fixture compiled path only at generation time; generated production
        # paths are fixed and their actual execution is never remapped on host.
        with patch.object(vm_bootstrap, "REPO", self.fixture.repo):
            return vm_bootstrap.plan(self.fixture.share, self.digest, self.fixture.repo, "gallery")

    def loader_code(self):
        # Only the local test stand-in reports a read-only mount; production has
        # no override. Fixture files remain private, not real guest evidence.
        source = (Path(__file__).parent / "vm_bootstrap_loader.py").read_text()
        return ("from unittest.mock import patch\nfrom types import SimpleNamespace\nimport os\n"
                "with patch('os.statvfs', return_value=SimpleNamespace(f_flag=os.ST_RDONLY)):\n"
                + textwrap.indent(source, "    "))

    def binding(self):
        return {"share": str(self.fixture.share), "payload_sha256": self.digest,
                "repo": str(self.fixture.repo), "suite": "gallery", "work": str(self.fixture.work),
                "modules": self.modules}

    def test_generation_is_read_only_fixed_quoted_and_source_bound(self):
        script = self.plan()
        self.assertIn("/usr/bin/env -i", script)
        self.assertIn("/bin/zsh -f", script)
        self.assertIn("mount_virtiofs -r bbr-visual-payload", script)
        self.assertIn("--offline", script)
        self.assertIn(self.digest, script)
        self.assertIn(self.lock["python"]["path"], script)
        self.assertNotIn("--realise", script)
        self.assertNotIn("--repair", script)
        self.assertNotIn("curl ", script)
        self.assertNotIn("chown -R", script)
        self.assertNotIn("sys.path.insert", script)
        identity = script.index("bootstrap host forbidden")
        first_runtime = script.index(" hash path ")
        mount = script.index("mount_virtiofs -r")
        python = script.index(" -I -B -c ")
        self.assertLess(identity, first_runtime)
        self.assertLess(first_runtime, mount)
        self.assertLess(mount, python)
        self.assertFalse(self.fixture.repo.exists())
        self.assertFalse(self.fixture.work.exists())
        result = subprocess.run(["/bin/zsh", "-n"], input=script, text=True, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_semantic_bootstrap_is_source_frozen_no_install_and_host_forbidden(self):
        path = self.fixture.share / "payload.json"
        payload = json.loads(path.read_text())
        payload["suite"] = "gallery-semantic"
        for entry in payload["files"]:
            if entry["guest"].endswith("/target/release/bbr-gallery"):
                entry["guest"] = str(self.fixture.repo / "target/release/deps/bbr_gallery-fixture")
        raw = json.dumps(payload).encode()
        path.write_bytes(raw)
        digest = hashlib.sha256(raw).hexdigest()
        with patch.object(vm_bootstrap, "REPO", self.fixture.repo):
            script = vm_bootstrap.plan(self.fixture.share, digest, self.fixture.repo, "gallery-semantic")
        self.assertIn('gallery-semantic', script)
        self.assertIn('--offline', script)
        self.assertIn('mount_virtiofs -r', script)
        for forbidden in ('--realise', '--repair', 'curl ', 'chown -R', 'sys.path.insert'):
            self.assertNotIn(forbidden, script)
        syntax = subprocess.run(["/bin/zsh", "-n"], input=script, text=True, capture_output=True, timeout=5)
        self.assertEqual(syntax.returncode, 0, syntax.stderr)
        actual = subprocess.run(["/bin/zsh", "-f", "-s"], input=script, text=True, capture_output=True, timeout=5)
        self.assertNotEqual(actual.returncode, 0)
        self.assertIn('host forbidden', actual.stderr)
        with patch.object(vm_bootstrap_guest.vm_install, 'inspect') as inspect, \
                patch.object(vm_bootstrap_guest, 'prepare_parent') as parent:
            with self.assertRaisesRegex(Exception, 'host forbidden'):
                vm_bootstrap_guest.run(self.fixture.share, digest, self.fixture.repo,
                                       'gallery-semantic', self.fixture.work)
            inspect.assert_not_called()
            parent.assert_not_called()
        with patch.object(vm_bootstrap, 'REPO', self.fixture.repo):
            with self.assertRaisesRegex(ValueError, 'selected request'):
                vm_bootstrap.plan(self.fixture.share, digest, self.fixture.repo, 'gallery-semantic', '{}', 'a' * 64)
        binding = self.binding()
        binding.update(suite='gallery-semantic', payload_sha256=digest)
        frozen = subprocess.run([sys.executable, '-I', '-B', '-c', self.loader_code(), json.dumps(binding)],
                                text=True, capture_output=True, timeout=5)
        self.assertNotEqual(frozen.returncode, 0)
        self.assertIn('host forbidden', frozen.stderr)
        self.assertNotIn('ModuleNotFoundError', frozen.stderr)
        calls = []
        with patch.object(vm_bootstrap_guest, 'REPO', self.fixture.repo), \
                patch.object(vm_bootstrap_guest, 'identity', side_effect=lambda: calls.append('identity')), \
                patch.object(vm_bootstrap_guest.vm_install, 'inspect', side_effect=lambda *a: calls.append('inspect')), \
                patch.object(vm_bootstrap_guest, 'prepare_parent', side_effect=lambda *a: calls.append('parent')), \
                patch.object(vm_bootstrap_guest.vm_guest, 'main', side_effect=lambda a: calls.append(a)):
            vm_bootstrap_guest.run(self.fixture.share, digest, self.fixture.repo,
                                   'gallery-semantic', self.fixture.work)
        self.assertEqual(calls[:3], ['identity', 'inspect', 'parent'])
        self.assertEqual(calls[3], ['vm_guest.py', str(self.fixture.share), digest, str(self.fixture.repo),
                                   'gallery-semantic', str(self.fixture.work)])
        self.assertFalse(self.fixture.repo.exists())
        self.assertFalse(self.fixture.work.exists())

    def test_heredoc_collision_is_rejected_not_interpreted_as_shell(self):
        with patch.object(vm_bootstrap, "DELIMITER", "set -euo pipefail"), self.assertRaisesRegex(vm_bootstrap.BootstrapError, "delimiter"):
            self.plan()

    def test_actual_host_refuses_before_mount_runtime_or_writes(self):
        result = subprocess.run(["/bin/zsh", "-f", "-s"], input=self.plan(), text=True, capture_output=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("host forbidden", result.stderr)
        self.assertFalse(self.fixture.repo.exists())
        self.assertFalse(self.fixture.work.exists())

    def test_changed_or_missing_helper_and_payload_fail_before_generation(self):
        with self.assertRaises(Exception):
            vm_bootstrap.plan(self.fixture.share, "f" * 64, self.fixture.repo, "gallery")
        path = self.fixture.share / "source/scripts/visual/vm_guest.py"
        path.chmod(0o600)
        path.write_text("raise Exception('unbound bytes executed')")
        with self.assertRaises(Exception): self.plan()
        result = subprocess.run([sys.executable, "-I", "-B", "-c", self.loader_code(), json.dumps(self.binding())],
                                text=True, capture_output=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("helper bytes differ", result.stderr)
        self.assertNotIn("unbound bytes executed", result.stderr)

    def test_missing_helper_cannot_be_omitted_from_an_otherwise_valid_payload(self):
        path = self.fixture.share / "source/scripts/visual/vm_bootstrap_guest.py"
        path.unlink()
        manifest = self.fixture.share / "payload.json"
        payload = json.loads(manifest.read_text())
        relative = "scripts/visual/vm_bootstrap_guest.py"
        del payload["sources"][relative]
        payload["files"] = [f for f in payload["files"] if f["payload"] != "source/" + relative]
        raw = json.dumps(payload).encode()
        manifest.write_bytes(raw)
        self.digest = hashlib.sha256(raw).hexdigest()
        with self.assertRaisesRegex(vm_bootstrap.BootstrapError, "missing source-bound"):
            self.plan()

    def test_loader_rejects_linked_helpers_before_import(self):
        path = self.fixture.share / "source/scripts/visual/vm_install.py"
        copy = self.fixture.root / "linked-helper"
        copy.write_bytes(path.read_bytes())
        for symlink in (False, True):
            path.unlink()
            if symlink: path.symlink_to(copy)
            else: os.link(copy, path)
            result = subprocess.run([sys.executable, "-I", "-B", "-c", self.loader_code(), json.dumps(self.binding())],
                                    text=True, capture_output=True, timeout=5)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("bootstrap source", result.stderr)
            self.assertFalse(self.fixture.repo.exists())

    def test_foreign_repo_and_bootstrap_loader_source_are_rejected(self):
        with self.assertRaisesRegex(vm_bootstrap.BootstrapError, "compiled"):
            vm_bootstrap.plan(self.fixture.share, self.digest, self.fixture.repo, "gallery")
        # A fresh binding to changed helpers cannot silently change the trusted
        # host loader implementation used for generating the script.
        path = self.fixture.share / "source/scripts/visual/vm_bootstrap_loader.py"
        path.chmod(0o600)
        path.write_text("pass")
        with self.assertRaises(Exception): self.plan()

    def test_real_frozen_helper_chain_imports_then_refuses_host_before_mutation(self):
        result = subprocess.run([sys.executable, "-I", "-B", "-c", self.loader_code(), json.dumps(self.binding())],
                                text=True, capture_output=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("host forbidden", result.stderr)
        self.assertNotIn("ModuleNotFoundError", result.stderr)
        self.assertFalse(self.fixture.repo.exists())
        self.assertFalse(self.fixture.work.exists())

    def test_loader_checks_every_helper_before_any_code_runs(self):
        # An early module would create a tripwire if loader import order allowed
        # execution before verifying the later module's independent digest.
        first = self.fixture.share / "source/scripts/visual/vm_install.py"
        first.chmod(0o600)
        first.write_text("raise RuntimeError('executed too early')")
        binding = self.binding()
        binding["modules"]["vm_install"] = hashlib.sha256(first.read_bytes()).hexdigest()
        binding["modules"]["vm_bootstrap_guest"] = "f" * 64
        result = subprocess.run([sys.executable, "-I", "-B", "-c", self.loader_code(), json.dumps(binding)],
                                text=True, capture_output=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("helper bytes differ", result.stderr)
        self.assertNotIn("executed too early", result.stderr)

    def test_loader_executes_verified_frozen_bytes_without_search_path_import(self):
        binding = self.binding()
        marker = self.fixture.root / "frozen-success"
        for name in vm_bootstrap_loader.MODULES:
            path = self.fixture.share / "source/scripts/visual" / (name + ".py")
            if name == "vm_bootstrap_guest":
                source = "import vm_install\ndef run(*args):\n    vm_install.check()\n"
            elif name == "vm_install":
                source = f"def check():\n    open({str(marker)!r}, 'x').write('fixture, not guest evidence')\n"
            else: source = "# frozen fixture\n"
            path.chmod(0o600)
            path.write_text(source)
            binding["modules"][name] = hashlib.sha256(source.encode()).hexdigest()
        result = subprocess.run([sys.executable, "-I", "-B", "-c", self.loader_code(), json.dumps(binding)],
                                text=True, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(marker.exists())
        self.assertFalse(self.fixture.repo.exists())

    def test_verified_helper_executes_frozen_bytes_after_shared_file_changes(self):
        binding = self.binding()
        target = self.fixture.share / "source/scripts/visual/vm_guest.py"
        marker = self.fixture.root / "frozen-not-reread"
        for name in vm_bootstrap_loader.MODULES:
            path = self.fixture.share / "source/scripts/visual" / (name + ".py")
            if name == "vm_bootstrap_guest":
                source = (f"from pathlib import Path\nPath({str(target)!r}).write_text(\"raise RuntimeError('reread changed bytes')\")\n"
                          "import vm_guest\ndef run(*args):\n    vm_guest.check()\n")
            elif name == "vm_guest":
                source = f"def check():\n    open({str(marker)!r}, 'x').write('frozen fixture')\n"
            else: source = "# frozen fixture\n"
            path.chmod(0o600)
            path.write_text(source)
            binding["modules"][name] = hashlib.sha256(source.encode()).hexdigest()
        result = subprocess.run([sys.executable, "-I", "-B", "-c", self.loader_code(), json.dumps(binding)],
                                text=True, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(marker.exists())
        self.assertIn("reread changed bytes", target.read_text())

    def test_writable_share_rejected_before_source_import(self):
        code = (Path(__file__).parent / "vm_bootstrap_loader.py").read_text()
        result = subprocess.run([sys.executable, "-I", "-B", "-c", code, json.dumps(self.binding())],
                                text=True, capture_output=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("mount must be read-only", result.stderr)
        self.assertFalse(self.fixture.repo.exists())

    def test_tail_orders_identity_inspect_parent_and_fixed_guest_main(self):
        calls = []
        with patch.object(vm_bootstrap_guest, "REPO", self.fixture.repo), \
             patch.object(vm_bootstrap_guest, "identity", side_effect=lambda: calls.append("identity")), \
             patch.object(vm_bootstrap_guest.vm_install, "inspect", side_effect=lambda *a: calls.append("inspect")), \
             patch.object(vm_bootstrap_guest, "prepare_parent", side_effect=lambda *a: calls.append("parent")), \
             patch.object(vm_bootstrap_guest.vm_guest, "main", side_effect=lambda a: calls.append(a)):
            vm_bootstrap_guest.run(self.fixture.share, self.digest, self.fixture.repo, "gallery", self.fixture.work)
        self.assertEqual(calls[:3], ["identity", "inspect", "parent"])
        self.assertEqual(calls[3], ["vm_guest.py", str(self.fixture.share), self.digest,
                                   str(self.fixture.repo), "gallery", str(self.fixture.work)])

    def test_existing_compiled_parent_refused_before_ownership_command(self):
        actual_exists = Path.exists
        def exists(path):
            return True if path == Path("/Users/patwoz") else actual_exists(path)
        with patch.object(Path, "exists", exists), \
             patch.object(vm_bootstrap_guest.subprocess, "run") as command, \
             self.assertRaisesRegex(Exception, "never take ownership"):
            vm_bootstrap_guest.prepare_parent(vm_bootstrap_guest.REPO)
        command.assert_not_called()

    def test_exact_parent_commands_have_no_recursive_or_existing_reuse(self):
        from types import SimpleNamespace
        calls = []
        def mkdir(path, **kwargs): calls.append((str(path), kwargs))
        with patch.object(Path, "exists", return_value=False), \
             patch.object(Path, "is_symlink", return_value=False), \
             patch.object(vm_bootstrap_guest.vm_install, "_absolute"), \
             patch.object(vm_bootstrap_guest.subprocess, "run") as command, \
             patch.object(Path, "stat", return_value=SimpleNamespace(st_uid=501, st_mode=0o40700)), \
             patch.object(Path, "mkdir", mkdir):
            vm_bootstrap_guest.prepare_parent(vm_bootstrap_guest.REPO)
        self.assertEqual([call.args[0] for call in command.call_args_list], [
            ["/usr/bin/sudo", "-n", "/bin/mkdir", "-m", "700", "/Users/patwoz"],
            ["/usr/bin/sudo", "-n", "/usr/sbin/chown", "501:20", "/Users/patwoz"]])
        self.assertEqual(calls, [(path, {"mode": 0o700}) for path in (
            "/Users/patwoz/dev", "/Users/patwoz/dev/Personal", "/Users/patwoz/dev/Personal/macos")])

    def test_compiled_checkout_is_the_clone_or_a_sibling_lane_worktree_only(self):
        parent = vm_bootstrap.REPO.parent
        for module, error in ((vm_bootstrap, vm_bootstrap.BootstrapError),
                              (vm_bootstrap_guest, vm_bootstrap_guest.vm_guest.GuestError)):
            for accepted in (vm_bootstrap.REPO, parent / "bbr-content-card-seated-edge",
                             parent / "bbr-lane-2"):
                module.compiled(accepted)
            for rejected in (parent / "bbrx", parent / "bbr-", parent / "bbr-lane/..",
                             parent / "bbr-lane.evil", parent / "bbr-la ne", parent / "other-bbr",
                             parent.parent / "bbr-lane", Path("/private/tmp/bbr-lane"),
                             parent / ("bbr-" + "x" * 70), self.fixture.repo):
                with self.assertRaisesRegex(error, "compiled"):
                    module.compiled(rejected)

    def test_guest_tail_rejects_host_before_source_or_sudo_and_foreign_parent(self):
        with patch.object(vm_bootstrap_guest.vm_install, "inspect") as inspect, \
             patch.object(vm_bootstrap_guest, "prepare_parent") as parent:
            with self.assertRaisesRegex(Exception, "host forbidden"):
                vm_bootstrap_guest.run(self.fixture.share, self.digest, self.fixture.repo, "gallery", self.fixture.work)
        inspect.assert_not_called()
        parent.assert_not_called()
        with patch.object(vm_bootstrap_guest.subprocess, "run") as command:
            with self.assertRaisesRegex(Exception, "compiled"):
                vm_bootstrap_guest.prepare_parent(self.fixture.repo)
        command.assert_not_called()


if __name__ == "__main__":
    unittest.main()
