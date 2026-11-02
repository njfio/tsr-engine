"""Authored local provider responses, native admission/review, zero target execution."""
import copy,json,unittest,tempfile
from pathlib import Path
import test_engine_project_cycle_adapter as base
m=base.m

class Representation(base.NativeAccounting):
 def setUp(self):
  super().setUp()
  self.a=m.admission(self.root,self.origin,self.manifest,self.driver.admission['expires_at'],max_decisions=1,max_executions=0,release=True,correction_allowance=2,representation_regeneration=True)
 def oversized(self):return {'alternatives':[{'edits':[{'path':'src/lib.rs','replacement':'x'*2600}],'rationale':'Authored size control; not a program solution'}]}
 def test_fresh_valid_native_review_and_durable_replay(self):
  bodies=[]
  def transport(body,timeout):
   bodies.append(copy.deepcopy(body))
   if len(bodies)==1:return {'value':self.oversized(),'receipt':self.receipt()}
   return self.transport(body,timeout)
  a=m.Adapter(self.root,self.a,self.c,self.ledger,transport)
  ca={'version':'local-project-cycle@v3','journal':str((self.base.root/'outer').resolve()),'scope':self.origin['scope'],'max_decisions':1,'max_executions':0,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__))}
  c=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review);result=c.run()
  self.assertEqual(result['status'],'execution_budget');self.assertEqual(len(bodies),3)
  self.assertEqual(bodies[0]['messages'],bodies[1]['messages'][:-1])
  feedback=json.loads(bodies[1]['messages'][-1]['content'])['protocol_correction']
  self.assertEqual(feedback['category'],'representation_size')
  self.assertEqual(feedback['native_structure']['path'],'$.alternatives[0].edits[0].replacement')
  self.assertEqual(feedback['native_structure']['actual_utf8_bytes'],2600);self.assertEqual(feedback['native_structure']['max_utf8_bytes'],2048)
  self.assertEqual(self.ledger.status()['calls'],3);self.assertEqual(self.ledger.status()['execution_groups'],0)
  self.assertEqual(self.c.budget()['model_calls'],3);self.assertEqual(len(a.corrections.events()),1)
  before=self.ledger.path.read_bytes();again=m.Adapter(self.root,self.a,self.c,self.ledger,lambda *_:self.fail('redispatch'))
  self.assertEqual(m.loop.Cycle(ca['journal'],ca,again.native,again.proposal,again.review).run(),result)
  self.assertEqual(self.ledger.path.read_bytes(),before)
 def test_exhaustion_restart_and_old_admission_terminal(self):
  calls=[]
  def bad(*_):calls.append(1);return {'value':self.oversized(),'receipt':self.receipt()}
  a=m.Adapter(self.root,self.a,self.c,self.ledger,bad)
  with self.assertRaisesRegex(m.d.Blocked,'exhausted'):a.proposal(a.native('request',None))
  self.assertEqual(len(calls),3);self.assertEqual(len(a.corrections.events()),2)
  restarted=m.Adapter(self.root,self.a,self.c,self.ledger,bad)
  with self.assertRaisesRegex(m.d.Blocked,'terminal'):restarted.proposal(restarted.native('request',None))
  self.assertEqual(len(calls),3)
 def test_old_profile_stays_terminal(self):
  old={k:v for k,v in self.a.items() if k!='representation_regeneration'}
  a=m.Adapter(self.root,old,self.c,self.ledger,lambda *_:{'value':self.oversized(),'receipt':self.receipt()})
  with self.assertRaises(m.d.Blocked):a.proposal(a.native('request',None))
  self.assertEqual(self.ledger.status()['calls'],1);self.assertEqual(len(a.corrections.events()),0)
 def test_unknown_no_second_request(self):
  a=m.Adapter(self.root,self.a,self.c,self.ledger,lambda *_:{'value':self.oversized(),'receipt':{**self.receipt(),'usd':None}})
  with self.assertRaises(m.d.Blocked):a.proposal(a.native('request',None))
  self.assertEqual(self.ledger.status()['calls'],1);self.assertEqual(len(a.corrections.events()),0)
 def test_forbidden_mixed_stale_and_semantic_errors_ineligible(self):
  a=m.Adapter(self.root,self.a,self.c,self.ledger,self.transport);view=a.native('request',None);request,context=a.project(view)
  cases=[]
  bad=self.oversized();bad['alternatives'][0]['edits'][0]['path']='tests/contract.rs';cases.append((bad,view))
  bad=self.oversized();bad['alternatives'].append({'experiment':{}});cases.append((bad,view))
  cases.append((self.oversized(),{**view,'state_root':'f'*64}))
  cases.append((self.oversized(),{**view,'branch':'reconsider'}))
  bad=self.oversized();bad['alternatives'][0]['common_event_probability']=2;cases.append((bad,view))
  bad=self.oversized();bad['alternatives'][0]['rationale']='x'*513;cases.append((bad,view))
  bad=self.oversized();bad['alternatives'][0]['rationale']='   ';cases.append((bad,view))
  bad=self.oversized();bad['alternatives'][0]['edits'][0]['replacement']='x'*9000;cases.append((bad,view))
  for value,current in cases:
   try:a.decode_proposal(value,request,context,current)
   except (ValueError,m.d.Blocked) as error:self.assertFalse(m.correction.native_feedback(getattr(error,'diagnostic',None)))
   else:self.fail('invalid candidate accepted')
  self.assertEqual(self.ledger.status()['calls'],0);self.assertEqual(a.corrections.events(),[])
