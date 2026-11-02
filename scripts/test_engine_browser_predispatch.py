"""Authored proof/recovery fixtures; no provider or target execution."""
import copy,importlib.util,json,tempfile,unittest
from pathlib import Path
from types import SimpleNamespace
S=Path(__file__).parent

def load(name,file):
 s=importlib.util.spec_from_file_location(name,S/file);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
p=load('predispatch_test','engine-browser-predispatch.py');m=load('predispatch_campaign','engine-b91-campaign.py')
class Fake:
 def __init__(self,root):
  root=root.resolve();self.root=root;root.mkdir();self.admission={'authored':True};self.trial='b'*64;self.recovery_sha=m.digest;self.source={'state_root':'1'*64,'source_snapshot':'2'*64,'scope':'3'*64};self.rows=[];self.resolved={};self.interrupt=False;self.launch_lock_owned=True
  manifest=m.manifest('a'*64,root/'campaign',fixture=True);self.ledger=m.Ledger(root/'campaign',manifest,m.digest(manifest),create=True);self.ledger.enroll(self.trial,m.digest(self.admission))
  self.body={'model':m.MODEL,'stream':False,'max_completion_tokens':8192,'messages':[{'role':'user','content':'Authored retained proposal'}]}
  self.ident=m.digest({'admission':m.digest(self.admission),'kind':'proposal','index':0,'body':m.digest(self.body)})
  self.controller=SimpleNamespace(request=lambda:{'state_root':self.source['state_root'],'coding':{'snapshot':self.source['source_snapshot']},'next_attention':{'scope':self.source['scope']}},json_cli=self.resolve)
  self.op('request_body_'+self.ident,lambda:{'body':self.body,'body_sha256':m.digest(self.body),'source':copy.deepcopy(self.source)})
  self.op('reserve_'+self.ident,lambda:{'unsettled_requests':1});(root/'output-0').mkdir()
 def predispatch_tokens(self,body):return 10000
 def native(self,*args):return self.source
 def append(self,e):
  row={'seq':len(self.rows),'previous':self.rows[-1]['hash'] if self.rows else None,'event':e};self.rows.append({**row,'hash':m.digest(row)})
 def read(self):
  pending={};done={}
  for row in self.rows:
   e=row['event'];name=e['name']
   if e['kind']=='intent':pending[name]=e
   else:
    if e['intent']!=m.digest(pending.pop(name)):raise ValueError('bad fixture chain')
    done[name]=e['value']
  return self.rows,pending,done
 def op(self,name,fn):
  _,pending,done=self.read()
  if name in done:return done[name]
  if pending:raise ValueError('unknown fixture effect')
  intent={'kind':'intent','name':name};self.append(intent);value=fn()
  if self.interrupt and name.startswith('not_dispatched_'):self.interrupt=False;raise RuntimeError('authored after-native interruption')
  self.append({'kind':'result','name':name,'intent':m.digest(intent),'value':value});return value
 def resolve(self,args,proof,*_):
  self.resolved.setdefault(proof['reservation']['request_id'],copy.deepcopy(proof))
  if self.resolved[proof['reservation']['request_id']]!=proof:raise ValueError('changed proof')
  return {'known_not_dispatched_requests':1}
