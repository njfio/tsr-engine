"""Authored controller fault boundaries; no model/native execution."""
import unittest
from scripts.test_engine_browser_adapter import m
class Restart(unittest.TestCase):
 def test_recovered_artifact_abstention_terminates_without_callback(self):
  whole={'deliberation':{'development':None,'artifact':{'proposals':[{'kind':'edit_set'}]},'artifact_dependency':'independent'}}
  class Harness:
   def __init__(self):self.done={'compound_0_artifact_outcome':{'response':whole['deliberation']['artifact'],'judgment':{'decision':'abstain'},'advance':None}}
   def read(self):return [],{},self.done
   def record_component(self,name,value):
    if name in self.done:assert self.done[name]==value
    self.done[name]=value;return value
   def terminal(self,**fields):return fields
   def current(self):raise AssertionError('no new context/model/effect after settled abstention')
  h=Harness();result=m.loop.Cycle.deliberation(h,'0',whole,{})
  self.assertEqual(result['status'],'review_abstain');self.assertFalse(result['artifact_executed'])

class Provenance(unittest.TestCase):
 def test_historical_binding_is_attention_not_cycle_and_tampering_fails(self):
  import copy
  helper=m.load('historical_component_fixture','engine-browser-deliberation.py')
  guard={'snapshot':'s','requirements_revision':'r','files':[],'acceptance_parents':[],'parents':[],'history':[],'guidance':[]}
  component={'state_root':'old','attention_binding':'old-attention','proposals':[{'kind':'edit_set','edits':['authored immutable payload']}]}
  whole={'deliberation':{'artifact':component,'development':None,'artifact_dependency':'independent','guard':guard}}
  response={**component,'state_root':'new','attention_binding':'actual-attention'}
  source={'state_root':'new','source_snapshot':'s','scope':'scope'}
  record={'response':whole,'return_basis':{'artifact':{},'development':{}}}
  bound={'decoded_sha256':m.e.sha([whole,record['return_basis']]),'source':{'state_root':'old','source_snapshot':'s','scope':'scope'}}
  proof={'version':'owned-deliberation-component@v1','origin':'origin','component':'artifact','original_response_sha256':m.e.sha(whole),'original_component_sha256':m.e.sha(component),'original_source':bound['source'],'target_source':source,'target_attention_binding':'actual-attention','before_guard':guard,'after_guard':guard,'response':response,'return_basis':{},'development_outcome':None,'metadata_transformation':['state_root','attention_binding']}
  ident='component_'+m.e.sha(proof)
  class Adapter:
   component_sha=staticmethod(m.e.sha)
   def __init__(self):self.done={'derivation_'+ident:proof,'generated_origin':record}
   def read(self):return [],{},self.done
   def base_bound_component_origin(self,key):
    if key!='origin':raise ValueError('foreign origin')
    return bound
  a=Adapter();context={**source,'binding':'different-cycle-binding'}
  self.assertEqual(helper.verify(a,ident,response,{},historical_context=context)['decoded_sha256'],m.e.sha([response,{}]))
  for field,value in [('origin','foreign'),('response',{}),('development_outcome',{'forged':True}),('target_attention_binding','cycle-binding')]:
   bad=copy.deepcopy(proof);bad[field]=value;a.done['derivation_'+ident]=bad
   with self.subTest(field=field),self.assertRaises(ValueError):helper.verify(a,ident,response,{},historical_context=context)
  a.done['derivation_'+ident]=proof
  with self.assertRaises(ValueError):helper.verify(a,ident,response,{},historical_context={**context,'source_snapshot':'different'})
 def test_new_child_cannot_be_replaced_by_later_development(self):
  import copy
  helper=m.load('child_delta_fixture','engine-browser-deliberation.py')
  before={'snapshot':'s','requirements_revision':'r','files':[],'acceptance_parents':[],'parents':[{'id':'root','meaning':'retained'}],'history':[],'guidance':[]}
  after=copy.deepcopy(before);after['parents'].append({'id':'child','meaning':'admitted'})
  capture={'event':'event','advance_result':{'preference_admitted':True},'generated_preferences':[{'id':'child','admission':'event'}],'parents':copy.deepcopy(after['parents'])}
  development={'proposals':[{'kind':'provisional_preference'}]}
  helper.unchanged(before,after,development,capture)
  after['parents'][-1]['meaning']='later revision'
  with self.assertRaisesRegex(ValueError,'exact admission'):helper.unchanged(before,after,development,capture)

class ReconsiderationBookkeeping(unittest.TestCase):
 def test_each_committed_advisory_kind_recovers_its_decision_marker(self):
  import tempfile
  from pathlib import Path
  for flag in ('qualitative_admitted','meaning_admitted','preference_admitted'):
   with self.subTest(flag=flag),tempfile.TemporaryDirectory() as tmp:
    class Harness:
     continuous=True;objective=False
     admission={}
     root=Path(tmp)
     def __init__(self):self.done={'reconsider_0_advance':{flag:True,'review_required':False}}
     def _recover_pending(self):return 0
     def validate_current_admission(self):pass
     def read(self):return [],{},self.done
     def context_refresh(self,done):return None
     def op(self,name,fn):self.done[name]=fn();return self.done[name]
     def terminal(self,status,**fields):return {'status':status,**fields}
     def current(self):return {'branch':'review_required'}
    h=Harness();m.loop.Cycle._run_locked(h)
    self.assertEqual(h.done['advance_0'],h.done['reconsider_0_advance'])

class RetentionReview(unittest.TestCase):
 def test_reconsideration_rejection_is_not_presented_as_approved_retention(self):
  whole={'deliberation':{'development':None,'artifact':{'retain':'Authored no change'},'artifact_dependency':'independent'}}
  class Harness:
   def __init__(self):self.done={};self.view={'branch':'reconsider','state_root':'root','source_snapshot':'source','binding':'binding','reconsideration':{'evidence_key':'evidence'}}
   def read(self):return [],{},self.done
   def record_component(self,name,value):self.done[name]=value;return value
   def current(self,**kwargs):return self.view.copy()
   def op(self,name,fn):self.done[name]=fn();return self.done[name]
   def native(self,kind,value):
    if kind=='deliberation_component':return {'response':whole['deliberation']['artifact']}
    if kind=='acknowledge':self.view={**self.view,'branch':'propose'};return value
    raise AssertionError(kind)
   def model_stage(self,*args):return {'judgment':{'decision':'reject','response_digest':m.e.sha(whole['deliberation']['artifact']),'reviewer':'independent fixture','generator':'fixture'}}
   def same_advisory(self,a,b):assert a==b
   def terminal(self,**fields):return fields
  h=Harness();result=m.loop.Cycle.deliberation(h,'0',whole,{})
  self.assertEqual(result['status'],'advisory_rejected');self.assertEqual(result['retention_review']['decision'],'reject')
  self.assertEqual(h.done['compound_0_acknowledge']['outcome'],'rejected')
