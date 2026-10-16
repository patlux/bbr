"""Portable cross-build tooling contracts; no compiler, SDK or VM required."""
import hashlib
import io
import json
import os
from pathlib import Path
import plistlib
import tarfile
import tempfile
import unittest
from unittest.mock import patch

from scripts.cross_macos import apple_sdk, build, source


class SDKTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.sdk = self.root / "installed/MacOSX.sdk"
        self.sdk.mkdir(parents=True)
        (self.sdk / "SDKSettings.plist").write_bytes(plistlib.dumps(
            {"Version": "27.0", "CanonicalName": "macosx27.0"}))
        (self.sdk / "header.h").write_text("typedef int Example;\n")
        os.link(self.sdk / "header.h", self.sdk / "alias.h")
        (self.sdk / "link.h").symlink_to("header.h")
        self.output = self.root / "export"
        with patch.object(apple_sdk, "installed_sdk", return_value=self.sdk), \
                patch.object(apple_sdk.subprocess, "check_output", return_value="Xcode 27.0\nBuild version 27A5252f\n"):
            self.manifest = apple_sdk.export("macosx", self.output, "/selected/Developer")
        self.archive = self.output / "MacOSX.sdk.tar.gz"
        self.manifest_path = self.output / "MacOSX.sdk.manifest.json"

    def test_hardlinked_headers_materialize_and_roundtrip(self):
        with tarfile.open(self.archive) as tar:
            self.assertTrue(tar.getmember("MacOSX.sdk/header.h").isfile())
            self.assertTrue(tar.getmember("MacOSX.sdk/alias.h").isfile())
        result = apple_sdk.unpack(self.archive, self.manifest_path, self.root / "unpacked")
        self.assertEqual(apple_sdk.entries(result), self.manifest["entries"])
        self.assertEqual(apple_sdk.verify(result)["entries"], 4)
        self.assertNotEqual((result / "header.h").stat().st_ino, (result / "alias.h").stat().st_ino)
        with self.assertRaises(ValueError):
            apple_sdk.unpack(self.archive, self.manifest_path, self.root / "unpacked")

    def test_modified_missing_extra_and_external_link_refused(self):
        for mutation in ("modified", "missing", "extra", "outside"):
            with self.subTest(mutation=mutation):
                sdk = apple_sdk.unpack(self.archive, self.manifest_path, self.root / mutation)
                if mutation == "modified":
                    (sdk / "header.h").write_text("different")
                elif mutation == "missing":
                    (sdk / "alias.h").unlink()
                elif mutation == "extra":
                    (sdk / "extra").write_text("extra")
                else:
                    (sdk / "link.h").unlink()
                    (sdk / "link.h").symlink_to("../../outside")
                with self.assertRaises(ValueError):
                    apple_sdk.verify(sdk)

    def test_manifest_traversal_duplicates_bad_types_refused(self):
        original = json.loads(self.manifest_path.read_text())
        mutations = [lambda m: m.update(name="../other.sdk"),
            lambda m: m["entries"].append(m["entries"][0]),
            lambda m: m["entries"][0].update(path="/outside"),
            lambda m: m["entries"][0].update(path="../outside"),
            lambda m: m["entries"][0].update(bytes=True),
            lambda m: m["entries"].append({"path": "link", "link": "../../outside"}),
            lambda m: m["entries"].append({"path": "header.h/child", "link": "../alias.h"})]
        for mutate in mutations:
            data = json.loads(json.dumps(original)); mutate(data)
            self.manifest_path.write_text(json.dumps(data))
            with self.assertRaises(ValueError):
                apple_sdk.load_manifest(self.manifest_path)
        self.manifest_path.write_text('{"schema":1,"schema":1}')
        with self.assertRaisesRegex(ValueError, "duplicate JSON"):
            apple_sdk.load_manifest(self.manifest_path)

    def test_tampered_archive_and_duplicate_members_refused_before_extract(self):
        self.archive.write_bytes(self.archive.read_bytes() + b"tamper")
        destination = self.root / "rejected"
        with self.assertRaisesRegex(ValueError, "archive hash"):
            apple_sdk.unpack(self.archive, self.manifest_path, destination)
        self.assertFalse(destination.exists())
        with tarfile.open(self.archive, "w:gz") as tar:
            for _ in self.manifest["entries"]:
                info = tarfile.TarInfo("MacOSX.sdk/header.h")
                info.size = 1; tar.addfile(info, io.BytesIO(b"x"))
        self.manifest["archive_sha256"] = apple_sdk.digest(self.archive)
        self.manifest_path.write_text(json.dumps(self.manifest))
        with self.assertRaisesRegex(ValueError, "member count"):
            apple_sdk.unpack(self.archive, self.manifest_path, destination)
        self.assertFalse(destination.exists())

    def test_export_requires_explicit_xcode(self):
        with patch.object(apple_sdk.sys, "platform", "darwin"):
            with self.assertRaisesRegex(ValueError, "explicit absolute"):
                apple_sdk.installed_sdk("macosx")


class BuildTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()

    def test_clean_cross_environment_and_quoted_linker(self):
        tools = self.root / "tools with space"
        (tools / "bin").mkdir(parents=True)
        for name in ("rustc", "cargo", "clang", "clang++", "ld64.lld", "llvm-ar", "llvm-readobj", "rcodesign"):
            (tools / "bin" / name).touch()
        sdk = self.root / "SDK with space"; sdk.mkdir()
        mbx = self.root / "mbx"; mbx.touch()
        env = {"RUSTC_WRAPPER": "wrong", "CARGO_TARGET_DIR": "wrong", "RUSTFLAGS": "wrong"}
        with patch.dict(os.environ, env):
            result = build.environment(self.root, tools, sdk, "27.0", mbx, self.root)
        self.assertFalse(set(env) & set(result))
        wrapper = (self.root / "link-macos.sh").read_text()
        self.assertIn("13.0,27.0", wrapper)
        self.assertIn('"$@"', wrapper)
        self.assertIn("'" + str(sdk) + "'", wrapper)
        self.assertNotIn("xcrun", wrapper)
        self.assertEqual(result["PYTHONDONTWRITEBYTECODE"], "1")

    def test_macho_rejects_wrong_platform_and_non_system_dependency(self):
        text = "Format: Mach-O arm64\nFileType: Executable\nPlatform: macos\nVersion: 13.0\nNeededLibraries [\n /usr/lib/libSystem.B.dylib\n]"
        build.validate_macho(text)
        for invalid in (text.replace("macos", "ios"), text.replace("arm64", "x86_64"),
                        text.replace("/usr/lib/", "/nix/store/")):
            with self.assertRaises(ValueError):
                build.validate_macho(invalid)

    def test_registry_comparison_preserves_versions_checksums(self):
        lock = {"package": [{"name": "example", "version": "1", "source": "registry+x", "checksum": "abc"},
                            {"name": "gpui", "version": "1", "source": source.GIT_SOURCE}]}
        self.assertEqual(source.registry(lock), [("example", "1", "registry+x", "abc")])

    def test_one_gpui_source_guard(self):
        (self.root / "bbr").mkdir()
        lock = 'version = 4\n[[package]]\nname = "example"\nversion = "1.0.0"\nsource = "registry+x"\nchecksum = "abc"\n'
        (self.root / "original-Cargo.lock").write_text(lock)
        (self.root / "bbr/Cargo.lock").write_text(lock)
        gpui = {"name": "gpui", "source": None, "manifest_path": str(self.root / "gpui/crates/gpui/Cargo.toml")}
        build.check_graph(self.root, {"packages": [gpui]})
        for packages in ([gpui, gpui], [dict(gpui, source=source.GIT_SOURCE)], []):
            with self.assertRaises(ValueError):
                build.check_graph(self.root, {"packages": packages})
        (self.root / "bbr/Cargo.lock").write_text(lock.replace('"abc"', '"changed"'))
        with self.assertRaisesRegex(ValueError, "registry"):
            build.check_graph(self.root, {"packages": [gpui]})

    def test_source_archive_sha_precedes_writes(self):
        archive = self.root / "source.tar.gz"; archive.write_bytes(b"invalid")
        dest = self.root / "snapshot"
        with self.assertRaisesRegex(ValueError, "SHA-256"):
            source.unpack(archive, "0" * 64, dest)
        self.assertFalse(dest.exists())

    def test_prepared_source_mutation_refused(self):
        (self.root / "bbr").mkdir()
        file = self.root / "bbr/input"; file.write_bytes(b"source")
        receipt = {"inputs": {"bbr/input": hashlib.sha256(b"source").hexdigest()}}
        (self.root / "prepared.json").write_text(json.dumps(receipt))
        build.verify_inputs(self.root)
        file.write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "drift"):
            build.verify_inputs(self.root)
        file.write_bytes(b"source")
        (self.root / "bbr/extra.pyc").touch()
        with self.assertRaisesRegex(ValueError, "inventory drift"):
            build.verify_inputs(self.root)

    def test_git_export_normalizes_modes(self):
        raw = io.BytesIO()
        with tarfile.open(fileobj=raw, mode="w") as tar:
            for name, mode in (("Cargo.toml", 0o664), ("script.sh", 0o775)):
                info = tarfile.TarInfo(name); info.size = 1; info.mode = mode
                tar.addfile(info, io.BytesIO(b"x"))
        with patch.object(source, "git", side_effect=[b"Cargo.toml\0script.sh\0", raw.getvalue()]):
            files = source.committed_files(self.root, "revision")
        self.assertEqual(files["Cargo.toml"][1], 0o644)
        self.assertEqual(files["script.sh"][1], 0o755)


if __name__ == "__main__":
    unittest.main()
