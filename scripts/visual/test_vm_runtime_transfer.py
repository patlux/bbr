"""Offline runtime transfer contracts; no real store writes, SSH or VM."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import vm_runtime_transfer as transfer
import vm_runtime
import test_vm_runtime


class RuntimeTransferContracts(unittest.TestCase):
    def setUp(self):
        helper = test_vm_runtime.RuntimeContracts()
        self.fixture, self.digest, self.lock = helper.setup_payload()
        self.addCleanup(helper.doCleanups)
        self.workspace = self.fixture.root / 'transport'
        self.workspace.mkdir(mode=0o700)
        self.calls = []

    def execute(self, argv, stdout, stderr, **kwargs):
        self.calls.append(argv)
        self.assertEqual(argv[1], '--export')
        self.assertEqual(set(argv[2:]), set(self.lock['paths']))
        self.assertIsNone(kwargs['script'])
        stdout.write_bytes(b'inert Nix export fixture')
        stdout.chmod(0o600)
        stderr.write_bytes(b'')

    def prepare(self):
        with patch.object(transfer.vm_runtime, 'verify', return_value={}), \
                patch.object(transfer.shutil, 'which', return_value='/fixture/nix-store'):
            return transfer.prepare(self.fixture.share, self.digest, self.fixture.repo,
                                    'gallery', self.workspace, self.execute)

    def test_export_binds_lock_payload_and_bytes_and_checks_before_after(self):
        with patch.object(transfer.vm_runtime, 'verify', return_value={}) as verify, \
                patch.object(transfer.shutil, 'which', return_value='/fixture/nix-store'):
            value = transfer.prepare(self.fixture.share, self.digest, self.fixture.repo,
                                     'gallery', self.workspace, self.execute)
        self.assertEqual(verify.call_count, 2)
        self.assertEqual(value.payload_sha256, self.digest)
        self.assertEqual(value.sha256, hashlib.sha256(b'inert Nix export fixture').hexdigest())
        self.assertEqual(value.path.stat().st_mode & 0o777, 0o400)
        value.recheck()
        self.assertFalse(self.fixture.repo.exists())
        with self.assertRaises(Exception): self.prepare()

    def test_changed_linked_empty_oversized_exports_cannot_import(self):
        value = self.prepare()
        value.path.chmod(0o600)
        value.path.write_bytes(b'changed')
        with self.assertRaisesRegex(Exception, 'changed'): value.recheck()
        value.path.unlink()
        target = self.workspace / 'other'
        target.write_bytes(b'inert Nix export fixture')
        value.path.symlink_to(target)
        with self.assertRaises(Exception): value.recheck()
        value.path.unlink()
        for raw in (b'', b'oversized'):
            value.path.write_bytes(raw)
            with patch.object(transfer, 'ARCHIVE_LIMIT', 4), self.assertRaises(Exception):
                transfer.fingerprint(value.path)
            value.path.unlink()

    def test_import_is_fixed_offline_guest_only_and_checks_bytes_before_write(self):
        value = self.prepare()
        script = transfer.import_script(value)
        for required in ('VirtualMac', 'admin', '501', '--import', '--option substitute false',
                         '--option builders', 'shasum -a 256', value.sha256):
            self.assertIn(required, script)
        for forbidden in ('--realise', '--repair', '--no-check-sigs', 'require-sigs', 'curl ', 'nix-env', 'chown'):
            self.assertNotIn(forbidden, script)
        self.assertLess(script.index('host forbidden'), script.index('/bin/cat'))
        self.assertLess(script.index('shasum'), script.index('--import'))
        syntax = subprocess.run(['/bin/zsh', '-n'], input=script, text=True,
                                capture_output=True, timeout=5)
        self.assertEqual(syntax.returncode, 0, syntax.stderr)
        actual = subprocess.run(['/bin/zsh', '-f', '-c', script], input=b'',
                                capture_output=True, timeout=5)
        self.assertNotEqual(actual.returncode, 0)
        self.assertIn(b'host forbidden', actual.stderr)

    def test_stale_payload_and_failed_export_never_return_import_authority(self):
        with self.assertRaises(Exception):
            transfer.prepare(self.fixture.share, 'f'*64, self.fixture.repo, 'gallery',
                             self.workspace, self.execute)
        self.assertEqual(self.calls, [])
        def fail(*args, **kwargs): raise RuntimeError('export failed')
        with patch.object(transfer.vm_runtime, 'verify', return_value={}), \
                patch.object(transfer.shutil, 'which', return_value='/fixture/nix-store'), \
                self.assertRaisesRegex(RuntimeError, 'export failed'):
            transfer.prepare(self.fixture.share, self.digest, self.fixture.repo,
                             'gallery', self.workspace, fail)


if __name__ == '__main__': unittest.main()