for name in base.NativeAccounting.__dict__:
 if name.startswith('test_'):setattr(Representation,name,None)

class Boundaries(unittest.TestCase):
 def test_captured_failure_metadata_and_size_classification(self):
  path=Path(m.e.ROOT/'target/native-information-live-1/adapter/output-0/failed-response-1/content.bin')
  if not path.exists():self.skipTest('historical retained local diagnostic not available; never reconstructed')
  value=json.loads(path.read_bytes());m.e.validate_stage_envelope('final_proposal_only',value['alternatives'][0])
  self.assertEqual(len(value['alternatives'][0]['edits'][0]['replacement'].encode()),2600)
  with self.assertRaises(m.e.StageValidationError):m.e.validate_trial_payload(value['alternatives'][0],{},'open_choice')
 def test_budget_unknown_review_resource_stop_and_default(self):
  diagnostic='Error: TSR_SHAPE_V1:'+json.dumps({'version':'native-shape-diagnostic@v1','code':'representation_size','path':'$.alternatives[0].edits[0].replacement','max_utf8_bytes':2048,'actual_utf8_bytes':2600})
  with tempfile.TemporaryDirectory() as tmp:
   b=m.correction.Budget(Path(tmp)/'journal',{'representation_regeneration':True},2)
   for stage,result in [('review',{'receipt':{'status':'settled','usd':'.001'}}),('proposal',{'resource_stop':True,'receipt':{'status':'settled','usd':'.001'}}),('proposal',{'receipt':{'status':'settled','usd':None}}),('proposal',{'receipt':{'status':'unknown','usd':'.001'}})]:
    with self.assertRaises(ValueError):b.feedback(stage,'a'*64,result,'b'*64,diagnostic)
   self.assertEqual(b.events(),[])
   known={'receipt':{'status':'settled','usd':'.001','parse_location':'content','error_code':'malformed_json'}}
   b.feedback('proposal','a'*64,known,'1'*64)
   b.feedback('proposal','a'*64,known,'2'*64,diagnostic)
   with self.assertRaisesRegex(ValueError,'exhausted'):b.feedback('proposal','a'*64,known,'3'*64,diagnostic)
   self.assertEqual(len(b.events()),2)
   restarted=m.correction.Budget(Path(tmp)/'journal',{'representation_regeneration':True},2)
   with self.assertRaisesRegex(ValueError,'exhausted'):restarted.feedback('proposal','a'*64,known,'4'*64,diagnostic)
   old=m.correction.Budget(Path(tmp)/'old',{},2)
   with self.assertRaises(ValueError):old.feedback('proposal','a'*64,{'receipt':{'status':'settled','usd':'.001'}},'b'*64,diagnostic)
 def test_post_upgrade_grant_prediction_exact_and_no_mutation(self):
  p=m.load('representation_prepare','engine-information-prepare.py');camp=base.campaign
  rows=[camp.Ledger.row([],{'kind':'authored_fixture'})];before=copy.deepcopy(rows)
  upgrade={'authored':'local only'};predicted=p.prospective_upgrade_row(rows,upgrade,camp)
  self.assertEqual(predicted,camp.Ledger.row(rows,{'kind':'upgrade','key':camp.digest(upgrade),'value':upgrade}));self.assertEqual(rows,before)
  grant={'version':'native-information-extension@v2','manifest_sha256':'a'*64,'ledger_prefix_head':predicted['hash'],'previous_max_calls':238,'max_calls':239,'previous_max_execution_groups':64,'max_execution_groups':64,'cap_microusd':5000000,'reservation_microusd':50000,'authority_sha256':'c'*64,'reviewer':'authored fixture','suite_manifest_sha256':'d'*64,'source_class':'newly-authored-nonprivate-native-information-pursuit'}
  camp.validate_evaluation_grant(grant,'a'*64,predicted['hash'],238,64)
  with self.assertRaises(ValueError):camp.validate_evaluation_grant(grant,'a'*64,rows[-1]['hash'],238,64)
  with self.assertRaises(ValueError):camp.validate_evaluation_grant({**grant,'max_calls':240},'a'*64,predicted['hash'],238,64)

 def test_real_append_and_replay_of_v2_grant_on_authored_fixture(self):
  camp=base.campaign
  with tempfile.TemporaryDirectory() as tmp:
   path=Path(tmp)/'ledger';manifest=camp.manifest('a'*64,path,fixture=True)
   ledger=camp.Ledger(path,manifest,camp.digest(manifest),create=True)
   steps=[('b91-call-extension@v1',140,40,False),('b91-evaluation-extension@v1',200,60,True),('b91-inquiry-call-extension@v1',202,60,False),('b91-inquiry-call-extension@v2',220,60,False),('native-pursuit-extension@v1',228,62,True),('native-pursuit-extension@v2',230,62,True),('native-information-extension@v1',238,64,True),('native-information-extension@v2',239,64,True)]
   for version,calls,groups,evaluation in steps:
    before=ledger.status();head=json.loads(path.read_text().splitlines()[-1])['hash']
    grant={'version':version,'manifest_sha256':camp.digest(manifest),'ledger_prefix_head':head,'previous_max_calls':before['effective_max_calls'],'max_calls':calls,'max_execution_groups':groups,'cap_microusd':5000000,'reservation_microusd':50000,'authority_sha256':'c'*64,'reviewer':'authored local fixture'}
    if evaluation:
     source='newly-authored-nonprivate-native-information-pursuit' if version.startswith('native-information') else 'newly-authored-nonprivate-native-pursuit' if version.startswith('native-pursuit') else 'newly-authored-nonprivate-rust-repair-suite'
     grant.update(previous_max_execution_groups=before['effective_max_execution_groups'],suite_manifest_sha256='d'*64,source_class=source)
    after=ledger.grant_calls(grant,camp.digest(grant));self.assertEqual(after['effective_max_calls'],calls)
    for key in ('calls','execution_groups','known_cost_microusd','unknown_calls','unknown_executions'):self.assertEqual(after[key],before[key])
   frozen=path.read_bytes()
   with self.assertRaises(ValueError):ledger.grant_calls(grant,camp.digest(grant))
   with self.assertRaises(ValueError):ledger.grant_calls({**grant,'max_calls':240},camp.digest({**grant,'max_calls':240}))
   self.assertEqual(path.read_bytes(),frozen)
   self.assertEqual(camp.Ledger(path,manifest,camp.digest(manifest)).status(),after)
