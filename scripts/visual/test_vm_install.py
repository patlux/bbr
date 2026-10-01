"""Synthetic payload/installation contracts; no VM, native binary or host repo writes."""
from pathlib import Path
import copy
import hashlib
import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_install


class InstallationContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.share = self.root / "share"
        self.share.mkdir()
        self.repo = self.root / "compiled-repo"
        self.files = {"source/src/lib.rs": (b"// source\n", "src/lib.rs", False),
                      "executables/0": (b"not a real executable", "target/release/bbr-gallery", True)}
        self.value = {"schema": 1, "suite": "gallery", "repo": str(self.repo), "sources": {}, "files": []}
        for name, (data, guest, executable) in self.files.items():
            path = self.share / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            path.chmod(0o500 if executable else 0o400)
            digest = hashlib.sha256(data).hexdigest()
            self.value["files"].append({"payload": name, "guest": str(self.repo / guest),
                                        "sha256": digest, "bytes": len(data), "executable": executable})
            if not executable:
                self.value["sources"][guest] = digest
        self.digest = self.manifest(self.value)

    def manifest(self, value):
        raw = json.dumps(value).encode()
        (self.share / "payload.json").write_bytes(raw)
        return hashlib.sha256(raw).hexdigest()

    def install(self, digest=None):
        return vm_install.install(self.share, digest or self.digest, self.repo, "gallery")

    def test_exact_compiled_paths_and_modes_without_execution_or_completion_seal(self):
        installed = self.install()
        self.assertEqual(installed, {entry["guest"]: entry["sha256"] for entry in self.value["files"]})
        for data, guest, executable in self.files.values():
            path = self.repo / guest
            self.assertEqual(path.read_bytes(), data)
            self.assertEqual(path.stat().st_mode & 0o777, 0o500 if executable else 0o400)
        self.assertEqual(installed, vm_install.verify_installed(self.share, self.digest, self.repo, "gallery"))
        self.assertFalse((self.repo / "capture.json").exists())
        with self.assertRaisesRegex(vm_install.InstallError, "fresh compiled repo"):
            self.install()

    def test_malformed_manifest_mappings_fail_before_any_installation(self):
        for mutation in ["schema", "suite", "repo", "source", "duplicate", "guest", "role", "namespace",
                         "traversal", "sensitive", "size", "executable", "unknown", "digest", "case-alias", "parent-file"]:
            with self.subTest(mutation=mutation):
                value = copy.deepcopy(self.value)
                source, binary = value["files"]
                if mutation == "schema": value["schema"] = True
                if mutation == "suite": value["suite"] = "browser"
                if mutation == "repo": value["repo"] = str(self.root)
                if mutation == "source": value["sources"]["missing.rs"] = "a" * 64
                if mutation == "duplicate": value["files"].append(source.copy())
                if mutation == "guest": source["guest"] = str(self.root / "outside")
                if mutation == "role": binary["guest"] = str(self.repo / "target/release/foreign")
                if mutation == "namespace": source["payload"] = "other/src/lib.rs"
                if mutation == "traversal": source["guest"] = str(self.repo / "../outside")
                if mutation == "sensitive": source["payload"] = "source/secrets/synthetic.rs"
                if mutation == "size": source["bytes"] = True
                if mutation == "executable": source["executable"] = 1
                if mutation == "unknown": source["extra"] = "not allowed"
                if mutation == "digest": source["sha256"] = "no digest"
                if mutation in {"case-alias", "parent-file"}:
                    name = "src/LIB.rs" if mutation == "case-alias" else "src"
                    value["sources"][name] = source["sha256"]
                    value["files"].append({**source, "payload": "source/" + name, "guest": str(self.repo / name)})
                with self.assertRaises(vm_install.InstallError):
                    self.install(self.manifest(value))
                self.assertFalse(self.repo.exists())

    def test_wrong_or_duplicate_manifest_and_limits_never_create_output(self):
        with self.assertRaisesRegex(vm_install.InstallError, "manifest hash"):
            self.install("f" * 64)
        raw = (self.share / "payload.json").read_bytes().replace(b'"schema": 1', b'"schema": 1, "schema": 1')
        (self.share / "payload.json").write_bytes(raw)
        with self.assertRaisesRegex(vm_install.InstallError, "duplicate"):
            self.install(hashlib.sha256(raw).hexdigest())
        self.manifest(self.value)
        for field in ("FILE_LIMIT", "TOTAL_LIMIT", "MANIFEST_LIMIT", "MAX_FILES"):
            with self.subTest(field=field), patch.object(vm_install, field, 1):
                with self.assertRaises(vm_install.InstallError):
                    self.install()
        self.assertFalse(self.repo.exists())

    def test_missing_extra_bytes_modes_links_and_nonregular_entries_fail_before_writes(self):
        path = self.share / "source/src/lib.rs"
        data = path.read_bytes()
        for mutation in ["missing", "bytes", "mode", "symlink", "hardlink", "fifo", "extra-file", "extra-dir"]:
            with self.subTest(mutation=mutation):
                path.chmod(0o600)
                if mutation in {"missing", "symlink", "hardlink", "fifo"}: path.unlink()
                if mutation == "bytes": path.write_bytes(b"different")
                if mutation == "mode": path.chmod(0o700)
                if mutation == "symlink": path.symlink_to(self.share / "executables/0")
                if mutation == "hardlink": os.link(self.share / "executables/0", path)
                if mutation == "fifo": os.mkfifo(path)
                if mutation == "extra-file": (self.share / "extra").write_bytes(b"x")
                if mutation == "extra-dir": (self.share / "empty").mkdir()
                with self.assertRaises(vm_install.InstallError): self.install()
                self.assertFalse(self.repo.exists())
                if path.exists() or path.is_symlink(): path.unlink()
                path.write_bytes(data)
                path.chmod(0o400)
                if (self.share / "extra").exists(): (self.share / "extra").unlink()
                if (self.share / "empty").exists(): (self.share / "empty").rmdir()

    def test_fresh_canonical_disjoint_destination_required(self):
        alias = self.root / "alias"
        alias.symlink_to(self.root)
        for repo in (Path("relative"), alias / "repo", self.share, self.share / "nested", self.root):
            with self.subTest(repo=repo), self.assertRaises(vm_install.InstallError):
                vm_install.install(self.share, self.digest, repo, "gallery")
        self.assertFalse(self.repo.exists())

    def test_interruption_and_shared_mutation_leave_no_success_or_overwrite(self):
        original = vm_install._copy_hash
        def interrupted(path, record, deadline, output=None):
            if output is not None:
                output.write(b"partial")
                raise vm_install.InstallError("injected interruption")
            return original(path, record, deadline, output)
        with patch.object(vm_install, "_copy_hash", interrupted):
            with self.assertRaisesRegex(vm_install.InstallError, "injected interruption"): self.install()
        self.assertTrue(self.repo.is_dir())
        with self.assertRaises(vm_install.InstallError):
            vm_install.verify_installed(self.share, self.digest, self.repo, "gallery")
        with self.assertRaisesRegex(vm_install.InstallError, "fresh compiled repo"): self.install()

    def test_share_mutation_during_installation_is_rejected_after_copy(self):
        original = vm_install._copy_hash
        def mutate(path, record, deadline, output=None):
            result = original(path, record, deadline, output)
            if output is not None and record["executable"]:
                self.manifest({**self.value, "suite": "browser"})
            return result
        with patch.object(vm_install, "_copy_hash", mutate):
            with self.assertRaisesRegex(vm_install.InstallError, "manifest hash"): self.install()
        self.assertTrue(self.repo.is_dir())

    def test_installed_drift_extra_files_and_deadlines_are_rejected(self):
        self.install()
        path = self.repo / "src/lib.rs"
        path.chmod(0o600)
        with self.assertRaisesRegex(vm_install.InstallError, "permissions"):
            vm_install.verify_installed(self.share, self.digest, self.repo, "gallery")
        path.write_bytes(b"mutated source")
        path.chmod(0o400)
        with self.assertRaises(vm_install.InstallError):
            vm_install.verify_installed(self.share, self.digest, self.repo, "gallery")
        path.chmod(0o600)
        path.write_bytes(self.files["source/src/lib.rs"][0])
        path.chmod(0o400)
        (self.repo / "unexpected").write_bytes(b"x")
        with self.assertRaises(vm_install.InstallError):
            vm_install.verify_installed(self.share, self.digest, self.repo, "gallery")
        with self.assertRaisesRegex(vm_install.InstallError, "deadline"):
            vm_install.inspect(self.share, self.digest, self.repo, "gallery", deadline=0)


if __name__ == "__main__":
    unittest.main()
