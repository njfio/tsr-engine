"""Authored accounting-only partial-disposition regressions; no real effects."""
import copy,importlib.util,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('prior_disposition_tests',Path(__file__).with_name('test_engine_execution_disposition_campaign.py'));old=importlib.util.module_from_spec(s);s.loader.exec_module(old)
class Partial(old.Disposition):
 def setUp(self):
  super().setUp()
  from contextlib import nullcontext
  self.helper.locked_source=lambda _:nullcontext()
 def apply(self):return self.ledger.reconcile_execution_partial(self.key,self.evidence)
 def test_exact_append_retains_unknown_and_consumed_count_restart(self):
  before=self.path.read_bytes();status=self.apply();self.assertTrue(self.path.read_bytes().startswith(before));self.assertEqual(status['execution_groups'],1);self.assertEqual(status['partial_observation_unavailable'],[self.key]);self.assertEqual(status['execution_not_started'],[]);self.assertEqual(status['historical_unknown_executions'],[self.key]);self.assertFalse(status['stopped'])
  again=old.m.Ledger(self.path,self.man,old.m.digest(self.man));self.assertEqual(again.status(),status)
  with self.assertRaises(ValueError):self.apply()
  with self.assertRaises(ValueError):again.reserve('execution',self.trial,self.request,self.ad)
  with self.assertRaises(ValueError):again.reconcile_execution_not_started(self.key,self.evidence,self.review)
 def test_cross_key_source_old_return_and_review_rejected(self):
  before=self.path.read_bytes()
  for name in self.binding:
   bad=copy.deepcopy(self.evidence);bad['binding'][name]='f'*64
   with self.assertRaises(ValueError):self.ledger.reconcile_execution_partial(self.key,bad)
  self.assertEqual(self.path.read_bytes(),before)
 def test_ambiguous_and_current_proof_failure_do_not_append(self):
  before=self.path.read_bytes();self.helper.validate_current_evidence=lambda *_:(_ for _ in ()).throw(ValueError('unknown writable tail'))
  with self.assertRaises(ValueError):self.apply()
  self.assertEqual(self.path.read_bytes(),before)

