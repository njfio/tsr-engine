"""Offline authored proof controls; never contacts a provider."""
import copy, importlib.util, tempfile, unittest, json, os
from pathlib import Path
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('failed_review_test',Path(__file__).with_name('engine-browser-failed-review.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class ProofTests(unittest.TestCase):
 def setUp(self):
  self.pid='f20ac9ba-8859-4a5b-be1f-b78e07e1f6d8'
  self.receipt={'response_status':'received_http_error','http_status':400,'response_request_ids':[self.pid]}
  self.lookup={'request_id':self.pid,'endpoint':'https://api.cheaperinference.com/v1/usage/requests?request_ids='+self.pid,'retrieved_at':'2026-11-01T20:01:00Z','record':{'has_more':False,'missing_request_ids':[],'data':[{'request_id':self.pid,'model':'gpt-6-luna','endpoint':'/v1/chat/completions','status':'failed','currency':'USD','prompt_tokens':0,'completion_tokens':0,'total_tokens':0,'created_at':'2026-11-01T20:00:01Z','completed_at':'2026-11-01T20:00:02Z','billed_cost_usd':None,'billing':{'status':'failed','billed_cost_usd':'0.000000'}}]}}
  self.at=m.d.stamp('2026-11-01T20:00:00Z')
 def test_exact_zero_is_accounting_evidence(self):
  self.assertEqual(m.failed_zero(self.receipt,self.lookup,self.at),self.pid)
 def test_unknown_partial_and_nonzero_usage_refused(self):
  for key,value in [('prompt_tokens',None),('completion_tokens',1),('total_tokens',False),('status','completed'),('model','other')]:
   q=copy.deepcopy(self.lookup);q['record']['data'][0][key]=value
   with self.subTest(key=key),self.assertRaises(ValueError):m.failed_zero(self.receipt,q,self.at)
 def test_unknown_or_conflicting_charge_refused(self):
  for value in (None,'0.01','NaN'):
   q=copy.deepcopy(self.lookup);q['record']['data'][0]['billing']['billed_cost_usd']=value
   with self.subTest(value=value),self.assertRaises((ValueError,m.decimal.InvalidOperation)):m.failed_zero(self.receipt,q,self.at)
  q=copy.deepcopy(self.lookup);q['record']['data'][0]['billed_cost_usd']='0.01'
  with self.assertRaises(ValueError):m.failed_zero(self.receipt,q,self.at)
 def test_mismatch_missing_duplicate_and_chronology_refused(self):
  for edit in (lambda q:q.update(request_id='other'),lambda q:q['record']['data'].append(copy.deepcopy(q['record']['data'][0])),lambda q:q['record'].update(has_more=True),lambda q:q['record']['data'][0].update(created_at='2026-10-01T20:00:00Z')):
   q=copy.deepcopy(self.lookup);edit(q)
   with self.assertRaises(ValueError):m.failed_zero(self.receipt,q,self.at)
 def test_semantic_result_is_not_failed_provider(self):
  for status in (200,429,502):
   with self.assertRaises(ValueError):m.failed_zero({**self.receipt,'http_status':status},self.lookup,self.at)
 def test_reconciliation_review_does_not_authorize_retry(self):
  p={'example':'authored'};r={'version':'failed-review-review@v1','implementation_sha256':m.file_sha(m.__file__),'proof_sha256':m.e.sha(p),'decision':'exact_failed_zero','reviewer':'authored independent fixture'}
  m.reviewed(p,r)
  with self.assertRaises(ValueError):m.reviewed(p,r,retry=True)
  m.reviewed(p,{**r,'retry':'one-identical-review-only'},retry=True)
  with self.assertRaises(ValueError):m.reviewed({**p,'changed':True},r)
 def test_stop_and_exclusive_duplicate_control(self):
  with tempfile.TemporaryDirectory() as t:
   p={'parent':t}
   for n in ('adapter','cycle'):Path(t,n).mkdir()
   m.check_stop(p)
   with m.exclusive(p):
    with self.assertRaises(BlockingIOError):
     with m.exclusive(p):pass
   Path(t,'cycle','STOP').touch()
   with self.assertRaises(ValueError):m.check_stop(p)
 def test_ticket_stale_pending_and_second_failure(self):
  p={'adapter_rows':0,'source':{'snapshot':'s'},'body':{'same':'evidence'}}
  r={'version':'failed-review-review@v1','implementation_sha256':m.file_sha(m.__file__),'proof_sha256':m.e.sha(p),'decision':'exact_failed_zero','reviewer':'authored','retry':'one-identical-review-only'}
  self.assertEqual(m.retry_state(p,r,[],{}, {},p['source']),'fresh')
  with self.assertRaises(ValueError):m.retry_state(p,r,[],{'dispatch_unknown':{}},{},p['source'])
  with self.assertRaises(ValueError):m.retry_state(p,r,[],{},{},{'snapshot':'stale'})
  value={'candidate':{'verdict':'reject'},'resource_stop':False,'receipt':{'usage_status':'reported','usd':'0.0001'}}
  rows=[{'event':{'kind':'result','name':'dispatch_fresh','value':value}}];done={'request_body_fresh':{'kind':'review','body':p['body']},'settle_fresh':{}}
  self.assertEqual(m.retry_state(p,r,rows,{},done,p['source']),'fresh')
  for changed in ({**value,'resource_stop':True},{**value,'candidate':None},{**value,'receipt':{'usage_status':'unknown','usd':None}}):
   with self.assertRaises(ValueError):m.retry_state(p,r,[{'event':{**rows[0]['event'],'value':changed}}],{},done,p['source'])
  with self.assertRaises(ValueError):m.retry_state(p,r,rows+rows,{},done,p['source'])
 def test_spent_ticket_does_not_replace_future_ordinary_recovery(self):
  adapter=object.__new__(m.Adapter);adapter.failed_review_proof={'parent':'/authored','stage':'review_0'}
  with patch.object(m.history_module,'history',return_value=([],{},{'review_0':{}})),patch.object(m.ordinary.Adapter,'recovery_ready',return_value={'ordinary':'future policy'}) as original:
   self.assertEqual(adapter.recovery_ready(),{'ordinary':'future policy'});original.assert_called_once()

from test_engine_browser_adapter import BrowserAdapter
class NativeRecovery(BrowserAdapter):
 def setup_failure(self):
  self.a=m.a.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=4,max_executions=1,correction_allowance=1,feedback_cycle={'version':'authorized-feedback-cycle@v1','authorization_id':'authored-recovery','max_continuations':4},process_profile='ordinary-memory@v1')
  self.ca={'version':'local-project-cycle@v3','journal':str(self.root/'cycle'),'scope':self.a['origin']['public']['scope'],'max_decisions':4,'max_executions':1,'completion_tokens':32768,'input_tokens':m.a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'local_source_profile':'browser-scoped-source@v1','expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.a.loop.__file__)),'feedback_cycle':self.a['feedback_cycle'],'stage_receipts':'durable-local-stage@v1'}
  # Use the current controller's actual stage receipt contract.
  self.ca['stage_receipts']=m.a.loop.stage_receipts.PROFILE
  for n,v in (('adapter-admission',self.a),('cycle-admission',self.ca)):(self.root/(n+'.json')).write_text(json.dumps(v))
  self.sent=[];self.mode='failure';original=self.transport
  def wire(body,timeout):
   self.sent.append(copy.deepcopy(body))
   if len(self.sent)==2:
    binding=self.failed_adapter.active_wire_binding
    self.failed_adapter.retain_response_identity({'version':'response-header-identity@v1','request_body_sha256':binding['wire_body_sha256'],'request_body_bytes':binding['wire_body_bytes'],'response_request_ids':['f20ac9ba-8859-4a5b-be1f-b78e07e1f6d8'],'http_status':400,'identity_status':'single'})
    return {'value':None,'receipt':{'http_status':400,'response_status':'received_http_error','response_request_ids':['f20ac9ba-8859-4a5b-be1f-b78e07e1f6d8'],'usage_status':'unknown_or_inconsistent','usd':None,'usage':{'prompt_tokens':None,'completion_tokens':None,'total_tokens':None},'elapsed_seconds':.1}}
   v=original(body,timeout)
   if len(self.sent)==1:v['value']['alternatives'][0]['rationale']+=' — authored Unicode provenance'
   if len(self.sent)==3 and self.mode=='second_failure':return {'value':None,'receipt':{'http_status':503,'response_status':'received_http_error','response_request_ids':['320ed5ac-1b5a-4433-b5cd-3a2ccc25175f'],'usage_status':'unknown_or_inconsistent','usd':None,'usage':{'prompt_tokens':None,'completion_tokens':None,'total_tokens':None},'elapsed_seconds':.1}}
   if len(self.sent)==3 and self.mode=='malformed':v['value']={'not_a_review':True}
   if len(self.sent)==3 and self.mode=='reject':
    v['value']['verdict']='reject'
    for f in v['value']['findings'].values():f['finding']='not_met'
   if len(self.sent)>3:v['value']={'retain':'Authored retained choice after exact criticism; no player satisfaction claim','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'}
   return v
  self.wire=wire
  adapter=m.ordinary.Adapter(self.root/'adapter',self.a,self.c,self.ledger,wire)
  self.failed_adapter=adapter
  cycle=m.ordinary.Cycle(self.root/'cycle',self.ca,adapter.native,adapter.proposal,adapter.review)
  with self.assertRaises(Exception):cycle.run()
  self.assertEqual(len(self.sent),2);self.assertEqual(self.ledger.status()['execution_groups'],0)
  ar,_,done=m.history_module.history(self.root/'adapter/adapter.jsonl');failed=[(k,v) for k,v in done.items() if k.startswith('dispatch_')][-1]
  at=next(r['event']['at'] for r in ar if r['event'].get('name')==failed[0] and r['event']['kind']=='intent')
  fixture=ProofTests();fixture.setUp();lookup=fixture.lookup;lookup['retrieved_at']=m.d.iso_now();lookup['record']['data'][0].update(created_at=at,completed_at=at)
  (self.root/'provider-exact-lookup.json').write_text(json.dumps(lookup));self.proof=m.inspect(self.root)
  self.review={'version':'failed-review-review@v1','implementation_sha256':m.file_sha(m.__file__),'proof_sha256':m.e.sha(self.proof),'decision':'exact_failed_zero','reviewer':'authored independent fixture','retry':'one-identical-review-only'}
 def test_native_reconciliation_interruption_and_exact_review_restart(self):
  self.setup_failure();self.assertNotEqual(m.e.sha(self.proof['body']),m.campaign.digest(self.proof['body']));original=self.ledger.append_locked;failed=[False]
  def crash(f,rows,event):
   if event['kind']=='bill_reconciled' and not failed[0]:failed[0]=True;raise RuntimeError('authored after native proof before campaign append')
   return original(f,rows,event)
  self.ledger.append_locked=crash
  with self.assertRaisesRegex(RuntimeError,'after native'):m.reconcile(self.proof,self.review,self.ledger,self.c)
  self.assertEqual(self.c.budget()['unknown_usage'],0);self.assertEqual(len(self.ledger.status()['blocking_unknown_calls']),1)
  self.ledger.append_locked=original;m.reconcile(self.proof,self.review,self.ledger,self.c)
  before=self.ledger.path.read_bytes();m.reconcile(self.proof,self.review,self.ledger,self.c);self.assertEqual(before,self.ledger.path.read_bytes())
  adapter=m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,self.wire,proof=self.proof,review=self.review)
  callback=self.proof['callback'];first=adapter.review(callback['context'],callback['response']);second=adapter.review(callback['context'],callback['response'])
  self.assertEqual(m.a.loop.Cycle.measured(first,'judgment'),m.a.loop.Cycle.measured(second,'judgment'));self.assertEqual(len(self.sent),3);self.assertEqual(self.sent[1],self.sent[2]);self.assertEqual(self.ledger.status()['calls'],3);self.assertEqual(self.ledger.status()['execution_groups'],0)
 def test_ordinary_rejection_then_fresh_retention_not_review_shopping(self):
  self.setup_failure();self.mode='reject';m.reconcile(self.proof,self.review,self.ledger,self.c)
  adapter=m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,self.wire,proof=self.proof,review=self.review)
  cycle=m.ordinary.Cycle(self.root/'cycle',self.ca,adapter.native,adapter.proposal,adapter.review)
  outcome=cycle.run();self.assertEqual(outcome['status'],'proposal_retention');self.assertEqual(len(self.sent),4);self.assertEqual(self.sent[1],self.sent[2]);self.assertEqual(self.ledger.status()['execution_groups'],0)
  before=self.ledger.path.read_bytes();self.assertEqual(cycle.run(),outcome);self.assertEqual(before,self.ledger.path.read_bytes())
 def failure_stops(self,mode):
  self.setup_failure();self.mode=mode;m.reconcile(self.proof,self.review,self.ledger,self.c)
  adapter=m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,self.wire,proof=self.proof,review=self.review);callback=self.proof['callback']
  with self.assertRaises(Exception):adapter.review(callback['context'],callback['response'])
  self.assertEqual(len(self.sent),3)
  again=m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,self.wire,proof=self.proof,review=self.review)
  with self.assertRaises(Exception):again.review(callback['context'],callback['response'])
  self.assertEqual(len(self.sent),3);self.assertEqual(self.ledger.status()['execution_groups'],0)
 def test_second_provider_failure_stops(self):self.failure_stops('second_failure')
 def test_malformed_fresh_review_does_not_reset_ticket(self):self.failure_stops('malformed')
 def interrupted_before_dispatch(self,stage):
  self.setup_failure();m.reconcile(self.proof,self.review,self.ledger,self.c)
  adapter=m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,self.wire,proof=self.proof,review=self.review);callback=self.proof['callback']
  def interrupt(phase,name):
   if phase=='after_result' and (name=='failed_review_retry_claim' if stage=='claim' else name.startswith('reserve_')):raise RuntimeError('authored predispatch interrupt')
  adapter.hook=interrupt
  with self.assertRaisesRegex(RuntimeError,'predispatch'):adapter.review(callback['context'],callback['response'])
  self.assertEqual(len(self.sent),2)
  again=m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,self.wire,proof=self.proof,review=self.review);again.launch_lock_owned=True
  if stage=='reserve':again.recover_predispatch()
  (self.root/'cycle/STOP').touch()
  with self.assertRaises(ValueError):again.review(callback['context'],callback['response'])
  (self.root/'cycle/STOP').unlink() # Authored fixture Stop/resume only.
  again.review(callback['context'],callback['response']);self.assertEqual(len(self.sent),3);self.assertEqual(self.sent[1],self.sent[2])
 def test_interrupted_claim_restart(self):self.interrupted_before_dispatch('claim')
 def test_interrupted_reservation_restart(self):self.interrupted_before_dispatch('reserve')

for name in dir(BrowserAdapter):
 if name.startswith('test_') and name not in NativeRecovery.__dict__:setattr(NativeRecovery,name,None)
del BrowserAdapter

if __name__=='__main__':unittest.main()
