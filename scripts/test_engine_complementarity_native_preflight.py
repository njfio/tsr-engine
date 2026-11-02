import importlib.util,json,tempfile,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('prep',Path(__file__).with_name('engine-complementarity-native-preflight.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Preparation(unittest.TestCase):
 def test_exact_origins_no_invented_predicates_or_returns(self):
  v=m.packet();old=m.p.source()
  for k in ('parents','records','observations','provenance'):self.assertEqual(v[k],old[k])
  self.assertIsNone(v['task_acceptance']);self.assertEqual(v['generated_guidance'],[])
  self.assertEqual(v['authority'],{'external_calls':0,'executions':0,'adoptions':0})
  self.assertEqual(v,m.packet())
 def test_native_mapping_keeps_meanings_and_attribution(self):
  v=m.bootstrap();original=m.p.source();self.assertEqual([p['meaning'] for p in v['parents']],[p['meaning'] for p in original['parents']]);self.assertEqual(v['acceptance_references'],[])
  self.assertEqual(len(v['records']),8)
  for r,source in zip(v['records'],original['records']):
   self.assertEqual(json.loads(r['content'])['assertion'],source['assertion']);self.assertEqual(r['uncertainty'],source['uncertainty']);self.assertFalse(json.loads(r['attribution'])['provenance']['human_observation'])
 def test_content_pin_rejects_stale_source(self):
  with tempfile.TemporaryDirectory() as tmp:
   f=Path(tmp)/'source.json';f.write_text(json.dumps(m.p.source())+' ')
   with self.assertRaisesRegex(ValueError,'frozen source'):m.p.source(f)
 def test_preparation_never_replaces_existing_packet(self):
  with tempfile.TemporaryDirectory() as tmp:
   out=Path(tmp)/'new';r=m.write(out);self.assertTrue(r['native_ready']);self.assertEqual(r['dispatches'],0)
   before=(out/'packet.json').read_bytes()
   with self.assertRaises(FileExistsError):m.write(out)
   self.assertEqual(before,(out/'packet.json').read_bytes())
if __name__=='__main__':unittest.main()
