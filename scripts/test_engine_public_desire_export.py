"""Authored local controls with real native state; zero inference/target commands."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

def load(name,file):
 s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
x=load('public_export_test','engine-public-desire-export.py');d=load('public_native_test','engine-b91-driver.py');suite=load('public_suite_test','engine-repair-suite.py')
class Export(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.tmp=tempfile.TemporaryDirectory();cls.root=Path(cls.tmp.name);scenario=suite.select('singleton-clamp');scenario.prepare(cls.root/'frozen');(cls.root/'project').mkdir();c=d.Native(cls.root/'project');cls.c=c
  c.cli('project','init','--spec',str(cls.root/'frozen/scope.json'),'--source',str(cls.root/'frozen/source'),'--requirements',str(cls.root/'frozen/requirements.json'))
  c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':1,'reason':'Authored zero-execution export test','non_tool':{'model_calls':2,'model_tokens':81920,'wall_ms':3600000,'patch_bytes':2048,'external_cost_microusd':0}},'--admission')
  c.json_cli(('project','acceptance','feedback'),{'requirements_revision':c.cli('project','acceptance','status')['requirements_revision'],'decision_record':None,'guidance':scenario.REQUIREMENTS})
  d.e.freeze_origin(cls.root/'frozen',c,cls.root/'origin.json',scenario_id='singleton-clamp');cls.origin=json.loads((cls.root/'origin.json').read_text());cls.request=c.request()
 @classmethod
 def tearDownClass(cls):cls.tmp.cleanup()
 def project(self,r=None,pins=None):
  r=r or self.request;return x.project(r,self.origin,pins,expected_request_sha256=x.e.sha(r))
 def value(self,r):
  i=r['coding']['relational']['inputs'];p=i['parents'][0]
  return {'alternatives':[{'public_desire':{'binding':r['coding']['relational']['binding'],'identity':p['id'],'revises':p.get('revision'),'parents':[q['id'] for q in i['parents']],'meaning':'Authored fixture clarification only, not generated development.','change':'clarification','basis':{'kind':'unknown'},'generator':'gpt-6-luna/public-desire-generator'}}]}
 def test_unchanged_allowlist_private_exclusion_and_stale(self):
  c=self.project();self.assertEqual(c,self.project());r=copy.deepcopy(self.request);r['private_operator_notes']='PRIVATE_SENTINEL';r['coding']['relational']['public_desire_contract']['private']='PRIVATE_SENTINEL'
  self.assertNotIn('PRIVATE_SENTINEL',json.dumps(self.project(r)))
  self.assertNotIn('tests/contract.rs',json.dumps(c));self.assertEqual(c['evidence']['observed'],[]);self.assertEqual(c['authority']['executions'],0)
  with self.assertRaisesRegex(ValueError,'stale request'):x.project(r,self.origin,expected_request_sha256=x.e.sha(self.request))
 def test_wrong_source_parent_evidence_and_handles(self):
  for mutate in (lambda r:r['coding'].__setitem__('snapshot','0'*64),lambda r:r['coding']['relational']['inputs']['parents'][0].__setitem__('meaning','invented'),lambda r:r['coding']['relational']['inputs']['returns'].append({'receipt':'0'*64}),lambda r:r['coding']['files'][0].__setitem__('source_digest','bad')):
   r=copy.deepcopy(self.request);mutate(r)
   with self.assertRaises(ValueError):self.project(r)
 def test_bad_references_authority_and_honest_abstention(self):
  self.assertEqual(x.bind({'abstain':'No new insight justified'},self.request,'develop'),{'abstain':'No new insight justified'})
  for key,value in (('binding','0'*64),('identity','0'*64),('revises','0'*64),('basis',{'kind':'observation','receipt':'0'*64}),('authority',True)):
   v=self.value(self.request);v['alternatives'][0]['public_desire'][key]=value
   with self.assertRaises(ValueError):x.bind(v,self.request,'develop')
 def test_nonblank_abstention_and_forecast_preservation(self):
  with self.assertRaises(ValueError):x.bind({'abstain':'   '},self.request,'develop')
  alternatives=[{'edits':[{'path':'src/lib.rs','replacement':self.origin['initial_source']+'\n// authored variant '+str(i)+'\n'}],'rationale':'Authored unexecuted forecast control','common_event_probability':p} for i,p in enumerate((0.25,0.5))]
  response=x.bind({'alternatives':alternatives},self.request,'next_proposal')
  self.assertEqual(response['common_event_forecasts'],{'0':0.25,'1':0.5})
 def test_real_native_admit_restart_next_preview_and_content_pin(self):
  with tempfile.TemporaryDirectory() as tmp:
   path=Path(tmp)/'project';shutil.copytree(self.root/'project',path);c=d.Native(path);r=c.request();before=c.cli('project','config');response=x.bind(self.value(r),r,'develop')
   preview=c.json_cli(('project','cycle','preview'),response,'--response');v=preview['cycle']
   review={'binding':v['binding'],'response_digest':preview['response_digest'],'state_root':r['state_root'],'source_snapshot':v['source_snapshot'],'reviewer':'independent-authored-fixture','generator':'gpt-6-luna/public-desire-generator','decision':'approve'}
   result=c.json_cli(('project','cycle','advance'),{'binding':v['binding'],'review':review,'response':response})
   self.assertTrue(result['review_required']);self.assertFalse(result['evaluation']['committed'])
   rev=c.json_cli(('project','desire-admit'),{'state_root':c.request()['state_root'],'proposal':result['proposal_record'],'index':0,'reviewer':'independent-authored-fixture','note':'PRIVATE_SENTINEL'})
   r=c.request();entries=r['coding']['relational']['inputs']['parents'];entry=next(p for p in entries if p.get('revision')==rev);pins={rev:x.e.sha(entry)}
   ctx=self.project(r,pins);self.assertNotIn('PRIVATE_SENTINEL',json.dumps(ctx));self.assertEqual(ctx['acceptance_parents'],self.origin['parents']);self.assertEqual(before,c.cli('project','config'))
   self.assertEqual(ctx,self.project(d.Native(path).request(),pins))
   tampered=copy.deepcopy(r);next(p for p in tampered['coding']['relational']['inputs']['parents'] if p.get('revision')==rev)['meaning']='unreviewed replacement'
   with self.assertRaises(ValueError):self.project(tampered,pins)
   proposed={'alternatives':[{'edits':[{'path':'src/lib.rs','replacement':self.origin['initial_source']+'\n// authored unexecuted preview\n'}],'rationale':'Authored native path control; not a model possibility.'}]}
   nxt=x.bind(proposed,r,'next_proposal');c.json_cli(('project','cycle','preview'),nxt,'--response')
   self.assertEqual(before,c.cli('project','config'));self.assertEqual(c.request()['state_root'],r['state_root'])
if __name__=='__main__':unittest.main()
