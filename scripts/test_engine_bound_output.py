"""Authored failure injection; no provider, target, or historical-state mutation."""
import copy,importlib.util,tempfile,unittest
from pathlib import Path

def load(name):
 s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name.replace('_','-')+'.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
b=load('engine_bound_output');shape=load('engine_project_output');correction=load('engine_bounded_correction')
class Association(unittest.TestCase):
 def setUp(self):
  self.admission={'version':'authored-admission','correction_allowance':2};self.source={'state_root':'root','source_snapshot':'snapshot','scope':'scope'}
  self.schema={'type':'object','required':['binding','receipt','decision'],'properties':{'binding':{'const':'13b34935475f83fe0db24270cf63142ce6c47a9f2e588a7a2167c5f15564a2c8'},'receipt':{'const':'r'*64},'decision':{'enum':['met','not_met','unknown','abstain']}}}
  self.semantic,self.metadata=b.review_contract(self.schema);self.wire={'response_schema':self.semantic,'context':'authored public only'}
  self.content={'decision':'met'};self.dispatch={'candidate_sha256':b.digest(self.content),'receipt':{'usage_status':'reported','usd':'0.0001'}};self.settled={'stopped':False,'unknown_usage':0}
  self.ident=b.digest({'admission':b.digest(self.admission),'kind':'review','index':0,'body':b.digest(self.wire)})
 def record(self):
  association=b.associate(admission=self.admission,kind='review',index=0,wire=self.wire,ident=self.ident,content=self.content,dispatch=self.dispatch,settled=self.settled,source=self.source,current_source=self.source)
  return {'association':association,'model_content':copy.deepcopy(self.content),'host_metadata':copy.deepcopy(self.metadata),'native_content_sha256':b.digest(b.attach_review(self.content,self.metadata)),'source':copy.deepcopy(self.source)}
 def verify(self,record,**changes):
  args=dict(admission=self.admission,ident=self.ident,dispatch=self.dispatch,settled=self.settled,source=self.source);args.update(changes);return b.verify_envelope(record,**args)
 def test_actual_extra_character_not_repaired_or_correction_eligible(self):
  bad='13b349354f75f83fe0db24270cf63142ce6c47a9f2e588a7a2167c5f15564a2c8'
  self.assertEqual(len(bad),65)
  with self.assertRaises(ValueError) as error:shape.check({**self.metadata,'binding':bad,**self.content},self.schema)
  self.assertFalse(hasattr(error.exception,'diagnostic'))
  for supplied in (bad,self.metadata['binding']):
   with self.assertRaisesRegex(ValueError,'no repair'):b.attach_review({**self.content,'binding':supplied},self.metadata)
  with tempfile.TemporaryDirectory() as t:
   budget=correction.Budget(Path(t)/'corrections',self.admission,2)
   with self.assertRaisesRegex(ValueError,'no correction'):budget.feedback('review','context',{'receipt':{'status':'settled','usd':'0.001'}},'request',native_diagnostic=str(error.exception))
   self.assertEqual(budget.events(),[])
 def test_all_semantic_decisions_preserved(self):
  for decision in ('met','not_met','unknown','abstain'):
   value={'decision':decision};shape.check(value,self.semantic);native=b.attach_review(value,self.metadata);shape.check(native,self.schema);self.assertEqual(native['decision'],decision)
  self.assertNotIn('binding',self.semantic['properties']);self.assertEqual(self.schema['properties']['binding']['const'],self.metadata['binding'])
 def test_unchanged_verification_replay_no_effect(self):
  record=self.record();original=copy.deepcopy(record)
  self.verify(record);self.verify(record);self.assertEqual(record,original)
 def test_cross_request_root_source_and_admission_rejected(self):
  record=self.record()
  for change in ({'ident':'different'},{'source':{**self.source,'state_root':'other'}},{'source':{**self.source,'source_snapshot':'stale'}},{'admission':{'foreign':True}}):
   with self.subTest(change=change),self.assertRaises(ValueError):self.verify(record,**change)
 def test_substituted_model_content_receipt_or_metadata_rejected(self):
  for field in ('model_content','host_metadata','source','association'):
   record=self.record()
   if field=='model_content':record[field]['decision']='not_met'
   elif field=='host_metadata':record[field]['binding']='other'
   elif field=='source':record[field]['state_root']='other'
   else:record[field]['index']=1
   with self.subTest(field=field),self.assertRaises(ValueError):self.verify(record)
  with self.assertRaises(ValueError):self.verify(self.record(),dispatch={**self.dispatch,'receipt':{'usage_status':'reported','usd':'1'}})
 def test_unknown_unsettled_and_stale_association_rejected(self):
  record=self.record()
  for settled in (None,{'stopped':True},{'unknown_usage':1},{'unknown_external_cost_requests':1},{'unsettled_requests':1}):
   with self.subTest(settled=settled),self.assertRaises(ValueError):self.verify(record,settled=settled)
  with self.assertRaises(ValueError):b.associate(admission=self.admission,kind='review',index=0,wire=self.wire,ident=self.ident,content={'decision':'unknown'},dispatch=self.dispatch,settled=self.settled,source=self.source,current_source=self.source)
 def test_semantic_citations_remain_model_owned(self):
  schema=copy.deepcopy(self.schema);schema['properties']['evidence']={'type':'array','items':{'enum':['exact-A','exact-B']}};schema['required'].append('evidence')
  semantic,metadata=b.review_contract(schema);self.assertNotIn('evidence',metadata)
  with self.assertRaises(ValueError):shape.check({'decision':'met','evidence':['invented']},semantic)
  shape.check({'decision':'unknown','evidence':['exact-B']},semantic)

