"""Actual counted checked/stage/correction/ledger code; authored transport and native view.
No provider, browser, execution callback, or historical campaign mutation.
"""
import copy,json,tempfile,types,unittest
from pathlib import Path
from scripts.test_engine_criterion_consistency import h,m,pursuit,shape,correction
from scripts import test_engine_criterion_consistency as contract_tests
from scripts.test_engine_browser_adapter import campaign

NAME='criterion_'+'a'*64+'_review'
class Harness:
 checked=m.base.Adapter.checked
 retained_format_feedback=m.Adapter.retained_format_feedback
 resume_criterion_reassessment=m.Adapter.resume_criterion_reassessment
 recover_session_transition=m.Adapter.recover_session_transition
 recovery_load=staticmethod(m.load)
 recovery_sha=staticmethod(m.e.sha)
 review_reference_aliases=True
 narrative_limit=131072
 def __init__(self,root,outputs,allowance=2,enabled=True,prepared=None):
  self.root=Path(root).resolve();self.outputs=outputs;self.crash=None;self.calls=[];self.execution_calls=0
  self.admission={'feedback_events':[],'version':'browser-project-cycle-adapter@v7','criterion_return_profile':{'consistency_reassessment':'single-fresh@v1'} if enabled else {},'payload_bytes':32768}
  if prepared is not None:self.admission=copy.deepcopy(prepared)
  self.source={'state_root':'r','source_snapshot':'s','scope':'scope'}
  self.file=self.root/'state.json'
  if not self.file.exists():self.file.write_text(json.dumps({'pending':{},'done':{}}))
  self.corrections=correction.Budget(self.root/'corrections.jsonl',self.admission,allowance)
  mp=self.root/'manifest.json'
  if not mp.exists():mp.write_text(json.dumps(campaign.manifest('f'*64,self.root/'campaign.jsonl',fixture=True)))
  manifest=json.loads(mp.read_text())
  self.ledger=campaign.Ledger(self.root/'campaign.jsonl',manifest,campaign.digest(manifest),create=not (self.root/'campaign.jsonl').exists())
  self.trial='b'*64
  if not self.ledger.status()['calls'] and not (self.root/'enrolled').exists():self.ledger.enroll(self.trial,m.e.sha(self.admission));(self.root/'enrolled').touch()
 def read(self):
  v=json.loads(self.file.read_text());return [],v['pending'],v['done']
 def append(self,event):
  _,pending,done=self.read()
  if event['kind']=='intent':pending[event['name']]=event
  else:
   pending.pop(event['name'],None);done[event['name']]=event['value']
  self.file.write_text(json.dumps({'pending':pending,'done':done}))
 def op(self,name,fn):
  if (self.root/'STOP').exists():raise ValueError('Stopped')
  _,pending,done=self.read()
  if pending:raise ValueError('unresolved fixture stage')
  if name in done:return done[name]
  if name.startswith('format_correction_') and self.crash=='ticket_gap':self.crash=None;raise RuntimeError('authored ticket gap')
  intent={'kind':'intent','name':name};self.append(intent)
  if name.startswith('format_correction_') and self.crash=='local_intent':self.crash=None;raise RuntimeError('authored local intent')
  value=fn();self.append({'kind':'result','name':name,'value':value})
  if name.startswith('format_correction_') and self.crash=='ticket_done':self.crash=None;raise RuntimeError('authored ticket done')
  if name.startswith('format_correction_') and self.crash=='ticket_done_consistency' and value['feedback'].get('category')=='criterion_consistency':self.crash=None;raise RuntimeError('authored consistency ticket done')
  return value
 def validate_current_admission(self):pass # authored fixed admission; no native mutation capability
 def native(self,action,arg):
  if action!='request':self.execution_calls+=1;raise AssertionError('must never execute')
  if (self.root/'STOP').exists():raise ValueError('Stopped')
  return {**copy.deepcopy(self.source),'branch':'propose'}
 def body(self,system,payload):return {'model':'gpt-6-luna','stream':False,'max_completion_tokens':8192,'messages':[{'role':'system','content':system},{'role':'user','content':json.dumps(payload)}],'response_format':{'type':'json_object'}}
 def known_stop(self,e):return e
 def retry_terminal_failure(self,*args):raise AssertionError('provider retry not granted in fixture')
 def call(self,kind,body):
  if (self.root/'STOP').exists():raise ValueError('Stopped')
  n=sum(k.startswith('dispatch_') for k in self.read()[2]);ident=m.e.sha({'admission':m.e.sha(self.admission),'kind':kind,'index':n,'body':m.e.sha(body)})
  self.op('request_body_'+ident,lambda:{'body':copy.deepcopy(body)})
  value=copy.deepcopy(self.outputs[n]);self.calls.append(copy.deepcopy(body))
  receipt={'status':'settled','usd':'0.001','usage_status':'reported','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':0.1}
  result={'value':value,'receipt':receipt}
  self.evidence.capture(None,json.dumps(value).encode(),None,False)
  intent={'kind':'intent','name':'dispatch_'+ident};self.append(intent)
  result=self.ledger.run_call(self.trial,ident,body,240,lambda *args:result,payload_limit=32768)
  if self.crash=='fresh_dispatch' and n==1:self.crash=None;raise RuntimeError('authored uncertain fresh dispatch')
  self.append({'kind':'result','name':'dispatch_'+ident,'value':{'candidate_sha256':m.e.sha(value),'receipt':receipt}})
  self.op('settle_'+ident,lambda:{})
  self.last_result=result;self.last_ident=ident
  return ident,value,receipt
 def verify_owned_output(self,ident):
  d=self.read()[2];m.base.bound_output.verify_envelope(d['bound_output_'+ident],admission=self.admission,ident=ident,dispatch=d['dispatch_'+ident],settled=d['settle_'+ident],source=self.source)
 def stage(self,payload=None):
  return pursuit._stage(self,NAME,h.CONSISTENT_SYSTEM,payload or contract_tests.Consistency().payload(),validator=h.validate_assessment)

