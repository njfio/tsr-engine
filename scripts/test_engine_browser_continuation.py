"""Local derivative controls; no provider calls or executions, frozen original untouched."""
import copy,importlib.util,json,os,shutil,tempfile,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('web',Path(__file__).with_name('engine-browser-project.py'));w=importlib.util.module_from_spec(s);s.loader.exec_module(w)
@unittest.skipUnless(os.getenv('TSR_BROWSER_CONTINUATION_SOURCE'),'explicit frozen native input')
class Continuation(unittest.TestCase):
 def setUp(self):
  self.t=tempfile.TemporaryDirectory(dir=w.e.ROOT/'target',prefix='continuation-local-');self.addCleanup(self.t.cleanup)
  self.root=Path(self.t.name);shutil.copytree(os.environ['TSR_BROWSER_CONTINUATION_SOURCE'],self.root/'native');self.c=w.base.d.Native(self.root/'native')
  self.before=self.c.budget();self.view=self.c.cli('project-lifecycle-view');self.req=self.c.request()
  self.extension={'state_root':self.req['state_root'],'previous':self.view['admission'],'limits':{'model_calls':self.before['model_calls']+200,'model_tokens':self.before['tokens_charged']+8192000,'wall_ms':self.before['elapsed_wall_ms']+7200000,'patch_bytes':self.before['patch_bytes_charged']+104857600,'external_cost_microusd':self.before['external_cost_charged_microusd']},'tool_ms_ceiling':self.before['tool_ms_charged']+7200000,'reason':'Authored local extension control; no external authority'}
 def extend(self,x=None):return self.c.json_cli(('project-resource-extend',),x or self.extension,'--request')
 def test_retained_usage_replay_and_feedback(self):
  after=self.extend()
  for k in ('episode','model_calls','tokens_charged','patch_bytes_charged','measured_external_cost_microusd'):self.assertEqual(after[k],self.before[k])
  self.assertFalse(after['stopped']);self.assertFalse(after['exceeded']);self.assertEqual(self.c.cli('project-lifecycle-view')['task'],self.view['task'])
  self.assertEqual(self.extend()['model_calls'],after['model_calls'])
  self.req=self.c.request();completion=self.c.cli('project','acceptance','status');cp=w.checkpoint(self.req,completion,after)
  before_ids=[p['id'] for p in self.req['coding']['relational']['inputs']['parents']]
  unchanged={p['requirement']:p['meaning'] for p in self.req['coding']['relational']['inputs']['parents']}
  self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':unchanged})
  self.assertEqual(self.c.cli('project','cycle','request')['branch'],'review_required')
  self.req=self.c.request();cp=w.checkpoint(self.req,completion,self.c.budget())
  draft=w.feedback_draft(cp,self.req,completion,'authored-test-not-player',{'engaging_challenge':'Authored different mechanic guidance; no observed satisfaction claim'})
  self.c.json_cli(('project','acceptance','feedback'),draft['native_feedback']);fresh=w.base.d.Native(self.root/'native');new=fresh.request()
  self.assertEqual(before_ids,[p['id'] for p in new['coding']['relational']['inputs']['parents']]);self.assertEqual(new['coding']['snapshot'],self.req['coding']['snapshot'])
  self.assertEqual(new['coding']['relational']['inputs']['returns'],self.req['coding']['relational']['inputs']['returns']);self.assertFalse(fresh.cli('project','acceptance','status')['accepted'])
  self.assertEqual(fresh.cli('project','cycle','request')['branch'],'propose')
  shadow=(self.root/'native/shadow/events.sqlite').read_bytes();self.extend();self.assertEqual(shadow,(self.root/'native/shadow/events.sqlite').read_bytes())
  with self.assertRaises(Exception):w.feedback_draft(cp,new,fresh.cli('project','acceptance','status'),'stale',{})
 def test_stale_wrong_previous_and_below_usage_reject(self):
  for change in ({'state_root':'f'*64},{'previous':{**self.extension['previous'],'reason':'different'}},{'limits':{**self.extension['limits'],'model_calls':0}}):
   with self.assertRaises(Exception):self.extend({**self.extension,**change})
  self.assertEqual(self.c.budget()['model_calls'],self.before['model_calls'])
 def test_unsettled_blocks_extension(self):
  self.extend();request={'request_id':'authored-unknown','state_root':self.c.request()['state_root'],'envelope':'authored','model':'gpt-6-luna','tokens_upper':1}
  self.c.json_cli(('project','resource-reserve-unpriced'),request)
  ext=copy.deepcopy(self.extension);ext['previous']=self.c.cli('project-lifecycle-view')['admission'];ext['limits']['model_calls']+=1
  with self.assertRaises(Exception):self.extend(ext)
 def test_unknown_compute_blocks_extension(self):
  self.extend();request={'request_id':'authored-duration-unknown','state_root':self.c.request()['state_root'],'envelope':'authored','model':'gpt-6-luna','tokens_upper':1}
  self.c.json_cli(('project','resource-reserve-unpriced'),request)
  self.c.json_cli(('project-resource-settle',),{'request_id':request['request_id'],'prompt_tokens':1,'output_tokens':0,'duration_ns':None,'external_cost_microusd':0,'outcome':'measured'},'--usage')
  self.assertEqual(self.c.budget()['unknown_compute_requests'],1)
  ext=copy.deepcopy(self.extension);ext['previous']=self.c.cli('project-lifecycle-view')['admission'];ext['limits']['model_calls']+=1
  with self.assertRaises(Exception):self.extend(ext)
if __name__=='__main__':unittest.main()
