"""Authored contract controls; captured failure is never repaired or admitted."""
import copy,json,unittest
from pathlib import Path
from test_engine_browser_adapter import m as a
class ReturnSchema(unittest.TestCase):
 def request(self):
  return {'state_root':'1'*64,'coding':{'kind':'browser_project@v1','cases':[{'id':'browser_smoke'}],
   'files':[{'path':'app.js','source_digest':'2'*64,'editable':True}],
   'relational':{'inputs':{'snapshot':'3'*64,'returns':[{'receipt':'4'*64,'commitment':'5'*64,'verdict':'passed','observations':[{'case_id':'browser_smoke','passed':True,'exit':0}]}]}}}}
 def test_exact_source_bound_shape_and_valid_empty_unknown(self):
  r=self.request();s=a.e.return_evidence.schema(r,narrative_bytes=131072);p=s['properties']
  self.assertEqual(p['observed']['items'],{'enum':['0']});self.assertEqual(p['unknown']['type'],'array')
  b={'binding':p['binding']['const'],'observed':['0'],'inferred':[],'unknown':[]}
  self.assertEqual(a.e.return_evidence.validate(b,r)['resolved_observations'][0]['case_id'],'browser_smoke')
  for key,value in [('observed',['browser_smoke']),('observed',['0','0']),('unknown','not an array'),('binding','f'*64)]:
   bad=copy.deepcopy(b);bad[key]=value
   with self.subTest(key=key,value=value),self.assertRaises(ValueError):a.e.return_evidence.validate(bad,r)
 def test_exact_source_pairs_and_stale_rejection(self):
  r=self.request();s=a.e.return_evidence.schema(r);item=s['properties']['inferred']['items']['oneOf'][0]
  self.assertEqual(item['properties']['digest'],{'const':'2'*64})
  b={'binding':s['properties']['binding']['const'],'observed':[],'inferred':[{'path':'app.js','digest':'2'*64,'kind':'source_prediction','statement':'Authored source inference'}],'unknown':['No gameplay claim']}
  a.e.return_evidence.validate(b,r)
  b['inferred'][0]['digest']='6'*64
  with self.assertRaises(ValueError):a.e.return_evidence.validate(b,r)
 def test_empty_catalog_publishes_valid_empty_array_items(self):
  r=self.request();r['coding']['relational']['inputs']['returns']=[];r['coding']['files']=[]
  p=a.e.return_evidence.schema(r)['properties']
  self.assertIs(p['observed']['items'],False);self.assertIs(p['inferred']['items'],False)
 def test_captured_failure_remains_invalid(self):
  p=Path('target/browser-game-session-live7/adapter/output-0/failed-response-1/content.bin')
  if not p.exists():self.skipTest('optional local captured evidence')
  b=json.loads(p.read_bytes())['alternatives'][0]['return_basis']
  self.assertEqual(b['observed'],['browser_smoke']);self.assertIsInstance(b['unknown'],str)
  # Exact original response is retained. This assertion does not turn its wrong citation into id 0.
if __name__=='__main__':unittest.main()