def response(verdict='unknown',states=('met','met')):
 v=contract_tests.Consistency().value(verdict,states);v.pop('review_id')
 for f in v['findings'].values():f['evidence']=['e0']
 return v

def prepared_three_condition_fixture(root):
 """Unadmitted authored profile; no native state, ledger grant or dispatch."""
 root=Path(root).resolve()
 frozen=Path(__file__).resolve().parents[1]/'experiments/relational-observation-v1'
 conditions=json.loads((frozen/'conditions.json').read_text())['conditions']
 requirements=json.loads((frozen/'criteria.json').read_text())['requirements']
 events=[{'id':c['id'],'attribution':'synthetic_harness_verification','guidance':{k:json.dumps(c,ensure_ascii=False) for k in requirements}} for c in conditions]
 profile={'version':'public-criterion-return@v2','policy':'a'*64,'policy_sha256':'b'*64,'imported_returns':[],'consistency_reassessment':'single-fresh@v1'}
 prepared=m.admission(root,{'public':{'parents':[{'requirement':k} for k in requirements]}},{},{},None,max_decisions=None,max_executions=None,correction_allowance=20,feedback_events=events,feedback_cycle={'version':'authorized-feedback-cycle@v2','authorization_id':'authored-offline-only','max_continuations':None},unlimited_policy_sha256='c'*64,criterion_return_profile=profile)
 return prepared