class Proof(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.a=Fake(Path(self.tmp.name)/'adapter')
 def test_exact_proof_recovery_idempotence_and_ticket_consumption(self):
  a=self.a;proof=p.inspect(a);self.assertEqual(proof['request'],a.ident)
  self.assertEqual(p.recover(a),1);self.assertEqual(p.recover(a),0);self.assertEqual(len(a.resolved),1)
  self.assertEqual(p.ticket(a)['body'],a.body)
  a.op('dispatch_new',lambda:{'authored':True});self.assertIsNone(p.ticket(a))
 def test_after_native_interruption_recovers_without_duplicate_disposition(self):
  a=self.a;a.interrupt=True
  with self.assertRaises(RuntimeError):p.recover(a)
  self.assertEqual(len(a.resolved),1);self.assertEqual(p.recover(a),1);self.assertEqual(len(a.resolved),1)
 def test_stale_stop_nonempty_missing_and_partial_dispatch_fail_closed(self):
  for kind in ['source','stop','output','body','intent','campaign']:
   with self.subTest(kind=kind),tempfile.TemporaryDirectory() as d:
    a=Fake(Path(d)/'a')
    if kind=='source':a.source['source_snapshot']='f'*64
    if kind=='stop':(a.root/'STOP').write_text('authored Stop')
    if kind=='output':(a.root/'output-0'/'partial').write_text('unknown')
    if kind=='body':a.body['messages'][0]['content']='changed'
    if kind=='intent':a.append({'kind':'intent','name':'dispatch_'+a.ident})
    if kind=='campaign':
     with self.assertRaises(RuntimeError):a.ledger.run_call(a.trial,a.ident,a.body,240,lambda *_:(_ for _ in ()).throw(RuntimeError('authored unknown')))
    with self.assertRaises(ValueError):p.recover(a)
    self.assertEqual(a.resolved,{})
 def test_missing_launch_ownership_blocks_disposition(self):
  self.a.launch_lock_owned=False
  with self.assertRaisesRegex(ValueError,'launch ownership'):p.recover(self.a)
  self.assertEqual(self.a.resolved,{})
 def test_existing_checkpoint_recovers_before_new_return_preparation(self):
  session=load('predispatch_session','engine-browser-session.py');a=self.a;root=a.root/'cycle';checkpoint=root/'checkpoint-0';checkpoint.mkdir(parents=True);(checkpoint/'cycle.jsonl').write_text('authored existence marker')
  events=[];a.admission['feedback_events']=[{'authored':True}];a.complete_pending_return=lambda:events.append('wrong-return');a.apply_feedback=lambda i:events.append('feedback');a.proposal=None;a.review=None;a.native=lambda *args:{'branch':'propose'}
  class Cycle:
   def __init__(self,*args):pass
   def run(self):events.append('saved-callback');return {'status':'proposal_retention'}
  result=session.run(a,{'journal':str(root)},cycle_type=Cycle)
  self.assertEqual(result['status'],'completed_checkpoints');self.assertEqual(events,['feedback','saved-callback'])

 def test_session_and_checkpoint_stop_block_recovery(self):
  launcher=load('predispatch_launcher','engine-native-cycle.py')
  root=self.a.root/'session';root.mkdir();launcher.require_recovery_not_stopped(root)
  for stop in (root/'STOP',root/'checkpoint-1'/'STOP'):
   stop.parent.mkdir(exist_ok=True);stop.write_text('authored Stop')
   with self.assertRaisesRegex(ValueError,'stopped'):launcher.require_recovery_not_stopped(root)
   stop.unlink()

 def test_stop_arrives_after_proof_before_native_disposition(self):
  a=self.a;a.recovery_session_root=a.root/'session'
  original=a.op
  def stop_after_prepare(name,fn):
   result=original(name,fn)
   if name.startswith('predispatch_input_'):
    stop=a.recovery_session_root/'checkpoint-0'/'STOP';stop.parent.mkdir(parents=True);stop.write_text('authored Stop')
   return result
  a.op=stop_after_prepare
  with self.assertRaisesRegex(ValueError,'Stop'):p.recover(a)
  self.assertEqual(a.resolved,{})

 def test_interrupted_pure_proof_capture_revalidated_once(self):
  a=self.a;name='predispatch_input_'+a.ident
  a.append({'kind':'intent','name':name})
  self.assertEqual(p.recover(a),1);self.assertEqual(p.recover(a),0)
  rows,_,done=a.read();self.assertEqual(sum(r['event']['kind']=='intent' and r['event']['name']==name for r in rows),1)
  self.assertEqual(len(a.resolved),1)
 def test_interrupted_proof_does_not_hide_dispatch_or_stale_source(self):
  a=self.a;a.append({'kind':'intent','name':'predispatch_input_'+a.ident});a.source['state_root']='f'*64
  with self.assertRaisesRegex(ValueError,'stale'):p.recover(a)
  self.assertEqual(a.resolved,{})

class RetainedResponseSafety(unittest.TestCase):
 def test_missing_content_and_resource_stop_never_fall_through_to_dispatch(self):
  helper=load('retained_response_safety','engine-browser-response-recovery.py')
  for dispatch,message in (({},'content unavailable'),({'candidate':{},'resource_stop':True},'resource stop')):
   with self.subTest(message=message):
    done={'request_body_r':{'kind':'review'},'dispatch_r':dispatch}
    adapter=SimpleNamespace(read=lambda:([],{},done))
    with self.assertRaisesRegex(ValueError,message):helper.latest(adapter)
