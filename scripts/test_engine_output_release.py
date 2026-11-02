"""Offline launch compatibility checks; no provider or worker is invoked."""
import runpy
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parent
G=runpy.run_path(str(ROOT/'engine-output-release.py'))
class Release(unittest.TestCase):
 def test_all_old_live_commands_fail_before_missing_inputs(self):
  for name in sorted(G['MODE_RUN']):
   with self.subTest(name=name):
    result=subprocess.run([sys.executable,str(ROOT/name),'run'],capture_output=True,text=True)
    self.assertNotEqual(result.returncode,0)
    self.assertIn('OUTPUT_BOUNDARY_INCOMPATIBLE',result.stderr)
 def test_abbreviated_live_flags_are_not_accepted(self):
  for name,arg in [('engine-directions.py','--synthetic-admiss=x'),('engine-run.py','--approved-liv')]:
   result=subprocess.run([sys.executable,str(ROOT/name),arg],capture_output=True,text=True)
   self.assertNotEqual(result.returncode,0)
   self.assertIn('usage:',result.stderr)
 def test_options_before_mode_cannot_bypass(self):
  for name in G['MODE_RUN']:
   with self.assertRaisesRegex(RuntimeError,'OUTPUT_BOUNDARY_INCOMPATIBLE'):
    G['check_cli'](name,['--directory','missing','run'])
 def test_direct_and_simulator_paths_blocked(self):
  for name,args in [(n,['/not-a-state']) for n in G['DIRECT']]+[('engine-run.py',['--approved-live']),('engine-directions.py',['--synthetic-admission=x']),('engine-directions.py',['--synthetic-admission','x'])]:
   with self.assertRaisesRegex(RuntimeError,'OUTPUT_BOUNDARY_INCOMPATIBLE'):G['check_cli'](name,args)
 def test_offline_modes_and_preserved_terminal_are_not_changed(self):
  for name in G['MODE_RUN']:
   for mode in ['prepare','inspect','check','status','--help']:
    G['check_cli'](name,[mode])
  with tempfile.TemporaryDirectory() as d:
   p=Path(d)/'terminal.json';p.write_bytes(b'{"historical":true}')
   for name in G['DIRECT']:G['check_cli'](name,[d])
   self.assertEqual(p.read_bytes(),b'{"historical":true}')
 def test_browser_opt_in_still_requires_complete_admission(self):
  G['check_cli']('engine-native-cycle.py',['--browser-v1','run'])
  result=subprocess.run([sys.executable,str(ROOT/'engine-native-cycle.py'),'--browser-v1','run'],capture_output=True,text=True)
  self.assertNotEqual(result.returncode,0)
  self.assertIn('required',result.stderr)
 def test_inventory_installed_and_active_runner_not_disabled(self):
  for name in G['MODE_RUN']|G['DIRECT']|{'engine-directions.py'}:
   self.assertIn("['check_cli'](__file__,sys.argv[1:])",(ROOT/name).read_text())
  G['check_cli']('engine-exploration-validation.py',['new-state'])
if __name__=='__main__':unittest.main()
