"""Malicious/truncated transfer fixtures, not native images or accepted reports."""
from pathlib import Path
import hashlib
import io
import json
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_transfer


class TransferContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.raw = self.root / "raw"
        self.raw.mkdir()
        self.archive = self.root / "return.tar"
        self.destination = self.root / "returned"
        self.payload = "a" * 64
        self.contents = {"fixture--light.png": b"synthetic encoded bytes", "fixture--dark.png": b"other fixture bytes",
                         "coverage.tsv": b"synthetic coverage\n", "guest-receipt.json": b'{"schema":1}'}
        for name, content in self.contents.items():
            (self.raw / name).write_bytes(content)

    def manifest(self):
        return {"schema": 1, "payload_sha256": self.payload, "files": {
            name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
            for name, data in self.contents.items()}}

    def archive_fixture(self, mutation):
        manifest = self.manifest()
        if mutation == "wrong-payload":
            manifest["payload_sha256"] = "b" * 64
        if mutation == "wrong-hash":
            manifest["files"]["fixture--light.png"]["sha256"] = "b" * 64
        if mutation == "missing-receipt":
            del manifest["files"]["guest-receipt.json"]
        if mutation == "invalid-size":
            manifest["files"]["fixture--light.png"]["bytes"] = True
        with tarfile.open(self.archive, "w", format=tarfile.USTAR_FORMAT) as tar:
            def add(name, content, kind=tarfile.REGTYPE):
                info = tarfile.TarInfo(name)
                info.size, info.type = len(content), kind
                if kind in {tarfile.SYMTYPE, tarfile.LNKTYPE}:
                    info.linkname = "../outside"
                tar.addfile(info, io.BytesIO(content))
            raw = json.dumps(manifest).encode()
            if mutation == "duplicate-json":
                raw = raw.replace(b'"schema": 1', b'"schema": 1, "schema": 1')
            add("return.json", raw)
            for name, data in self.contents.items():
                if mutation == "missing" and name.endswith("light.png"):
                    continue
                add(name, data)
            if mutation == "extra":
                add("extra--light.png", b"extra")
            if mutation == "duplicate":
                add("fixture--light.png", self.contents["fixture--light.png"])
            if mutation == "traversal":
                add("../outside", b"unsafe")
            if mutation == "absolute":
                add("/absolute", b"unsafe")
            if mutation == "symlink":
                add("alias--light.png", b"", tarfile.SYMTYPE)
            if mutation == "hardlink":
                add("alias--light.png", b"", tarfile.LNKTYPE)
            if mutation == "directory":
                add("dir", b"", tarfile.DIRTYPE)

    def test_round_trip_preserves_raw_bytes_and_requires_explicit_payload(self):
        packed = vm_transfer.pack(self.raw, self.archive, self.payload)
        returned = vm_transfer.unpack(self.archive, self.destination, self.payload)
        self.assertEqual(packed, returned)
        for name, data in self.contents.items():
            self.assertEqual((self.destination / name).read_bytes(), data)
        self.assertTrue((self.destination / "return.json").is_file())
        with self.assertRaises(vm_transfer.TransferError):
            vm_transfer.unpack(self.archive, self.destination, self.payload)
        with self.assertRaises(vm_transfer.TransferError):
            vm_transfer.inspect(self.archive, "b" * 64)

    def test_invalid_archives_fail_before_creating_candidate_directory(self):
        for mutation in ["wrong-payload", "wrong-hash", "missing-receipt", "invalid-size", "duplicate-json",
                         "missing", "extra", "duplicate", "traversal", "absolute", "symlink", "hardlink", "directory"]:
            with self.subTest(mutation=mutation):
                self.archive_fixture(mutation)
                with self.assertRaises(vm_transfer.TransferError):
                    vm_transfer.unpack(self.archive, self.destination, self.payload)
                self.assertFalse(self.destination.exists())
                self.assertFalse((self.root / "outside").exists())

    def test_truncated_archive_and_member_limit_fail_before_output(self):
        vm_transfer.pack(self.raw, self.archive, self.payload)
        original = self.archive.read_bytes()
        self.archive.write_bytes(original[:1500])
        with self.assertRaises((vm_transfer.TransferError, tarfile.TarError)):
            vm_transfer.unpack(self.archive, self.destination, self.payload)
        self.assertFalse(self.destination.exists())
        self.archive.write_bytes(original)
        with patch.object(vm_transfer, "FILE_LIMIT", 2):
            with self.assertRaises(vm_transfer.TransferError):
                vm_transfer.unpack(self.archive, self.destination, self.payload)
        self.assertFalse(self.destination.exists())

    def test_producer_links_unknown_files_and_missing_receipt_never_pack(self):
        extra = self.raw / "capture.json"
        extra.write_text("must not return a self-seal")
        with self.assertRaises(vm_transfer.TransferError):
            vm_transfer.pack(self.raw, self.archive, self.payload)
        self.assertFalse(self.archive.exists())
        extra.unlink()
        (self.raw / "fixture--light.png").unlink()
        (self.raw / "fixture--light.png").symlink_to(self.raw / "fixture--dark.png")
        with self.assertRaises(vm_transfer.TransferError):
            vm_transfer.pack(self.raw, self.archive, self.payload)
        self.assertFalse(self.archive.exists())

    def test_interrupted_copy_retains_partial_files_without_completion_manifest(self):
        vm_transfer.pack(self.raw, self.archive, self.payload)
        original = vm_transfer._digest
        def interrupted(handle, size, deadline, output=None):
            if output is not None:
                output.write(b"partial")
                raise vm_transfer.TransferError("injected copy deadline")
            return original(handle, size, deadline, output)
        with patch.object(vm_transfer, "_digest", interrupted):
            with self.assertRaisesRegex(vm_transfer.TransferError, "copy deadline"):
                vm_transfer.unpack(self.archive, self.destination, self.payload)
        self.assertTrue(self.destination.is_dir())
        self.assertFalse((self.destination / "return.json").exists())
        with self.assertRaises(vm_transfer.TransferError):
            vm_transfer.unpack(self.archive, self.destination, self.payload)

    def test_symlink_archive_destination_and_expired_deadline_are_rejected(self):
        vm_transfer.pack(self.raw, self.archive, self.payload)
        alias = self.root / "alias.tar"
        alias.symlink_to(self.archive)
        with self.assertRaises(vm_transfer.TransferError):
            vm_transfer.unpack(alias, self.destination, self.payload)
        with self.assertRaises(vm_transfer.TransferError):
            vm_transfer.inspect(self.archive, self.payload, deadline=0)
        linked = self.root / "linked"
        linked.symlink_to(self.root)
        with self.assertRaises(vm_transfer.TransferError):
            vm_transfer.unpack(self.archive, linked / "output", self.payload)
        self.assertFalse(self.destination.exists())


if __name__ == "__main__":
    unittest.main()