# Provider-failure path must not be entered for this known successful wire.
Harness.retry_terminal_failure=lambda self,*args:False
class Reassessment(unittest.TestCase):
 def test_success_retains_cost_and_exact_evidence_then_inert_replay(self):
  with tempfile.TemporaryDirectory() as folder:
   a=Harness(folder,[response(),response('unknown',('unknown','met'))]);result=a.stage()
   self.assertEqual(result['value']['verdict'],'unknown');self.assertEqual(a.ledger.status()['calls'],2);self.assertEqual(a.ledger.status()['known_cost_microusd'],2000)
   self.assertEqual(a.calls[1]['messages'][:-1],a.calls[0]['messages']);self.assertEqual(a.execution_calls,0)
   self.assertEqual(len(a.corrections.events()),1);self.assertEqual(len(list(a.root.glob('output-0/failed-response-1/content.bin'))),1)
   capture={'contract_version':'criterion-evidence@v4','payload':contract_tests.Consistency().payload(),'request_source':a.source,'frame':{'snapshot':'s'}}
   h.verify_retained(a,NAME.removesuffix('_review'),capture,result)
   before=(a.root/'campaign.jsonl').read_bytes();fresh=Harness(folder,[]);self.assertEqual(fresh.stage(),result);self.assertEqual(before,(a.root/'campaign.jsonl').read_bytes())
 def test_unicode_restart_and_retained_wire_after_prior_correction(self):
  for prior in (False,True):
   with tempfile.TemporaryDirectory() as folder:
    payload=contract_tests.Consistency().payload();payload['evidence']['source']='café — 方向'
    outputs=([{'unexpected':True}] if prior else [])+[response(),response('accept',('met','met'))]
    a=Harness(folder,outputs);a.crash='ticket_done_consistency'
    with self.assertRaises(RuntimeError):a.stage(payload)
    fresh=Harness(folder,outputs);result=fresh.stage(payload)
    capture={'contract_version':'criterion-evidence@v4','payload':payload,'request_source':fresh.source,'frame':{'snapshot':'s'}}
    h.verify_retained(fresh,NAME.removesuffix('_review'),capture,result)
    self.assertEqual(fresh.ledger.status()['calls'],len(outputs));self.assertEqual(fresh.execution_calls,0)
    before=(fresh.root/'campaign.jsonl').read_bytes()
    self.assertEqual(Harness(folder,[]).stage(payload),result)
    self.assertEqual(before,(fresh.root/'campaign.jsonl').read_bytes())
 def test_prepared_three_condition_session_uses_policy_without_handoff(self):
  session=m.load('prepared_session_test','engine-browser-session.py')
  frozen=Path(__file__).resolve().parents[1]/'experiments/relational-observation-v1'
  conditions=json.loads((frozen/'conditions.json').read_text())['conditions']
  requirements=json.loads((frozen/'criteria.json').read_text())['requirements']
  with tempfile.TemporaryDirectory() as folder:
   root=Path(folder).resolve();prepared=prepared_three_condition_fixture(root);events=prepared['feedback_events']
   # Serialized prospective shape only: no native or campaign grant admission.
   packet=root/'prepared-unadmitted.json';packet.write_text(json.dumps({'status':'authored_unadmitted','adapter':prepared}))
   prepared=json.loads(packet.read_text())['adapter'];a=Harness(root,[response(),response('accept',('met','met')),response('reject',('not_met','met')),response('unknown',('unknown','met'))],allowance=prepared['correction_allowance'],prepared=prepared)
   a.proposal=None;a.review=None;a.current=0
   def apply(index):
    a.current=index;a.op('feedback_'+str(index),lambda:events[index])
   a.apply_feedback=apply
   class Cycle:
    def __init__(self,*args):pass
    def run(self):
     payload=contract_tests.Consistency().payload();payload['published_context']=events[a.current]
     payload['binding']['review_id']=str(a.current+1)*64
     payload['response_schema']=shape.review_schema(payload['binding'],payload['criteria'])
     name='criterion_'+str(a.current+1)*64+'_review'
     pursuit._stage(a,name,h.CONSISTENT_SYSTEM,payload,validator=h.validate_assessment)
     return {'status':'proposal_retention'}
   ca={'journal':str(root/'cycle')};result=session.run(a,ca,cycle_type=Cycle)
   self.assertEqual(result['status'],'completed_checkpoints');self.assertEqual(result['feedback_applied'],3)
   self.assertEqual(a.ledger.status()['calls'],4);self.assertEqual(a.ledger.status()['known_cost_microusd'],4000)
   self.assertEqual(len(a.corrections.events()),1);self.assertEqual(a.execution_calls,0)
   self.assertEqual(a.calls[0]['messages'],a.calls[1]['messages'][:-1])
   for call,index in zip(a.calls,(0,0,1,2)):
    delivered=json.loads(call['messages'][1]['content'])
    self.assertEqual(delivered['published_context'],events[index])
    for future in events[index+1:]:self.assertNotIn(future['id'],call['messages'][1]['content'])
   before=(root/'campaign.jsonl').read_bytes();session.run(a,ca,cycle_type=Cycle)
   self.assertEqual(before,(root/'campaign.jsonl').read_bytes())
 def test_valid_decisions_never_reassessed(self):
  for verdict,states in [('accept',('met','met')),('reject',('not_met','met')),('unknown',('unknown','met')),('abstain',('met','met'))]:
   with tempfile.TemporaryDirectory() as folder:
    a=Harness(folder,[response(verdict,states)]);a.stage();self.assertEqual(a.ledger.status()['calls'],1);self.assertEqual(a.corrections.events(),[])
 def test_second_contradiction_or_malformed_response_is_terminal(self):
  for second in (response(),{'unexpected':True}):
   with tempfile.TemporaryDirectory() as folder:
    a=Harness(folder,[response(),second]);
    with self.assertRaisesRegex((ValueError,m.base.d.Blocked),'already consumed'):a.stage()
    self.assertEqual(a.ledger.status()['calls'],2);self.assertTrue((a.root/'terminal-output.json').exists())
    with self.assertRaises((ValueError,m.base.d.Blocked)):Harness(folder,[]).stage()
    self.assertEqual(a.ledger.status()['calls'],2);self.assertEqual(a.execution_calls,0)
 def test_durable_ticket_resume_and_stop_without_extra_allowance(self):
  for point in ('ticket_gap','local_intent','ticket_done'):
   with tempfile.TemporaryDirectory() as folder:
    a=Harness(folder,[response(),response('reject',('not_met','met'))]);a.crash=point
    with self.assertRaises(RuntimeError):a.stage()
    self.assertEqual(a.ledger.status()['calls'],1)
    (a.root/'STOP').touch()
    with self.assertRaisesRegex(ValueError,'Stopped|stopped'):Harness(folder,[]).stage()
    (a.root/'STOP').unlink() # authored fixture resume only
    fresh=Harness(folder,[response(),response('reject',('not_met','met'))]);r=fresh.stage();self.assertEqual(r['value']['verdict'],'reject')
    self.assertEqual(fresh.ledger.status()['calls'],2);self.assertEqual(len(fresh.corrections.events()),1);self.assertEqual(fresh.execution_calls,0)
 def test_changed_source_or_payload_and_unknown_dispatch_do_not_retry(self):
  for change in ('source','payload','unknown'):
   with tempfile.TemporaryDirectory() as folder:
    a=Harness(folder,[response(),response('accept',('met','met'))]);a.crash='fresh_dispatch' if change=='unknown' else 'ticket_done'
    with self.assertRaises(RuntimeError):a.stage()
    count=a.ledger.status()['calls'];fresh=Harness(folder,[]);payload=contract_tests.Consistency().payload()
    if change=='source':fresh.source['source_snapshot']='changed'
    if change=='payload':payload['evidence']['source']='changed'
    with self.assertRaises((ValueError,m.base.d.Blocked)):fresh.stage(payload)
    self.assertEqual(fresh.ledger.status()['calls'],count);self.assertEqual(fresh.execution_calls,0)
 def test_session_entry_recovers_local_ticket_after_checkpoint_stop(self):
  session=m.load('criterion_session_test','engine-browser-session.py')
  with tempfile.TemporaryDirectory() as folder:
   a=Harness(folder,[response(),response('accept',('met','met'))]);payload=contract_tests.Consistency().payload()
   a.op(NAME.removesuffix('_review')+'_input',lambda:{'contract_version':'criterion-evidence@v4','payload':payload})
   a.crash='local_intent'
   with self.assertRaises(RuntimeError):a.stage()
   fresh=Harness(folder,[response(),response('accept',('met','met'))]);fresh.complete_pending_return=fresh.stage;fresh.proposal=None;fresh.review=None
   root=Path(folder).resolve()/'cycle';(root/'checkpoint-0').mkdir(parents=True);stop=root/'checkpoint-0/STOP';stop.touch()
   class Cycle:
    def __init__(self,*args):pass
    def run(self):return {'status':'proposal_retention'}
   with self.assertRaisesRegex(ValueError,'session stopped'):session.run(fresh,{'journal':str(root)},cycle_type=Cycle)
   self.assertEqual(fresh.ledger.status()['calls'],1);self.assertTrue(fresh.read()[1])
   stop.unlink() # authored fixture resume
   self.assertEqual(session.run(fresh,{'journal':str(root)},cycle_type=Cycle)['status'],'completed_checkpoints')
   self.assertEqual(fresh.ledger.status()['calls'],2);self.assertFalse(fresh.read()[1]);self.assertEqual(fresh.execution_calls,0)
 def test_prior_structural_correction_does_not_change_evidence_or_reset_ticket(self):
  with tempfile.TemporaryDirectory() as folder:
   a=Harness(folder,[{'unexpected':True},response(),response('accept',('met','met'))]);a.crash='ticket_done_consistency'
   with self.assertRaises(RuntimeError):a.stage()
   self.assertEqual(a.ledger.status()['calls'],2)
   fresh=Harness(folder,[{},response(),response('accept',('met','met'))]);fresh.stage()
   self.assertEqual(fresh.ledger.status()['calls'],3);self.assertEqual(fresh.ledger.status()['known_cost_microusd'],3000)
   self.assertEqual(len(fresh.corrections.events()),2);self.assertEqual(fresh.execution_calls,0)
 def test_unknown_settlement_never_consumes_reassessment_ticket(self):
  for change in ({'status':'unknown'},{'usd':None},{'usage_status':'unknown'}):
   with tempfile.TemporaryDirectory() as folder:
    a=Harness(folder,[]);result={'value':response(),'receipt':{'status':'settled','usd':'0.001','usage_status':'reported',**change}}
    with self.assertRaises(ValueError):a.corrections.feedback(NAME,'source',result,'request',h.CriterionConsistencyFailure('unknown',{'met'}).diagnostic,request_source=a.source)
    self.assertEqual(a.corrections.events(),[])
 def test_fresh_attempt_cannot_enter_provider_retry_path(self):
  with tempfile.TemporaryDirectory() as folder:
   a=Harness(folder,[response(),None]);checks=[]
   a.retry_terminal_failure=lambda *args:checks.append(args) or False
   with self.assertRaisesRegex((ValueError,m.base.d.Blocked),'already consumed'):a.stage()
   self.assertEqual(len(checks),1);self.assertEqual(a.ledger.status()['calls'],2);self.assertEqual(a.execution_calls,0)
 def test_unrelated_pending_effect_is_not_reconciled(self):
  with tempfile.TemporaryDirectory() as folder:
   a=Harness(folder,[response()]);a.crash='ticket_done'
   with self.assertRaises(RuntimeError):a.stage()
   a.append({'kind':'intent','name':'execution_unknown'})
   before=a.file.read_bytes()
   with self.assertRaisesRegex(ValueError,'unresolved effect'):a.stage()
   self.assertEqual(a.file.read_bytes(),before);self.assertEqual(a.ledger.status()['calls'],1)
 def test_missing_policy_or_shared_budget_stops_first_failure(self):
  for enabled,allowance in ((False,2),(True,0)):
   with tempfile.TemporaryDirectory() as folder:
    a=Harness(folder,[response()],allowance,enabled)
    with self.assertRaises((ValueError,m.base.d.Blocked)):a.stage()
    self.assertEqual(a.ledger.status()['calls'],1);self.assertEqual(a.corrections.events(),[])
if __name__=='__main__':unittest.main()
