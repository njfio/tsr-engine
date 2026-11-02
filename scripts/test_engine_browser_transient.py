"""Authored failure injection only: no provider calls or target executions."""
import copy,datetime,fcntl,importlib.util,json,tempfile,time,unittest
from pathlib import Path
from unittest.mock import patch
from test_engine_browser_adapter import m
from test_engine_objective_continuation import failed,lookup,PID
r=m.recovery
class Backend:
 def _wire(self,body,timeout):return copy.deepcopy(self.result)
 def call(self,kind,body):
  ident=self.recovery_ident;self.events['dispatch_'+ident]={'receipt':{}}
  return ident,{},{}
 def retry_terminal_failure(self,*_):return False
class Fake(r.Recovery,Backend):
 recovery_sha=staticmethod(m.e.sha);recovery_stamp=staticmethod(m.d.stamp)
 def __init__(self):
  self.events={};self.pending={};self.source={'scope':'s','state_root':'r','source_snapshot':'v'}
  self.admission={'transient_recovery':{'version':'exact-failed-zero-recovery@v1','max_retries':2,'backoff_seconds':60},'max_calls':200,'expires_at':(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=1)).isoformat()}
  self.ledger=type('Ledger',(),{'status':lambda _: {'stopped':False}})();self.result=failed();self.failure_lookup=lookup;self.recovery_ident='first'
 def read(self):return [],self.pending,self.events
 def op(self,name,fn):
  if name not in self.events:self.events[name]=fn()
  return self.events[name]
 def native(self,*_):return copy.deepcopy(self.source)
 def validate_current_admission(self):pass
 def recovery_modules(self):
  from test_engine_objective_continuation import r as proof
  return proof
 def settle(self,body):
  out=self._wire(body,240);self.events['dispatch_first']={'receipt':out['receipt']};self.events['settle_first']={};return out
class Tests(unittest.TestCase):
 def test_exact_failed_zero_then_counted_same_request(self):
  f=Fake();body={'task':'unchanged'};out=f.settle(body)
  self.assertEqual(out['receipt']['usd'],'0.000000');self.assertIn('provider_zero_first',f.events)
  self.assertTrue(f.retry_terminal_failure('first',out['receipt']))
  ticket=f.recovery_ticket();before=ticket['not_before']
  with patch.object(r.time,'time',return_value=before+1):f.call('review',body)
  self.assertEqual(sum(k.startswith('dispatch_') for k in f.events),2)
  self.assertIsNone(f.recovery_ticket())
 def test_unknown_and_semantic_never_retry(self):
  for mode in ('lookup_missing','charge','semantic'):
   f=Fake()
   if mode=='lookup_missing':f.failure_lookup=lambda _: {}
   if mode=='charge':
    def charged(pid):
     value=lookup(pid);value['record']['data'][0]['billing']['billed_cost_usd']='0.1';return value
    f.failure_lookup=charged
   if mode=='semantic':f.result={'value':{'decision':'reject'},'receipt':{'status':'settled'}}
   out=f.settle({});self.assertFalse(f.retry_terminal_failure('first',out['receipt']));self.assertNotIn('provider_zero_first',f.events)
 def test_stale_body_source_unknown_and_exhaustion(self):
  for mode in ('body','source','pending','unknown','cap','deadline','allowance'):
   f=Fake();f.settle({'task':1})
   if mode=='source':f.source['state_root']='changed'
   if mode=='pending':f.pending={'dispatch_unknown':{}}
   if mode=='unknown':f.ledger.status=lambda: {'stopped':True}
   if mode=='cap':f.admission['max_calls']=1
   if mode=='deadline':f.admission['expires_at']=datetime.datetime.now(datetime.timezone.utc).isoformat()
   if mode=='allowance':f.admission['transient_recovery']['max_retries']=1;f.events['provider_retry_old']={}
   with self.subTest(mode=mode),self.assertRaises(ValueError):
    if mode=='body':f.call('review',{'task':2})
    else:f.recovery_ready()
   self.assertEqual(sum(k.startswith('dispatch_') for k in f.events),1)
 def test_restart_preserves_backoff_and_no_completed_request_replay(self):
  f=Fake();body={'task':1};f.settle(body);ticket=f.recovery_ticket()
  fresh=Fake();fresh.events=copy.deepcopy(f.events);fresh.admission=copy.deepcopy(f.admission)
  self.assertEqual(fresh.recovery_ticket(),ticket)
  with patch.object(r.time,'time',return_value=ticket['not_before']+1):fresh.call('review',body)
  self.assertIsNone(fresh.recovery_ticket())
  fresh.pending={'dispatch_uncertain':{}}
  with self.assertRaises(ValueError):fresh.recovery_ready()
 def test_corrected_wire_survives_restart(self):
  f=Fake();original={'task':'original'};corrected={'task':'original','format_feedback':'exact'}
  f.recovery_wire(original,corrected);f.settle(corrected);f.recovery_ticket()
  fresh=Fake();fresh.events=copy.deepcopy(f.events)
  self.assertEqual(fresh.recovery_wire(original,original),corrected)
  with self.assertRaises(ValueError):fresh.recovery_wire({'task':'changed'},original)
 def test_policy_absent_and_invalid(self):
  r.validate(None)
  for p in ({},{'version':'exact-failed-zero-recovery@v1','max_retries':0,'backoff_seconds':60},{'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':0}):
   with self.assertRaises(ValueError):r.validate(p)
 def test_outer_review_restart_uses_captured_input_once(self):
  with tempfile.TemporaryDirectory(dir=m.e.ROOT/'target') as td:
   policy={'version':'authorized-feedback-cycle@v1','authorization_id':'authored-test','max_continuations':1}
   ad={'version':'local-project-cycle@v3','journal':td,'scope':'s','max_decisions':2,'max_executions':0,'expires_at':(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=1)).isoformat(),'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','feedback_cycle':policy}
   context={'scope':'s','state_root':'r','source_snapshot':'v','dispatch_authorized':False};calls=[]
   def native(kind,value):
    if kind=='provider_recovery_ready':return {'ready':True}
    return context
   def review(c,response):
    calls.append((c,response));return {'judgment':{'decision':'reject'},'receipt':{'mode':'fixture','status':'settled','elapsed_seconds':0.1,'usd':'0','usage_status':'reported','usage':{'prompt_tokens':0,'completion_tokens':0,'total_tokens':0}}}
   cycle=m.loop.Cycle(td,ad,native,lambda *_:None,review)
   cycle.op('callback_input_review_0',lambda:{'kind':'review','context':context,'response':{'proposals':[]}})
   cycle.append({'kind':'intent','name':'review_0','at':m.d.iso_now()})
   with open(Path(td)/'run.lock','a') as held:
    fcntl.flock(held,fcntl.LOCK_EX|fcntl.LOCK_NB)
    with self.assertRaises(BlockingIOError):cycle.recover_pending()
   self.assertFalse(calls)
   self.assertEqual(cycle.recover_pending(),1);self.assertEqual(cycle.recover_pending(),0);self.assertEqual(len(calls),1)
   self.assertEqual(cycle.read()[2]['review_0']['judgment']['decision'],'reject')
if __name__=='__main__':unittest.main()