adapter=load('engine_project_cycle_adapter')
objective=load('engine_objective_adapter')
class AdapterOwnership(unittest.TestCase):
 def setUp(self):
  self.a=object.__new__(adapter.Adapter);self.a.admission={'authored':True};self.source={'scope':'s','state_root':'r','source_snapshot':'p'};self.done={};self.pending={}
  self.a.read=lambda:('',self.pending,self.done);self.a.native=lambda kind,value:copy.deepcopy(self.source)
  self.response={'proposals':[{'kind':'service'}]};self.review={'response_digest':'candidate','decision':'approve','evidence':'authored'}
  self.gen=self.add('proposal',{'alternatives':[]},decoded=[self.response,{}]);self.done['generated_'+self.gen]={'response':self.response,'return_basis':{}}
  self.rev=self.add('review',{'decision':'approve','evidence':'authored'},metadata={'response_digest':'candidate'})
  self.done['reviewed_'+self.rev]={'report':self.review,'candidate':self.response}
  self.judgment={'decision':'approve','response_digest':'candidate','reviewer':'separate-runtime-invocation:'+self.rev,'generator':adapter.MODEL}
 def add(self,kind,content,metadata=None,decoded=None):
  wire={'authored':kind};idx=len(self.done);ident=adapter.e.sha({'admission':adapter.e.sha(self.a.admission),'kind':kind,'index':idx,'body':adapter.e.sha(wire)})
  dispatch={'candidate_sha256':b.digest(content),'receipt':{'usage_status':'reported'}};settled={'stopped':False}
  association=b.associate(admission=self.a.admission,kind=kind,index=idx,wire=wire,ident=ident,content=content,dispatch=dispatch,settled=settled,source=self.source,current_source=self.source)
  self.done['dispatch_'+ident]=dispatch;self.done['settle_'+ident]=settled
  self.done['bound_output_'+ident]={'association':association,'source':copy.deepcopy(self.source),'model_content':content,'host_metadata':metadata or {},'native_content_sha256':b.digest(b.attach_review(content,metadata or {})),'decoded_sha256':b.digest(decoded)}
  return ident
 def test_owned_import_and_replay(self):
  self.a.verify_owned_review(self.judgment,self.response);self.a.verify_owned_review(self.judgment,self.response)
 def test_cross_candidate_decision_missing_generator_and_stale_root(self):
  with self.assertRaises(ValueError):self.a.verify_owned_review(self.judgment,{'proposals':[]})
  with self.assertRaises(ValueError):self.a.verify_owned_review({**self.judgment,'decision':'reject'},self.response)
  self.done.pop('generated_'+self.gen)
  with self.assertRaises(ValueError):self.a.verify_owned_review(self.judgment,self.response)
  self.source['state_root']='new'
  with self.assertRaises(ValueError):self.a.verify_owned_review(self.judgment)
 def test_unknown_pending_and_cross_request_report(self):
  self.pending['request']='unknown'
  with self.assertRaises(ValueError):self.a.verify_owned_review(self.judgment)
  self.pending.clear();self.done['reviewed_'+self.rev]['report']={**self.review,'decision':'reject'}
  with self.assertRaises(ValueError):self.a.verify_owned_review(self.judgment)
 def test_exact_generated_basis_binding_and_legacy_ambiguity(self):
  second=self.add('proposal',{'alternatives':['corrected']},decoded=[self.response,{}]);self.done['generated_'+second]={'response':self.response,'return_basis':{}}
  with self.assertRaisesRegex(ValueError,'ambiguous'):self.a.verify_owned_review(self.judgment,self.response)
  basis=adapter.e.sha(adapter.review_contract.indexed_basis(self.response,{}))
  owned=self.done['reviewed_'+self.rev];owned.update(generated_request=second,return_basis_sha256=basis,binding={'evidence_ids':['candidate:candidate','return_basis:'+basis]})
  self.a.verify_owned_review(self.judgment,self.response)
  owned['return_basis_sha256']='wrong'
  with self.assertRaisesRegex(ValueError,'basis substitution'):self.a.verify_owned_review(self.judgment,self.response)
  owned['return_basis_sha256']=basis;owned['generated_request']='missing'
  with self.assertRaises(ValueError):self.a.verify_owned_review(self.judgment,self.response)
 def test_generated_source_metadata_is_rejected_not_rewritten(self):
  a=object.__new__(objective.Adapter);a.native=lambda *args:None
  context={'source':{'path':'lib.rs','source':'old','source_digest':'d','inspection_handle':'h'}}
  request={'protocol':'p','state_root':'r','next_attention':{'binding':'a'}}
  model={'alternatives':[{'kind':'edit_set','rationale':'authored','patches':[{'path':'lib.rs','old_text':'old','new_text':'new'}]}]}
  original=copy.deepcopy(model);native,_=a.decode_proposal(model,request,context,{})
  self.assertEqual(model,original);self.assertEqual(native['proposals'][0]['patches'][0]['source_digest'],'d')
  for key in ('source_digest','inspection_handle'):
   bad=copy.deepcopy(model);bad['alternatives'][0]['patches'][0][key]='wrong'
   with self.assertRaisesRegex(ValueError,'no repair'):a.decode_proposal(bad,request,context,{})

class CanonicalIdentity(unittest.TestCase):
 def test_unicode_request_and_content_share_transport_canonicalization(self):
  content={'decision':'unknown','explanation':'Une relation — élargie'};wire={'question':'Qu’en savons-nous ?'};admission={'authored':True};source={'root':'r'}
  ident=adapter.e.sha({'admission':adapter.e.sha(admission),'kind':'review','index':0,'body':adapter.e.sha(wire)})
  self.assertEqual(b.digest(content),adapter.e.sha(content))
  b.associate(admission=admission,kind='review',index=0,wire=wire,ident=ident,content=content,dispatch={'candidate_sha256':adapter.e.sha(content),'receipt':{'usage_status':'reported'}},settled={},source=source,current_source=source)
 def test_semantic_constants_are_not_host_owned(self):
  schema={'properties':{'binding':{'const':'id'},'role':{'const':'semantic-role'}},'required':['binding','role']}
  semantic,metadata=b.review_contract(schema)
  self.assertEqual(metadata,{'binding':'id'});self.assertIn('role',semantic['properties']);self.assertIn('role',semantic['required'])
if __name__=='__main__':unittest.main()
