import importlib.util
from pathlib import Path
import unittest
import tempfile
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('ready', Path(__file__).with_name('engine-collection-ready.py'))
ready = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ready)


class ReadinessTests(unittest.TestCase):
    def test_window_uses_bytes_and_excludes_protected_suffix(self):
        prefix = '// é\n'
        body = '        let names: Vec<String> = match value { _ => vec![] };\n'
        suffix = '        let known = lookup();\n#[test] fn frozen() {}'
        text = prefix + body + suffix
        window = ready.window(text)
        self.assertEqual(text.encode()[window['start']:window['end']], body.encode())

    def test_window_refuses_oversized_or_test_content(self):
        for body in ('x' * 2049, '#[test] fn forbidden() {}'):
            with self.assertRaises(ValueError):
                ready.window('        let names: Vec<String> = match ' + body + '        let known = ')

    def test_cas_checks_admitted_identity_before_use(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'blob'
            path.write_bytes(b'changed')
            with patch.object(ready.subprocess, 'check_output', return_value=b'wrong\n'):
                with self.assertRaises(ValueError):
                    ready.cas(path, 'admitted')
            with patch.object(ready.subprocess, 'check_output', return_value=b'admitted\n'):
                self.assertEqual(ready.cas(path, 'admitted'), b'changed')

    def test_output_cannot_be_original_repository(self):
        with self.assertRaises(ValueError):
            ready.prepare(Path('/tmp/not-admitted-target'))


if __name__ == '__main__':
    unittest.main()
