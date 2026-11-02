"""Authored transport/checker outputs and cfg(test) native receipts only.

No provider, container, generated code, or live campaign mutation. These controls
establish recovery wiring, not model judgment or target behavior.
"""
import copy
import datetime
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch
s=importlib.util.spec_from_file_location('recovery',Path(__file__).with_name('engine-objective-continuation.py'));r=importlib.util.module_from_spec(s);s.loader.exec_module(r)

def expiry():return (datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=2)).isoformat()
def lookup(pid,now=None):
 now=now or r.d.iso_now()
 return {'request_id':pid,'retrieved_at':now,'endpoint':'https://api.cheaperinference.com/v1/usage/requests?request_ids='+pid,'record':{'has_more':False,'data':[{'request_id':pid,'model':r.c.MODEL,'endpoint':'/v1/chat/completions','status':'failed','currency':'USD','billing':{'status':'failed','billed_cost_usd':'0.000000'},'billed_cost_usd':None,'prompt_tokens':0,'completion_tokens':0,'total_tokens':0,'created_at':now}]}}
PID='11111111-1111-4111-8111-111111111111'
def failed():return {'value':None,'receipt':{'status':'unknown','http_status':503,'response_status':'received_http_error','response_request_ids':[PID],'usd':None,'usage':{},'elapsed_seconds':0.01}}
def settled(value):return {'value':value,'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':0.01}}

class ProofControls(unittest.TestCase):
 def test_exact_failed_zero_and_substitutions(self):
  receipt=failed()['receipt'];v=lookup(PID);self.assertEqual(r.failed_zero(receipt,v,earliest=time.time()),PID)
  for change in ('request','charge','missing','status','model','tokens','page','chronology'):
   bad=copy.deepcopy(v);record=bad['record']['data'][0]
   if change=='request':bad['request_id']='22222222-2222-4222-8222-222222222222'
   if change=='charge':record['billing']['billed_cost_usd']='0.1'
   if change=='missing':record['billing']['billed_cost_usd']=None
   if change=='status':record['status']='pending'
   if change=='model':record['model']='different'
   if change=='tokens':record['completion_tokens']=1
   if change=='page':bad['record']['has_more']=True
   if change=='chronology':record['created_at']='2000-01-01T00:00:00Z'
   with self.subTest(change=change),self.assertRaises((ValueError,r.decimal.InvalidOperation)):r.failed_zero(receipt,bad,earliest=time.time())
  for code in (200,400,401,429,500):
   with self.assertRaises(ValueError):r.failed_zero({**receipt,'http_status':code},v,earliest=time.time())
 def test_retry_default_zero_and_exact_optin(self):
  prefix={'origin':{},'parent':'/tmp/unused'}
  for n in (False,True,-1,2,'1'):
   with self.assertRaises(ValueError):r.admission('/tmp/unused-derivative',prefix,{},expiry(),retry_allowance=n)
  self.assertEqual(r.admission('/tmp/unused-derivative',prefix,{},expiry())['terminal_retry_allowance'],0)
 def test_live_derivative_scope_and_fixture_scope(self):
  class Ledger:manifest={'fixture_only':False}
  prefix={'origin':{},'parent':str(r.e.ROOT/'target/frozen-parent')}
  ad=r.admission('/tmp/not-admitted-live-root',prefix,Ledger.manifest,expiry())
  with patch.object(r,'verify_parent') as check:
   with self.assertRaisesRegex(ValueError,'inside repository target'):r.validate(ad,Ledger())
   check.assert_not_called()
  Ledger.manifest={'fixture_only':True};ad=r.admission('/tmp/authored-fixture',prefix,Ledger.manifest,expiry())
  with patch.object(r,'verify_parent') as check:r.validate(ad,Ledger());check.assert_called_once()
 def test_billing_lookup_deadline_includes_credentials_and_read(self):
  from unittest.mock import MagicMock
  broker=r.a.base.transport_module.broker;opener=MagicMock();response=opener.open.return_value.__enter__.return_value
  def credential():
   self.assertGreater(r.signal.getitimer(r.signal.ITIMER_REAL)[0],0);return 'authored-test-placeholder'
  def raw(*_):
   self.assertGreater(r.signal.getitimer(r.signal.ITIMER_REAL)[0],0);return r.e.canonical(lookup(PID)['record'])
  response.read.side_effect=raw
  with patch.object(broker,'credential',side_effect=credential),patch.object(r.urllib.request,'build_opener',return_value=opener):
   value=r.lookup_failed(PID);self.assertEqual(value['request_id'],PID)
  self.assertEqual(r.signal.getitimer(r.signal.ITIMER_REAL)[0],0)
 def test_lookup_not_dispatched_on_unsupported_identity(self):
  with patch.object(r.urllib.request,'build_opener',side_effect=AssertionError('network forbidden')):
   with self.assertRaises(ValueError):r.lookup_failed('../outside')

