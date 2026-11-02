"""Offline lifecycle and isolated-interpreter regressions."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch
from m3_test_budget_inputs import isolated_budget_input

spec = importlib.util.spec_from_file_location('controller', Path(__file__).with_name('m3-progress-validation-controller.py'))
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)

class Lifecycle(unittest.TestCase):
    def test_M3_SMOKE_04_workers_and_resumed_coordinator_block(self):
        for row in ['123 S python /owned/scripts/m3-qualification.py', '123 S /owned/m3-qualification-engine-module-v2', '29246 S python /owned/scripts/m3-following-native-expansion.py']:
            with patch.object(c.subprocess,'check_output',return_value=row):
                with self.assertRaises(ValueError): c.require_idle()
        with patch.object(c.subprocess,'check_output',side_effect=['29246 Ts /opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B /Users/n/Documents/Codex/2026-10-14/task/tsr-engine-m3/scripts/m3-following-native-expansion.py', 'Wed Oct 14 21:19:49 2026', '']):
            c.require_idle()
    def test_M3_SMOKE_05_missing_or_wrong_coordinator_is_rejected(self):
        for replies in [['', ''], ['29246 Ts python /wrong/scripts/m3-following-native-expansion.py','Wed Oct 14 21:19:49 2026','']]:
            with patch.object(c.subprocess,'check_output',side_effect=replies):
                with self.assertRaises(ValueError): c.require_idle()
    def test_isolated_python_can_load_budget_without_credentials(self):
        with isolated_budget_input('progress') as (broker, ledger):
            code = "import importlib.util;from pathlib import Path;s=importlib.util.spec_from_file_location('b',"+repr(str(broker))+");b=importlib.util.module_from_spec(s);s.loader.exec_module(b);p=Path("+repr(str(ledger))+");f=p.open('r',encoding='utf8',newline='');print(b.smoke_budget(f))"
            result = subprocess.run([sys.executable,'-I','-c',code],capture_output=True,text=True,check=True)
        self.assertEqual(result.stdout.strip(),'0.208627')

if __name__ == '__main__': unittest.main()
