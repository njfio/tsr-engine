"""Inert handoff tests: no process signals or workers are used."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[1]
PREFIX = '/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B '

def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT/'scripts'/f'{name}.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

class BoundaryTests(unittest.TestCase):
    def test_exact_paused_identity_required(self):
        m = load('m3-filtering-boundary')
        cmd = PREFIX + str(ROOT/'scripts/m3-following-native-expansion.py')
        for started, state, command, valid in [(m.STARTED,'T',cmd,True), (m.STARTED,'S',cmd,False), ('new process','T',cmd,False), (m.STARTED,'T','extra '+cmd,False)]:
            with patch.object(m.subprocess,'check_output',side_effect=[command,started,state]):
                if valid: m.require_paused()
                else:
                    with self.assertRaises(RuntimeError): m.require_paused()
    def test_low_disk_pauses_only_outer(self):
        m = load('m3-reserve-watch')
        with tempfile.TemporaryDirectory() as tmp:
            m.STATE = Path(tmp)/'watch'
            cmd = PREFIX + m.EXPECTED
            with patch.object(m.subprocess,'run',side_effect=[SimpleNamespace(stdout=cmd,returncode=0),SimpleNamespace(stdout=m.STARTED)]), patch.object(m.subprocess,'check_output',return_value='watch-start'), patch.object(m.shutil,'disk_usage',return_value=SimpleNamespace(free=m.THRESHOLD-1)), patch.object(m.os,'kill') as kill:
                m.main()
                kill.assert_called_once_with(m.PID,m.signal.SIGSTOP)
    def test_changed_identity_is_never_signalled(self):
        m = load('m3-reserve-watch')
        with tempfile.TemporaryDirectory() as tmp:
            m.STATE = Path(tmp)/'watch'
            with patch.object(m.subprocess,'run',side_effect=[SimpleNamespace(stdout=PREFIX+m.EXPECTED,returncode=0),SimpleNamespace(stdout='reused pid')]), patch.object(m.subprocess,'check_output',return_value='watch-start'), patch.object(m.shutil,'disk_usage',return_value=SimpleNamespace(free=0)), patch.object(m.os,'kill') as kill:
                m.main()
                kill.assert_not_called()

if __name__ == '__main__': unittest.main()