class NativeContinuation(unittest.TestCase):
 def test_full_verified_prefix_missing_review_ordinary_choice_and_restart(self):
  worker_bin=os.environ.get('TSR_OBJECTIVE_TEST_WORKER_BIN')
  if not worker_bin:self.skipTest('explicit cfg(test) fixture worker required')
  import test_engine_b91_campaign_extension as ext
  helper=ext.EvaluationExtension(methodName='test_prospective_objective_extension_full_dispatch_chain');helper.setUp();self.addCleanup(helper.doCleanups);helper.test_prospective_objective_extension_full_dispatch_chain();ledger=helper.l
  with tempfile.TemporaryDirectory(dir=r.e.ROOT/'target',prefix='authored-continuation-') as tmp:
   root=Path(tmp).resolve();parent=root/'parent';parent.mkdir();(parent/'fixture-only.marker').write_text('objective-native-fixture-only@v1\n')
   packet=r.e.ROOT/'experiments/objective-service-v1';spec=r.read(packet/'project-spec.json');public=r.read(packet/'public.json');conditions=r.read(packet/'conditions.json')['rounds'];source=parent/'source';source.mkdir()
   for group in ('editable','protected'):
    for name,text in spec[group].items():(source/name).write_text(text)
   r.e.private_save(parent/'spec.json',spec);requirements={p['id']:p['meaning'] for p in public['parents']};r.e.private_save(parent/'requirements.json',requirements)
   (parent/'state').mkdir();native=r.d.Native(parent/'state');native.cli('project','init','--spec',str(parent/'spec.json'),'--source',str(source),'--requirements',str(parent/'requirements.json'))
   native.json_cli(('project-lifecycle-open',),{'state_root':native.request()['state_root'],'training':None,'tool_ms_ceiling':300000,'reason':'Authored continuation wiring fixture','non_tool':{'model_calls':24,'model_tokens':983040,'wall_ms':7200000,'patch_bytes':4096,'external_cost_microusd':0}},'--admission')
   native.json_cli(('project','acceptance','feedback'),{'requirements_revision':native.cli('project','acceptance','status')['requirements_revision'],'decision_record':None,'guidance':requirements})
   native.json_cli(('project','objective-configure'),{'state_root':native.request()['state_root'],'conditions':[{'id':v['id'],'input':v['body']} for v in conditions],'service_entry':'service.rs','max_output_bytes':2048,'timeout_ms':1000})
   native.cli('project','resource-unpriced-admit','--state-root',native.request()['state_root'])
   old=r.a.admission(parent/'adapter',r.a.freeze_origin(native,public,'2'*64),ledger.manifest,expiry());ca={'version':'local-project-cycle@v5','journal':str(parent/'cycle'),'scope':old['origin']['scope'],'max_decisions':9,'max_executions':5,'max_service_rounds':3,'expires_at':old['expires_at'],'implementation_sha256':r.d.file_sha(Path(r.loop.__file__))}
   r.e.private_save(parent/'admission.json',old);r.e.private_save(parent/'cycle-admission.json',ca)
   bodies=[];workers=[];failure_time=None
   def transport(body,timeout):
    nonlocal failure_time
    bodies.append(copy.deepcopy(body));print('authored request',len(bodies),flush=True)
    self.assertEqual(body['response_format'],{'type':'json_object'});data=json.loads(body['messages'][1]['content']);context=data.get('context',data)
    self.assertNotIn(spec['protected']['service.rs'],json.dumps(data))
    for later in conditions[context['objective']['context']['round']+1:]:self.assertNotIn(later['body'],json.dumps(context))
    if len(bodies)==6:failure_time=r.d.iso_now();return failed()
    if 'response_schema' in data:
     props=data['response_schema']['properties'];value={k:v['const'] for k,v in props.items() if 'const' in v};stage=data['candidate'].get('objective_stage');value.update(decision='met' if stage=='round_review' else 'accept' if stage=='objective_review' else 'approve',evidence='Authored test assessment, not a model result.',findings={k:{'finding':'met','basis':'static_source','receipts':[],'explanation':'Authored control only','limitations':'No target execution or semantic proof'} for k in data['criteria']})
    else:value={'alternatives':[{'kind':'service','choice':{'rationale':'Authored unchanged option, no preferred real strategy','success_probability':0.5}}]}
    return settled(value)
   def worker_for(controller):
    def worker(view):
     index=controller.cli('project','objective-status')['context']['round'];out=controller.directory.parent/('output-'+str(len(workers))+'.json')
     env={**os.environ,'TSR_OBJECTIVE_FIXTURE_DIRECTORY':str(controller.directory),'TSR_OBJECTIVE_FIXTURE_OUTPUT':str(out),'TSR_OBJECTIVE_FIXTURE_STDOUT':['11,21,31\n','12,21,31\n','12,31\n'][index]}
     result=subprocess.run([worker_bin,'--exact','engine::adaptive::coding::project::objective::tests::fixture_worker','--nocapture'],env=env,capture_output=True,text=True,timeout=90)
     self.assertEqual(result.returncode,0,result.stdout+result.stderr);v=r.read(out);self.assertEqual(v['target_commands_executed'],0);workers.append(index);return v
    return worker
   adapter=r.a.Adapter(old['directory'],old,native,ledger,transport,worker=worker_for(native))
   with self.assertRaises(r.d.Blocked):r.loop.Cycle(ca['journal'],ca,adapter.native,adapter.proposal,adapter.review).run()
   self.assertEqual(workers,[0,1]);self.assertEqual(len(bodies),6)
   r.e.private_save(parent/'CLOSED.json',{'remaining_batch_authority':False,'actual_calls':6,'actual_execution_groups':2,'fixture_only':True})
   r.e.private_save(parent/'provider-exact-lookup.json',lookup(PID,failure_time))
   names=['admission.json','adapter/adapter.jsonl','cycle/cycle.jsonl','CLOSED.json','provider-exact-lookup.json']
   assessment={'decision':'exact_failed_zero','reviewer':'authored independent fixture assessor','artifacts':{str((parent/n).relative_to(r.e.ROOT)):r.d.file_sha(parent/n) for n in names}}
   r.e.private_save(parent/'billing-review.json',assessment)
   done=adapter.read()[2];name,result=[(k,v) for k,v in done.items() if k.startswith('dispatch_')][-1];ident=name.removeprefix('dispatch_');key=r.c.digest({'trial':adapter.trial,'request':ident,'kind':'call'})
   with ledger.locked() as (f,rows):
    bill=next(v for v in rows if v['event'].get('kind')=='bill' and v['event'].get('key')==key)
    event={'kind':'bill_reconciled','key':key,'original_bill_row':bill['hash'],'microusd':0,'basis':'exact_provider_failed_zero','provider_request_id':PID,'lookup_sha256':r.d.file_sha(parent/'provider-exact-lookup.json'),'native_journal_sha256':r.d.file_sha(parent/'cycle/cycle.jsonl'),'provider_journal_sha256':r.d.file_sha(parent/'adapter/adapter.jsonl'),'native_result_sha256':r.e.sha(result),'review_sha256':r.e.sha(assessment),'billing_semantics_sha256':'f'*64}
    ledger.append_locked(f,rows,event)
   r.e.private_save(parent/'billing-reconciled.json',{'event':event})
   prefix=r.inspect_prefix(parent,ledger);frozen=r.tree(parent);before=ledger.status()
   self.assertEqual(prefix['objective']['context']['next'],'round_review')
   # Every frozen artifact/state substitution fails before derivative construction.
   for name in ('provider-exact-lookup.json','billing-review.json','adapter/adapter.jsonl','cycle/cycle.jsonl'):
    file=parent/name;raw=file.read_bytes();file.write_bytes(raw+b' ')
    with self.assertRaises(ValueError):r.verify_parent(prefix,ledger)
    file.write_bytes(raw)
   for field in ('schedule','requirements_revision'):
    changed=copy.deepcopy(prefix);changed['objective']['context'][field]='f'*64
    with self.assertRaisesRegex(ValueError,'substituted'):r.verify_parent(changed,ledger)
   changed=copy.deepcopy(prefix);changed['inherited']['generated'].append({'proposals':[]})
   with self.assertRaisesRegex(ValueError,'substituted'):r.verify_parent(changed,ledger)
   proofpath=parent/'provider-exact-lookup.json';raw=proofpath.read_bytes();proofpath.unlink()
   with self.assertRaises((ValueError,FileNotFoundError)):r.inspect_prefix(parent,ledger)
   proofpath.write_bytes(raw)
   original_fold=ledger.fold
   def substituted(rows):
    status,trials,calls,executions=original_fold(rows)
    changed=copy.deepcopy(executions);key=next(k for k,v in changed.items() if v['trial']==adapter.trial);changed[key]['binding']='f'*64
    return status,trials,calls,changed
   with patch.object(ledger,'fold',side_effect=substituted):
    with self.assertRaisesRegex(ValueError,'execution receipt ledger'):r.inspect_prefix(parent,ledger)
   original_budget=r.d.Native.budget
   for key,value in [('unknown_compute_requests',1),('unknown_usage',2),('model_calls',7)]:
    def bad_budget(controller,key=key,value=value):return {**original_budget(controller),key:value}
    with patch.object(r.d.Native,'budget',bad_budget):
     with self.assertRaisesRegex(ValueError,'unknown action'):r.inspect_prefix(parent,ledger)
   altered=copy.deepcopy(prefix);altered['ledger']['head']='0'*64
   with self.assertRaises(ValueError):r.verify_parent(altered,ledger)
   derivative=root/'derivative';ad,newca=r.prepare(derivative,prefix,ledger,expiry(),retry_allowance=1)
   (derivative/'fixture-only.marker').write_text('objective-native-fixture-only@v1\n');controller=r.d.Native(derivative/'state')
   native_before=controller.cli('project','objective-status');self.assertEqual(native_before,prefix['objective']);self.assertEqual(r.tree(parent),frozen)
   newcalls=[]
   def successful(body,timeout):
    newcalls.append(copy.deepcopy(body))
    if len(newcalls)==1:
     v=failed();v['receipt']['response_request_ids']=['33333333-3333-4333-8333-333333333333'];return v
    return transport(body,timeout)
   def make():return r.Adapter(ad['directory'],ad,controller,ledger,successful,worker=worker_for(controller),failure_lookup=lambda pid:lookup(pid))
   archive=derivative/'parent-evidence/admission.json';raw=archive.read_bytes();archive.write_bytes(raw+b' ')
   with self.assertRaisesRegex(ValueError,'archive drift'):make()
   archive.write_bytes(raw)
   revived=make();v=revived.native('request',None);_,projection=revived.project(v)
   self.assertEqual(v['branch'],'objective_review_required');self.assertEqual(len(projection['objective']['context']['receipts']),2)
   # Cross-root/criteria/schedule mutations cannot enter a request.
   for key in ('scope','state_root','source_snapshot'):
    bad=copy.deepcopy(v);bad[key]='forged'
    with self.assertRaises(ValueError):revived.project(bad)
   def pause(event,name):
    if event=='after_result' and name=='objective_round_review_1_review':raise RuntimeError('authored reviewed-response checkpoint')
   with patch.object(r.time,'sleep'),self.assertRaisesRegex(RuntimeError,'checkpoint'):r.loop.Cycle(newca['journal'],newca,revived.native,revived.proposal,revived.review,hook=pause).run()
   self.assertEqual(len(newcalls),2);self.assertEqual(workers,[0,1])
   revived=make();outcome=r.loop.Cycle(newca['journal'],newca,revived.native,revived.proposal,revived.review).run()
   self.assertEqual(outcome['status'],'objective_final_assessment');self.assertFalse(outcome['acceptance_recorded']);self.assertEqual(len(newcalls),6);self.assertEqual(workers,[0,1,2]);self.assertEqual(r.tree(parent),frozen)
   self.assertEqual(ledger.status()['calls']-before['calls'],6);self.assertEqual(ledger.status()['execution_groups']-before['execution_groups'],1)
   self.assertIsNone(controller.cli('project','objective-status')['context']['final_review'])
   journal=(derivative/'cycle/cycle.jsonl').read_bytes();accounting=ledger.path.read_bytes()
   again=make();self.assertEqual(r.loop.Cycle(newca['journal'],newca,again.native,again.proposal,again.review).run(),outcome)
   self.assertEqual((derivative/'cycle/cycle.jsonl').read_bytes(),journal);self.assertEqual(ledger.path.read_bytes(),accounting);self.assertEqual(len(newcalls),6)
   # Independently attributed authored final assessment is transcribed only to
   # a separate temporary review copy; no historical/live acceptance is mutated.
   frozen_derivative=r.tree(derivative/'state');copydir=root/'authored-review-copy';r.shutil.copytree(derivative/'state',copydir);assessor=r.d.Native(copydir)
   judgment=copy.deepcopy(outcome['judgment']);judgment.update(state_root=assessor.request()['state_root'])
   bad={**judgment,'binding':'f'*64}
   with self.assertRaises(ValueError):assessor.json_cli(('project','objective-accept'),bad)
   accepted=assessor.json_cli(('project','objective-accept'),judgment)
   self.assertEqual(accepted['context']['final_review']['decision'],'accept')
   self.assertEqual(assessor.cli('project','cycle','request')['branch'],'stop')
   self.assertEqual(r.tree(derivative/'state'),frozen_derivative);self.assertEqual(r.tree(parent),frozen);self.assertEqual(ledger.path.read_bytes(),accounting)
   print('continuation authored full flow: 6 new calls including one proven failure retry, 1 stub operation; no completed action repeated; separate-copy authored acceptance',flush=True)


