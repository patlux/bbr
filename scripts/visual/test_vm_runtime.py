"""Offline runtime contracts. Fixture executables are never executed."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_runtime
import test_vm_guest

ROOT = "/nix/store/" + "0" * 32 + "-python3-3.13.11"
LIB = "/nix/store/" + "1" * 32 + "-libiconv-fixture"
NAR = "sha256-" + "A" * 43 + "="


def lock_fixture():
    return {"schema": 1, "arch": "arm64", "python": {
        "path": ROOT + "/bin/python3.13", "version": "3.13.11", "sha256": "a" * 64},
        "roots": [ROOT, LIB], "paths": {
            ROOT: {"narHash": NAR, "narSize": 80, "references": [ROOT, LIB]},
            LIB: {"narHash": NAR, "narSize": 32, "references": [LIB]}}}


class RuntimeContracts(unittest.TestCase):
    def test_lock_requires_exact_complete_bounded_reachable_closure(self):
        valid = lock_fixture()
        self.assertEqual(vm_runtime.validate(valid), valid)
        mutations = []
        def variant(change):
            value = deepcopy(valid)
            change(value)
            mutations.append(value)
        variant(lambda v: v.update(schema=True))
        variant(lambda v: v.update(arch="x86_64"))
        variant(lambda v: v.update(unexpected=True))
        variant(lambda v: v["python"].update(path="/usr/bin/python3"))
        variant(lambda v: v["python"].update(path=ROOT + "/bin/../python3"))
        variant(lambda v: v["python"].update(sha256="bad"))
        variant(lambda v: v["roots"].append(ROOT))
        variant(lambda v: v["roots"].remove(ROOT))
        variant(lambda v: v["paths"][LIB].update(narSize=True))
        variant(lambda v: v["paths"][LIB].update(narSize=vm_runtime.TOTAL_LIMIT + 1))
        variant(lambda v: v["paths"][LIB].update(narHash="sha256-invalid"))
        variant(lambda v: v["paths"][LIB]["references"].append("/nix/store/" + "2" * 32 + "-missing"))
        variant(lambda v: v["paths"][LIB]["references"].append(LIB))
        variant(lambda v: v["paths"].update({"/nix/store/" + "2" * 32 + "-unreachable": v["paths"][LIB]}))
        for value in mutations:
            with self.subTest(value=value), self.assertRaises(vm_runtime.RuntimeError):
                vm_runtime.validate(value)

    def test_macho_dependencies_are_exact_system_or_locked_nix_paths(self):
        artifact = Path("/private/tmp/synthetic executable")
        text = (f"{artifact}:\n"
                "\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1.0.0)\n"
                f"\t{LIB}/lib/iconv.dylib (compatibility version 1.0.0, current version 1.0.0)\n")
        self.assertEqual(vm_runtime.dependencies(text, artifact, lock_fixture()), [LIB + "/lib/iconv.dylib"])
        for replacement in ("@rpath/iconv.dylib", "/opt/homebrew/lib/iconv.dylib", LIB + "/../other",
                            "/usr/library/other", "/System/Library/Frameworks/../other",
                            "/nix/store/" + "2" * 32 + "-unknown/lib/x"):
            with self.subTest(replacement=replacement), self.assertRaises(vm_runtime.RuntimeError):
                vm_runtime.dependencies(text.replace(LIB + "/lib/iconv.dylib", replacement), artifact, lock_fixture())
        for bad in ("", text.replace(str(artifact), "/wrong"), text + "otool error\n", text.splitlines()[0]):
            with self.assertRaises(vm_runtime.RuntimeError):
                vm_runtime.dependencies(bad, artifact, lock_fixture())

    def test_metadata_rejects_missing_extra_changed_and_duplicate_json(self):
        expected = lock_fixture()["paths"]
        self.assertEqual(vm_runtime.metadata(json.dumps(expected)), expected)
        with self.assertRaises(vm_runtime.RuntimeError):
            vm_runtime.metadata('{"a":1,"a":2}')
        with self.assertRaises(vm_runtime.RuntimeError):
            vm_runtime.metadata("[]")
        for field, value in (("narSize", 999), ("narHash", "sha256-" + "B" * 43 + "="), ("references", [])):
            actual = deepcopy(expected)
            actual[ROOT][field] = value
            with self.assertRaises(vm_runtime.RuntimeError):
                vm_runtime.compare_metadata(json.dumps(actual), expected)
        for actual in ({}, {**expected, "extra": expected[LIB]}):
            with self.assertRaises(vm_runtime.RuntimeError):
                vm_runtime.compare_metadata(json.dumps(actual), expected)

    def setup_payload(self):
        fixture = test_vm_guest.GuestContracts()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        fixture.payload("gallery")
        lock = lock_fixture()
        relative = "scripts/visual/guest-runtime.json"
        raw = json.dumps(lock).encode()
        path = fixture.share / "source" / relative
        path.parent.mkdir(parents=True)
        path.write_bytes(raw)
        path.chmod(0o400)
        payload_path = fixture.share / "payload.json"
        payload = json.loads(payload_path.read_text())
        digest = hashlib.sha256(raw).hexdigest()
        payload["sources"][relative] = digest
        payload["files"].append({"payload": "source/" + relative, "guest": str(fixture.repo / relative),
                                 "sha256": digest, "bytes": len(raw), "executable": False})
        raw = json.dumps(payload).encode()
        payload_path.write_bytes(raw)
        return fixture, hashlib.sha256(raw).hexdigest(), lock

    def test_bound_payload_all_artifacts_and_runtime_checked_before_success(self):
        fixture, digest, lock = self.setup_payload()
        calls = []
        def command(argv, **kwargs):
            calls.append(argv)
            if argv[0] == "/usr/bin/lipo": return "arm64\n"
            if argv[0] == "/usr/bin/otool":
                return f"{argv[-1]}:\n\t{LIB}/lib/iconv.dylib (compatibility version 1.0.0, current version 1.0.0)\n"
            if "path-info" in argv: return json.dumps(lock["paths"])
            if "--verify-path" in argv: return ""
            self.fail(f"unexpected command {argv}")
        with patch.object(vm_runtime, "_command", side_effect=command), \
             patch.object(vm_runtime.shutil, "which", side_effect=lambda name: "/fixture/" + name), \
             patch.object(vm_runtime, "_python", return_value=lock["python"]) as python:
            value = vm_runtime.verify(fixture.share, digest, fixture.repo, "gallery")
        self.assertEqual(value["payload_sha256"], digest)
        self.assertEqual(len(value["dependencies"]), 2)
        self.assertEqual(sum("--verify-path" in call for call in calls), 1)
        self.assertEqual(sum("path-info" in call for call in calls), 2)
        python.assert_called_once_with(lock)
        self.assertFalse(fixture.repo.exists())
        self.assertFalse(fixture.work.exists())
        self.assertTrue(all(not any(arg in call for arg in ("--realise", "--repair", "build", "copy")) for call in calls))
        with patch.object(vm_runtime, "_command") as command:
            with self.assertRaises(Exception):
                vm_runtime.verify(fixture.share, "f" * 64, fixture.repo, "gallery")
            command.assert_not_called()

    def test_architecture_closure_and_byte_verification_failure_stop(self):
        fixture, digest, lock = self.setup_payload()
        for failure in ("arch", "closure", "verify", "post-closure"):
            queries = []
            def command(argv, **kwargs):
                if argv[0] == "/usr/bin/lipo": return "x86_64" if failure == "arch" else "arm64"
                if argv[0] == "/usr/bin/otool":
                    return f"{argv[-1]}:\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1.0.0)\n"
                if "path-info" in argv:
                    queries.append(1)
                    return "{}" if failure == "closure" or (failure == "post-closure" and len(queries) == 2) else json.dumps(lock["paths"])
                if "--verify-path" in argv:
                    if failure == "verify": raise vm_runtime.RuntimeError("injected verification failure")
                    return ""
                self.fail(f"unexpected command {argv}")
            with self.subTest(failure=failure), patch.object(vm_runtime, "_command", side_effect=command), \
                 patch.object(vm_runtime.shutil, "which", side_effect=lambda name: "/fixture/" + name), \
                 patch.object(vm_runtime, "_python", return_value=lock["python"]), self.assertRaises(vm_runtime.RuntimeError):
                vm_runtime.verify(fixture.share, digest, fixture.repo, "gallery")

    def test_runtime_lock_is_strict_source_input_and_never_optional(self):
        fixture, digest, _ = self.setup_payload()
        path = fixture.share / "source" / vm_runtime.LOCK
        path.chmod(0o600)
        path.write_text("{}")
        with patch.object(vm_runtime, "_command") as command, self.assertRaises(vm_runtime.vm_install.InstallError):
            vm_runtime.verify(fixture.share, digest, fixture.repo, "gallery")
        command.assert_not_called()
        payload_path = fixture.share / "payload.json"
        payload = json.loads(payload_path.read_text())
        del payload["sources"][vm_runtime.LOCK]
        payload["files"] = [entry for entry in payload["files"] if entry["payload"] != "source/" + vm_runtime.LOCK]
        path.unlink()
        for directory in (path.parent, path.parent.parent): directory.rmdir()
        raw = json.dumps(payload).encode()
        payload_path.write_bytes(raw)
        with patch.object(vm_runtime, "_command") as command, self.assertRaisesRegex(vm_runtime.RuntimeError, "lacks"):
            vm_runtime.verify(fixture.share, hashlib.sha256(raw).hexdigest(), fixture.repo, "gallery")
        command.assert_not_called()
        lock = vm_runtime.validate(json.loads((Path(__file__).parent / "guest-runtime.json").read_text()))
        self.assertEqual(len(lock["paths"]), 51)
        self.assertEqual(sum(item["narSize"] for item in lock["paths"].values()), 1492264592)

    def test_interpreter_identity_is_checked_before_execution(self):
        with patch.object(vm_runtime, "_command") as command, \
             patch.object(vm_runtime, "_file_hash", return_value="b" * 64), self.assertRaises(vm_runtime.RuntimeError):
            vm_runtime._python(lock_fixture())
        command.assert_not_called()

    def test_probe_never_escapes_rust_owned_group_and_reaps_on_timeout(self):
        from unittest.mock import Mock
        child = Mock(returncode=None)
        child.poll.return_value = None
        child.wait.side_effect = [0]
        with patch.object(vm_runtime.subprocess, "Popen", return_value=child) as spawn, self.assertRaises(vm_runtime.RuntimeError):
            vm_runtime._command(["/fixture/probe"], seconds=0)
        self.assertNotIn("start_new_session", spawn.call_args.kwargs)
        child.terminate.assert_called_once()
        child.wait.assert_called_once_with(timeout=3)
        child.kill.assert_not_called()

    def test_read_only_probe_is_bounded_and_rejects_nonzero_or_overflow(self):
        self.assertEqual(vm_runtime._command(["/usr/bin/printf", "fixture"], seconds=2), "fixture")
        for argv in (["/usr/bin/false"], ["/bin/sleep", "2"]):
            with self.assertRaises(vm_runtime.RuntimeError):
                vm_runtime._command(argv, seconds=0.1)
        with patch.object(vm_runtime, "OUTPUT_LIMIT", 2), self.assertRaises(vm_runtime.RuntimeError):
            vm_runtime._command(["/usr/bin/printf", "overflow"], seconds=2)


if __name__ == "__main__":
    unittest.main()
