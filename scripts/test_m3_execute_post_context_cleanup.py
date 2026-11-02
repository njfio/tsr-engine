import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('cleanup_v3', Path(__file__).with_name('m3-execute-post-context-cleanup.py'))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

class IdentityChecks(unittest.TestCase):
    def test_inode_change_refused(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / 'target'
            p.mkdir()
            s = p.stat()
            with self.assertRaises(RuntimeError):
                m.check_identity(p, dict(device=s.st_dev, inode=s.st_ino + 1))
            self.assertTrue(p.exists())

    def test_symlink_refused(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / 'target'
            p.symlink_to(d)
            s = Path(d).stat()
            with self.assertRaises(RuntimeError):
                m.check_identity(p, dict(device=s.st_dev, inode=s.st_ino))

    def test_exact_identity_allowed(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)
            s = p.stat()
            m.check_identity(p, dict(device=s.st_dev, inode=s.st_ino))

    def test_m3_cleanup_01_birth_uses_manifest_precision(self):
        self.assertTrue(m.valid_birth(1792116863.1211278, '2026-10-16T02:14:23.121128+00:00', 1792110000))
        self.assertFalse(m.valid_birth(1792116863.1211278, '2026-10-16T02:14:24.121128+00:00', 1792110000))
        self.assertFalse(m.valid_birth(1792116863.1211278, '2026-10-16T02:14:23.121128+00:00', 1792120000))
