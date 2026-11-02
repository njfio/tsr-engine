"""Authored contract/lifecycle checks only. No provider or tool execution."""
import copy,itertools,json,tempfile,unittest
from pathlib import Path
from scripts.test_engine_criterion_return import h,m
from scripts.test_engine_pursuit_reflection import Fake,m as pursuit
shape=m.load('criterion_consistency_shape','engine-project-output.py')
correction=m.load('criterion_consistency_correction','engine-bounded-correction.py')

class Consistency(unittest.TestCase):
 def payload(self):
  binding={'review_id':'r'*64,'evidence_ids':['source'],'observed_receipts':[]}
  return {'binding':binding,'criteria':{'a':'Authored criterion A','b':'Authored criterion B'},'evidence':{'source':'Fixed authored source'},'response_schema':shape.review_schema(binding,{'a':'A','b':'B'})}
 def value(self,verdict,states):
  return {'review_id':'r'*64,'verdict':verdict,'findings':{k:{'finding':state,'basis':'static_source','evidence':['source'],'explanation':'Authored finding','limitations':'No human observations'} for k,state in zip(('a','b'),states)}}
 def test_all_verdict_findings_combinations_remain_strict(self):
  p=self.payload()
  for verdict in h.VERDICT_RULES:
   for states in itertools.product(('met','not_met','unknown'),repeat=2):
    v=self.value(verdict,states);original=copy.deepcopy(v)
    valid=(verdict=='abstain' or verdict=='accept' and set(states)=={'met'} or verdict=='reject' and 'not_met' in states or verdict=='unknown' and 'unknown' in states)
    if valid:self.assertEqual(h.validate_assessment(v,p['binding'],shape,p['response_schema']),v)
    else:
     with self.assertRaises(h.CriterionConsistencyFailure) as error:h.validate_assessment(v,p['binding'],shape,p['response_schema'])
     self.assertEqual(error.exception.code,'criterion_verdict_findings_inconsistent')
     with tempfile.TemporaryDirectory() as folder:
      b=correction.Budget(Path(folder)/'budget',{},2)
      with self.assertRaisesRegex(ValueError,'not admitted or unsettled'):
       b.feedback('criterion_review','original',{'value':v,'receipt':{'status':'settled','usd':'0.001'}},'failed',error.exception.diagnostic)
      self.assertEqual(b.events(),[])
    self.assertEqual(v,original)
 def test_binding_or_evidence_errors_not_coached_as_consistency(self):
  p=self.payload();v=self.value('accept',('met','met'));v['review_id']='foreign'
  with self.assertRaises(ValueError) as error:h.validate_assessment(v,p['binding'],shape,p['response_schema'])
  self.assertNotIsInstance(error.exception,h.CriterionConsistencyFailure)
  v=self.value('accept',('met','met'));v['findings']['a']['evidence']=['foreign']
  with self.assertRaises(ValueError) as error:h.validate_assessment(v,p['binding'],shape,p['response_schema'])
  self.assertNotIsInstance(error.exception,h.CriterionConsistencyFailure)
 def test_consistent_reject_unknown_abstain_replay_without_reassessment(self):
  for verdict in ('accept','reject','unknown','abstain'):
   a=Fake(verdict=verdict);p=self.payload();original=copy.deepcopy(p)
   first=pursuit._stage(a,'criterion_review',h.CONSISTENT_SYSTEM,p,validator=h.validate_assessment)
   self.assertEqual(pursuit._stage(a,'criterion_review',h.CONSISTENT_SYSTEM,p,validator=h.validate_assessment),first)
   self.assertEqual(a.calls,1);self.assertFalse(a.applies);self.assertEqual(p,original)
   altered=copy.deepcopy(p);altered['evidence']['source']='different source'
   with self.assertRaisesRegex(ValueError,'payload drift'):pursuit._stage(a,'criterion_review',h.CONSISTENT_SYSTEM,altered,validator=h.validate_assessment)
   self.assertEqual(a.calls,1)
 def test_inconsistent_response_cannot_be_retried_on_restart(self):
  a=Fake();p=self.payload();checked=a.checked;retained=[]
  def inconsistent(name,body,validate,review=False):
   def fail(v):
    v['verdict']='unknown';retained.append(copy.deepcopy(v));return validate(v)
   return checked(name,body,fail,review)
  a.checked=inconsistent
  with self.assertRaises(h.CriterionConsistencyFailure):pursuit._stage(a,'criterion_review',h.CONSISTENT_SYSTEM,p,validator=h.validate_assessment)
  before=copy.deepcopy(a.done)
  with self.assertRaisesRegex(ValueError,'no redispatch'):pursuit._stage(a,'criterion_review',h.CONSISTENT_SYSTEM,p,validator=h.validate_assessment)
  self.assertEqual(a.calls,1);self.assertEqual(a.done,before);self.assertFalse(a.applies);self.assertEqual(retained[0]['verdict'],'unknown')
 def test_stop_and_unknown_effect_never_redispatch(self):
  p=self.payload();a=Fake();a.stopped=True
  with self.assertRaisesRegex(ValueError,'Stopped'):pursuit._stage(a,'criterion_review',h.CONSISTENT_SYSTEM,p,validator=h.validate_assessment)
  self.assertEqual(a.calls,0)
  a.stopped=False;a.unknown=True
  with self.assertRaisesRegex(ValueError,'Unknown response'):pursuit._stage(a,'criterion_review',h.CONSISTENT_SYSTEM,p,validator=h.validate_assessment)
  with self.assertRaisesRegex(ValueError,'no redispatch'):pursuit._stage(a,'criterion_review',h.CONSISTENT_SYSTEM,p,validator=h.validate_assessment)
  self.assertEqual(a.calls,1);self.assertFalse(a.applies)
 def test_ordinary_capture_publishes_rules_and_dispatches_exact_validator(self):
  import types
  from unittest.mock import patch
  frame={'requirements':{'a':'Authored criterion'},'publication':'p','requirements_revision':'r','snapshot':'s','evidence':{'index.html':'source'},'coverage':{'a':['index.html']},'prerequisites':{'a':{'kind':'static_semantic'}}}
  context={'frame':frame,'current':None,'state_root':'root','binding':'binding','published_context':{'version':'criterion-published-context@v1','guidance':{'a':'Authored current condition'},'provenance':{'publication':'p','requirements_revision':'r','current_source':'s','status':'current_publication','grants_authority':False,'review_carried':False}}}
  done={};applies=[];seen=[]
  def op(name,fn):
   if name not in done:done[name]=fn()
   return done[name]
  def stage(adapter,name,system,payload,**kwargs):
   seen.append(copy.deepcopy(payload));self.assertIs(kwargs['validator'],h.validate_assessment)
   self.assertEqual(system,h.CUMULATIVE_SYSTEM);self.assertEqual(payload['verdict_consistency'],h.VERDICT_RULES)
   self.assertIn('At least one finding must be unknown',payload['response_schema']['properties']['verdict']['description'])
   value={'review_id':payload['binding']['review_id'],'verdict':'unknown','findings':{'a':{'finding':'unknown','basis':'unknown','evidence':['index.html'],'explanation':'Authored uncertainty','limitations':'No human evidence'}}}
   kwargs['validator'](value,payload['binding'],shape,payload['response_schema'])
   return {'request':'authored-review','value':value}
  def apply(*args):applies.append(args);return 'authored-admission'
  adapter=types.SimpleNamespace(admission={'criterion_return_profile':{'version':'public-criterion-return@v1'}},controller=types.SimpleNamespace(cli=lambda *args:copy.deepcopy(context),json_cli=apply),native=lambda *args:{'state_root':'root','source_snapshot':'s','scope':'scope'},project=lambda v:({'coding':{'snapshot':'s','files':[{'path':'index.html','source':'source','editable':True}]}},{}),read=lambda:([],{},done),recovery_sha=correction.digest,recovery_load=lambda name,file:shape if file=='engine-project-output.py' else types.SimpleNamespace(_stage=stage),op=op,narrative_limit=131072)
  with patch.object(h,'verify',lambda *args:None):self.assertEqual(h.reflect(adapter),'authored-admission')
  captures=[v for k,v in done.items() if k.endswith('_input') and 'payload' in v]
  self.assertEqual(captures[0]['contract_version'],'criterion-evidence@v7');self.assertEqual(captures[0]['payload'],seen[0]);self.assertEqual(len(applies),1)
 def test_versioned_instruction_preserves_existing_captures(self):
  self.assertEqual(h.capture_system({'contract_version':'criterion-evidence@v3'}),h.CONTEXT_SYSTEM)
  self.assertEqual(h.capture_system({'contract_version':'criterion-evidence@v4'}),h.CONSISTENT_SYSTEM)
  self.assertNotIn('verdict_consistency',h.CONTEXT_SYSTEM)
  self.assertIn('verdict_consistency',h.CONSISTENT_SYSTEM)
if __name__=='__main__':unittest.main()