class RetryControls(unittest.TestCase):
 """Authored transport and in-memory journal, not provider evidence."""
 def make(self,*,allowance=1,proof=None):
  obj=object.__new__(r.Adapter);obj.injected_transport=lambda *_:failed();obj.admission={'terminal_retry_allowance':allowance,'prefix':{'provider_request_id':'22222222-2222-4222-8222-222222222222'},'max_calls':18,'expires_at':expiry(),'retry_backoff_seconds':60};obj.wire_kind='review';obj.events={};obj.read=lambda:([],{},obj.events);obj.op=lambda name,fn:obj.events.setdefault(name,fn());obj.failure_lookup=proof or (lambda pid:lookup(pid));return obj
 def test_confirmed_failure_retains_original_and_single_retry(self):
  obj=self.make();body={'model':r.c.MODEL};out=obj._wire(body,240)
  self.assertEqual(out['receipt']['usd'],'0.000000');proof=list(obj.events.values())[0];self.assertIsNone(proof['original_receipt']['usd']);self.assertEqual(proof['lookup']['request_id'],PID)
  ident=next(iter(obj.events)).removeprefix('failed_zero_');obj.events['dispatch_'+ident]={'receipt':out['receipt']}
  with patch.object(r.time,'sleep') as sleep:
   self.assertTrue(obj.retry_terminal_failure(ident,out['receipt']));sleep.assert_called_once()
  with self.assertRaises(r.d.Blocked):obj.retry_terminal_failure(ident,out['receipt'])
  # A second failed request remains unknown/terminal and never becomes format recovery.
  self.assertIsNone(obj._wire(body,240)['receipt']['usd'])
 def test_unknown_wrong_request_reused_id_policy_off_and_late_lookup_never_retry(self):
  wrong=lambda _:lookup('33333333-3333-4333-8333-333333333333')
  for kind in ('unknown','wrong','used','off','slow'):
   obj=self.make(allowance=0 if kind=='off' else 1,proof=wrong if kind=='wrong' else None)
   if kind=='unknown':obj.failure_lookup=lambda _:(_ for _ in ()).throw(OSError('authored unknown'))
   if kind=='used':obj.admission['prefix']['provider_request_id']=PID
   if kind=='slow':
    with patch.object(r.time,'time',side_effect=[0,220]):out=obj._wire({},240)
   else:out=obj._wire({},240)
   self.assertIsNone(out['receipt']['usd'],kind);self.assertFalse(obj.events,kind)
 def test_semantic_response_or_non503_does_not_trigger_lookup(self):
  obj=self.make(proof=lambda _:(_ for _ in ()).throw(AssertionError('lookup forbidden')))
  for result in (settled({'decision':'reject'}),settled({'decision':'abstain'}),{'value':None,'receipt':{'http_status':401}},settled(None)):
   obj.injected_transport=lambda *_,result=result:result
   self.assertEqual(obj._wire({},240),result)
 def test_no_remaining_call_or_deadline_refuses_retry(self):
  for limit in ('calls','deadline'):
   obj=self.make();obj.events['failed_zero_x']={};obj.events['dispatch_x']={}
   if limit=='calls':obj.admission['max_calls']=1
   else:obj.admission['expires_at']=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(seconds=200)).isoformat()
   with self.assertRaises(r.d.Blocked):obj.retry_terminal_failure('x',{})
   self.assertNotIn('terminal_retry_x',obj.events)

