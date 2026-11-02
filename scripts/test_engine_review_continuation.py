"""Read-only retained-response recovery controls; no generation or execution."""
import copy,importlib.util,json,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('m',Path(__file__).with_name('engine-browser-review-continuation.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class RetainedProposal(unittest.TestCase):
 def setUp(self):
  p=m.e.ROOT/'target/harness-feedback-review-continuation-6/prefix.json'
  if not p.exists():self.skipTest('exact local failed-zero fixture unavailable')
  self.p=m.read(p);self.request=m.d.Native(p.parent/'native').request();self.a=object.__new__(m.Adapter);self.a.prefix=self.p;self.a.native=lambda *_:copy.deepcopy(self.p['source']);self.a.controller=type('Native',(),{'request':lambda _:copy.deepcopy(self.request)})()
 def test_original_association_and_no_context_substitution(self):
  self.a.verify_owned_output(self.p['generated_id'])
  self.a.native=lambda *_:{**self.p['source'],'state_root':'0'*64}
  with self.assertRaises(ValueError):self.a.verify_owned_output(self.p['generated_id'])
 def test_inherited_body_and_admission_cannot_be_rewritten(self):
  self.a.prefix=copy.deepcopy(self.p);self.a.prefix['inherited']['generated_'+self.p['generated_id']]['response']['proposals']=[]
  with self.assertRaises(ValueError):self.a.verify_owned_output(self.p['generated_id'])
  self.a.prefix=copy.deepcopy(self.p);self.a.prefix['original_admission']['max_calls']=1
  with self.assertRaises(ValueError):self.a.verify_owned_output(self.p['generated_id'])
 def test_reuse_is_exact_and_not_generation(self):
  done={};self.a.read=lambda:('',{},done);self.a.op=lambda key,fn:done.setdefault(key,fn())
  first=self.a.proposal(self.p['source']);second=self.a.proposal(self.p['source'])
  self.assertEqual(m.a.loop.Cycle.measured(first,'response',32768,m.a.base.capacity.INPUT_TOKENS),{**self.p['proposal'],'response':self.p['materialized_response']});self.assertEqual(first,second);self.assertEqual(list(done),['historical_proposal_reuse']);self.assertFalse(done['historical_proposal_reuse']['new_provider_call'])
 def test_rematerialization_rejects_source_edit_metadata_and_mapping_substitution(self):
  changed=copy.deepcopy(self.request);next(x for x in changed['coding']['files'] if x['editable'])['source']+=' changed'
  with self.assertRaises(ValueError):m.materialize(self.p,changed)
  p=copy.deepcopy(self.p);p['inherited']['bound_output_'+p['generated_id']]['model_content']['alternatives'][0]['edits'][0]['inspection_handle']='model-id'
  with self.assertRaises(ValueError):m.materialize(p,self.request)
  self.a.prefix=copy.deepcopy(self.p);self.a.prefix['materialization']['materialized_response_sha256']='0'*64
  with self.assertRaises(ValueError):self.a.verify_owned_output(self.p['generated_id'])
 def test_remaining_authority_and_no_deadline_reset(self):
  r=m.e.ROOT/'target/harness-feedback-review-continuation-6';ad=m.read(r/'adapter-admission.json');grant=m.read(r/'scope-grant.json')
  self.assertEqual(ad['max_calls']+self.p['used_calls'],self.p['max_calls']);self.assertEqual(grant['session_cap_microusd']+self.p['used_microusd'],self.p['cap_microusd'])
  self.assertEqual(ad['feedback_cycle']['max_continuations']+self.p['used_continuations'],self.p['original_admission']['feedback_cycle']['max_continuations']);self.assertEqual(ad['expires_at'],self.p['expires_at'])
if __name__=='__main__':unittest.main()
