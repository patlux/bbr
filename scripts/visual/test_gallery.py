"""Probe completeness is not full supported-state or baseline acceptance."""
import fcntl
import struct
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import gallery
from gallery import inventory, validate

CASE = "gallery-00-state-00-standard-normal--light"
LABELS = ("Tokens", "default")


class GalleryProbeContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.output = Path(self.temp.name)
        self.page = f"{CASE}-page-00"
        self.png = self.output / f"{self.page}.png"
        self.png.write_bytes(b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR" + struct.pack(">II", 2400, 2000))
        self.marker = self.output / "coverage.tsv"
        self.row = f"{self.page}\t{CASE}\tTokens\tdefault\t0\t2\t2\n"
        self.marker.write_text(self.row)
        self.cases = {CASE: LABELS}

    def test_registry_inventory_is_filtered_without_copying_catalog(self):
        self.assertEqual(inventory(f"{CASE}\tTokens\tdefault\n", 0), self.cases)
        with self.assertRaises(ValueError):
            inventory(f"{CASE}\tTokens\tdefault\n", 999)

    def test_duplicate_and_malformed_inventory_fail(self):
        line = f"{CASE}\tTokens\tdefault\n"
        for value in ("", line + line, "../escape\tTokens\tdefault\n", "bad"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                inventory(value, None)

    def test_complete_probe_passes(self):
        self.assertEqual(validate(self.output, self.cases), {"cases": 1, "pages": 1})

    def test_lock_covers_timeout_cleanup(self):
        gate = self.output / "gate.lock"

        def interrupted(*args, **kwargs):
            with gate.open("a+") as contender:
                with self.assertRaises(BlockingIOError):
                    fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)
            raise subprocess.TimeoutExpired(args[0], kwargs["seconds"])

        with patch("gallery.with_gui_lock.lock_path", return_value=gate), \
             patch("gallery.run", side_effect=interrupted):
            with self.assertRaises(subprocess.TimeoutExpired):
                gallery.capture(Path("unused-binary"), self.output, 0, {})
        with gate.open("a+") as contender:
            fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_missing_completion_marker_fails(self):
        self.marker.unlink()
        with self.assertRaises(ValueError):
            validate(self.output, self.cases)

    def test_missing_or_extra_cases_fail(self):
        for cases in ({}, {**self.cases, CASE.replace("--light", "--dark"): LABELS}):
            with self.subTest(cases=cases), self.assertRaises(ValueError):
                validate(self.output, cases)

    def test_missing_overlapping_duplicate_and_mislabelled_ranges_fail(self):
        for row in (self.row.replace("\t0\t2\t2", "\t1\t2\t2"),
                    self.row.replace("\t0\t2\t2", "\t0\t1\t2"),
                    self.row + self.row, self.row.replace("Tokens", "Other"),
                    self.row.replace("page-00", "page-01")):
            self.marker.write_text(row)
            with self.subTest(row=row), self.assertRaises(ValueError):
                validate(self.output, self.cases)

    def test_wrong_dimensions_and_invalid_png_fail(self):
        for data in (b"", b"not a png", self.png.read_bytes()[:16] + struct.pack(">II", 1200, 1000)):
            self.png.write_bytes(data)
            with self.subTest(data=data), self.assertRaises(ValueError):
                validate(self.output, self.cases)

    def test_unexpected_file_and_symlink_fail(self):
        extra = self.output / "extra.png"
        extra.write_bytes(b"extra")
        with self.assertRaises(ValueError):
            validate(self.output, self.cases)
        self.png.unlink()
        self.png.symlink_to(extra)
        with self.assertRaises(ValueError):
            validate(self.output, self.cases)


if __name__ == "__main__":
    unittest.main()