# Existing shared Cycle is exercised directly from a completed two-round authored
# state. It does not import or reset historical operation counters into this run.
import test_engine_objective_cycle as controls
class RemainingCycleControls(controls.ObjectiveCycle):
 def setUp(self):
  super().setUp();self.state.update(round=1,phase='round_review',version=4,previous='authored-output-1',receipts=[{'receipt':'authored-receipt-0','output':'authored-output-0','previous':''},{'receipt':'authored-receipt-1','output':'authored-output-1','previous':'authored-output-0'}],round_reviews=[{'authored':True}],service_count=2);self.save();self.a.update(max_decisions=6,max_executions=3,max_service_rounds=1)
 # Inherited tests assume a fresh objective; only the continuation controls below
 # apply to this separate source state.
 for name in list(vars(controls.ObjectiveCycle)):
  if name.startswith('test_'):locals()[name]=None
 def test_resume_only_missing_review_then_ordinary_unchanged_service(self):
  result=self.runner().run();self.assertEqual(result['status'],'objective_final_assessment');self.assertEqual(self.calls,['round_review','proposal','proposal_review','round_review','objective_review']);self.assertEqual(self.effects,[(2,'authored-output-1')]);self.assertFalse(result['acceptance_recorded'])
 def test_abstention_after_review_has_no_new_operation(self):
  self.proposal=lambda _:self.meter('response',{'abstain':'No useful next action in this authored control'})
  self.assertEqual(self.runner().run()['status'],'proposal_abstention');self.assertEqual(self.effects,[])
 def test_unknown_review_does_not_disclose_or_execute_third(self):
  original=self.review
  def unknown(v,response):
   value=original(v,response);value['judgment']['decision']='unknown';return value
  self.review=unknown;self.assertEqual(self.runner().run()['status'],'objective_unknown');self.assertEqual(self.effects,[])
 def test_crash_after_review_effect_never_repeats(self):
  def crash(event,name):
   if event=='after_effect' and name=='objective_round_review_1_admit':raise RuntimeError('uncertain native admission')
  with self.assertRaises(RuntimeError):self.runner(hook=crash).run()
  count=len(self.calls)
  with self.assertRaisesRegex(controls.m.d.Blocked,'uncertain'):self.runner().run()
  self.assertEqual(len(self.calls),count);self.assertEqual(self.effects,[])
 def test_restart_after_review_before_admission_reuses_review(self):
  def pause(event,name):
   if event=='after_result' and name=='objective_round_review_1_review':raise RuntimeError('checkpoint')
  with self.assertRaises(RuntimeError):self.runner(hook=pause).run()
  self.assertEqual(self.calls,['round_review']);self.assertEqual(self.runner().run()['status'],'objective_final_assessment');self.assertEqual(self.calls.count('round_review'),2)

if __name__=='__main__':unittest.main()
