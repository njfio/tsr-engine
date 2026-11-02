"""Harmless in-memory archive and lock validation; no network, Docker or candidate execution."""
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('prepare', Path(__file__).with_name('engine-prepare-image.py'))
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)


def archive(entries):
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode='w:gz') as output:
        for path, contents, kind in entries:
            item = tarfile.TarInfo(path)
            item.type = kind
            item.size = len(contents) if kind == tarfile.REGTYPE else 0
            item.linkname = '/outside' if kind == tarfile.SYMTYPE else ''
            output.addfile(item, io.BytesIO(contents) if kind == tarfile.REGTYPE else None)
    return buffer.getvalue()


class PreparationTests(unittest.TestCase):
    def unpack(self, data, checksum=None, remaining=10000):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            size = prepare.unpack(data, 'sample-1.0.0', checksum or prepare.sha(data), root, remaining)
            metadata = json.loads((root / 'sample-1.0.0/.cargo-checksum.json').read_text())
            return size, metadata

    def test_verified_archive_generates_file_checksums(self):
        data = archive([('sample-1.0.0/Cargo.toml', b'[package]\n', tarfile.REGTYPE)])
        size, meta = self.unpack(data)
        self.assertEqual(size, 10)
        self.assertEqual(meta, {'package': prepare.sha(data), 'files': {'Cargo.toml': prepare.sha(b'[package]\n')}})

    def test_checksum_failure_precedes_extraction(self):
        data = archive([('sample-1.0.0/Cargo.toml', b'ok', tarfile.REGTYPE)])
        with self.assertRaisesRegex(ValueError, 'checksum mismatch'):
            self.unpack(data, '0' * 64)

    def test_traversal_links_and_special_files_are_refused(self):
        for path, kind in [('sample-1.0.0/../../outside', tarfile.REGTYPE), ('/tmp/outside', tarfile.REGTYPE), ('sample-1.0.0/link', tarfile.SYMTYPE), ('sample-1.0.0/dev', tarfile.CHRTYPE), ('other/Cargo.toml', tarfile.REGTYPE)]:
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.unpack(archive([(path, b'', kind)]))

    def test_duplicates_supplied_metadata_and_expansion_are_refused(self):
        for entries in [[('sample-1.0.0/Cargo.toml', b'a', tarfile.REGTYPE), ('sample-1.0.0/Cargo.toml', b'b', tarfile.REGTYPE)], [('sample-1.0.0/.cargo-checksum.json', b'{}', tarfile.REGTYPE)]]:
            with self.assertRaises(ValueError):
                self.unpack(archive(entries))
        with self.assertRaisesRegex(ValueError, 'expanded'):
            self.unpack(archive([('sample-1.0.0/Cargo.toml', b'large', tarfile.REGTYPE)]), remaining=1)

    def test_decompression_limit_precedes_tar_parsing(self):
        data = archive([('sample-1.0.0/Cargo.toml', b'ok', tarfile.REGTYPE)])
        with patch.object(prepare, 'EXPANDED_LIMIT', 8), self.assertRaisesRegex(ValueError, 'decompressed'):
            self.unpack(data)

    def test_lock_controls_url_and_rejects_foreign_sources(self):
        base = {'cargo': True, 'image': 'sha256:' + 'a' * 64, 'protected': {'Cargo.lock': '[[package]]\nname="sample"\nversion="1.0.0"\nsource="' + prepare.REGISTRY + '"\nchecksum="' + 'a' * 64 + '"\n'}}
        self.assertEqual(prepare.packages(base)[0][2], 'https://static.crates.io/crates/sample/sample-1.0.0.crate')
        for old, new in [(prepare.REGISTRY, 'https://example.invalid'), ('name="sample"', 'name="../outside"'), ('version="1.0.0"', 'version="1.0.0/other"')]:
            altered = json.loads(json.dumps(base))
            altered['protected']['Cargo.lock'] = altered['protected']['Cargo.lock'].replace(old, new)
            with self.assertRaises(ValueError):
                prepare.packages(altered)

    def test_redirects_are_refused(self):
        with self.assertRaisesRegex(ValueError, 'redirects'):
            prepare.NoRedirect().redirect_request(None, None, 302, '', {}, 'https://example.invalid')


if __name__ == '__main__':
    unittest.main()