class HostProof(unittest.TestCase):
 def setUp(self):
  import tempfile,json,hashlib
  from unittest.mock import patch
  from types import SimpleNamespace
  spec=importlib.util.spec_from_file_location('real_partial_helper',Path(__file__).with_name('engine-partial-execution.py'));self.h=importlib.util.module_from_spec(spec);spec.loader.exec_module(self.h)
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name);(self.root/'native').mkdir();(self.root/'adapter').mkdir();(self.root/'cycle').mkdir()
  self.sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();self.json=json
  self.old=self.root/'historical-binary';self.old.write_bytes(b'authored binary identity; never executed')
  code={}
  for name in ('engine-project-cycle-adapter.py','engine-browser-adapter.py','engine-b91-campaign.py'):
   path=self.root/name;path.write_text('authored historical fixture');code[name]=str(path)
  self.ad={'origin':{'directory':str(self.root/'native')},'binary_sha256':self.sha(self.old),'implementation':{name:self.sha(Path(path)) for name,path in code.items()}}
  (self.root/'adapter-admission.json').write_text(json.dumps(self.ad));(self.root/'cycle-admission.json').write_text('{}')
  trial=self.h.digest({'trial':str(self.root/'adapter'),'admission':self.h.digest(self.ad)});binding='a'*64
  request=self.h.digest({'admission':self.h.digest(self.ad),'binding':binding});key=self.h.digest({'trial':trial,'request':request,'kind':'execution'})
  self.b={'key':key,'trial':trial,'admission_sha256':self.h.digest(self.ad),'request_id':request,'original_return_row':'b'*64}
  self.write_chain(self.root/'adapter/adapter.jsonl',[{'kind':'admission','value':self.ad},{'kind':'intent','name':'execution_'+request}])
  intent={'kind':'intent','name':'advance_0'}
  self.next={'branch':'resume','binding':binding,'plan':{'commitment':'c'*64,'expected_after':'d'*64}}
  self.ce=[{'kind':'admission','value':{}},intent,{'kind':'result','name':'advance_0','intent':self.h.digest(intent),'value':{'next':self.next}},{'kind':'intent','name':'resume_0'}]
  self.write_chain(self.root/'cycle/cycle.jsonl',self.ce)
  self.r={'disposition':'e'*64,'commitment':'c'*64,'source_after':'d'*64,'receipts':['1'*64,'2'*64],'evidence':{'version':'partial-readonly-tail-proof@v1','directory':str(self.root/'native'),'commitment':'c'*64,'source_after':'d'*64,'attestation':{'native_pid':42,'outer_pid':43,'processes_absent':True,'containers_absent':True,'container_prefix':'tsr-m3-42-'}},'review':{'decision':'partial_observation_unavailable','reviewer':'authored separate review','report':'authored test only'},'observation':'unavailable','duration':'unmeasured_full_reservation_charged','replayed':False}
  binary=Path(__file__).resolve().parents[1]/'target/debug/tsr'
  self.v={'version':'partial-execution-disposition@v1','binding':self.b,'native_directory':str(self.root/'native'),'native_receipt':self.r,'binary':str(binary),'binary_sha256':self.sha(binary),'execution_binary':str(self.old),'historical_implementation':code,'artifacts':{}}
  self.refresh()
  self.p1=patch.object(self.h.os,'kill',side_effect=ProcessLookupError);self.p1.start();self.addCleanup(self.p1.stop)
  self.p2=patch.object(self.h.subprocess,'run',side_effect=lambda cmd,**_:SimpleNamespace(stdout=b'' if 'docker' in cmd[0] else json.dumps([self.r]).encode()));self.p2.start();self.addCleanup(self.p2.stop)
 def write_chain(self,path,events):
  rows=[]
  for e in events:
   row={'seq':len(rows),'previous':self.h.digest(rows[-1]) if rows else None,'event':e};rows.append({**row,'hash':self.h.digest(row)})
  path.write_text('\n'.join(self.json.dumps(r) for r in rows)+'\n')
 def refresh(self):
  self.v['artifacts']={str(self.root/p):self.sha(self.root/p) for p in ['adapter-admission.json','cycle-admission.json','adapter/adapter.jsonl','cycle/cycle.jsonl']}
 def test_exact_proof_and_source_locks(self):
  with self.h.locked_source(self.v):
   self.assertEqual(self.h.validate_current_evidence(self.v),self.b)
   with self.assertRaises(BlockingIOError):
    with self.h.locked_source(self.v):pass
 def test_cross_commitment_cycle_binding_directory_and_live_worker(self):
  for change in [lambda v:v['native_receipt'].update(commitment='f'*64),lambda v:v.update(native_directory=str(self.root/'other'))]:
   v=copy.deepcopy(self.v);change(v)
   with self.assertRaises(ValueError):self.h.validate_current_evidence(v)
  self.next['binding']='f'*64;self.write_chain(self.root/'cycle/cycle.jsonl',self.ce);self.refresh()
  with self.assertRaisesRegex(ValueError,'pre-execution'):self.h.validate_current_evidence(self.v)
 def test_historical_binary_and_artifact_tamper(self):
  self.old.write_bytes(b'wrong binary')
  with self.assertRaisesRegex(ValueError,'historical executed'):self.h.validate_current_evidence(self.v)

 def test_live_worker_and_container_block(self):
  from unittest.mock import patch
  from types import SimpleNamespace
  with patch.object(self.h.os,'kill',return_value=None):
   with self.assertRaisesRegex(ValueError,'process remains'):self.h.validate_current_evidence(self.v)
  with patch.object(self.h.subprocess,'run',return_value=SimpleNamespace(stdout=b'tsr-m3-42-2-123\n')):
   with self.assertRaisesRegex(ValueError,'container remains'):self.h.validate_current_evidence(self.v)
if __name__=='__main__':unittest.main()
