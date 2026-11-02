"""Authored disposal/accounting fixtures only; no model or game effects."""
import importlib.util,json,hashlib,tempfile,unittest,fcntl,os
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
s=importlib.util.spec_from_file_location('qcampaign',Path(__file__).with_name('engine-b91-campaign.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
d=m.load_module('qdriver','engine-b91-driver.py')
class Quarantine(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name).resolve();self.adapter=self.root/'adapter';self.adapter.mkdir();self.request='a'*64
  self.ad={'version':'browser-project-cycle-adapter@v7','model':'gpt-6-luna','directory':str(self.adapter)}
  self.body={'model':'gpt-6-luna','stream':False,'max_completion_tokens':32768,'response_format':{'type':'json_object'},'messages':[{'role':'user','content':'AUTHORED quarantine fixture'}]}
  self.source={k:ch*64 for k,ch in [('state_root','b'),('source_snapshot','c'),('scope','d')]}
  self.trial=d.e.sha({'trial':str(self.adapter),'admission':d.e.sha(self.ad)})
  self.man=m.manifest('e'*64,self.root/'ledger',fixture=True);self.ledger=m.Ledger(self.root/'ledger',self.man,m.digest(self.man),create=True);self.ledger.enroll(self.trial,d.e.sha(self.ad))
  self.key=self.ledger.reserve('call',self.trial,self.request,m.digest(self.body));self.write_journal()
 def write_journal(self,body=None,extra_pending=None,completed=False):
  body=body or self.body;rows=[d.Driver.row([],{'kind':'admission','value':self.ad,'fixture_only':True})]
  def add(e):rows.append(d.Driver.row(rows,e))
  def done(name,value):
   intent={'kind':'intent','name':name};add(intent);add({'kind':'result','name':name,'intent':d.e.sha(intent),'value':value})
  done('request_body_'+self.request,{'body':body,'body_sha256':d.e.sha(body),'source':self.source});done('reserve_'+self.request,{'reserved':True})
  if completed:done('dispatch_'+self.request,{'candidate':'never accepted'})
  else:add({'kind':'intent','name':'dispatch_'+self.request})
  if extra_pending:add({'kind':'intent','name':extra_pending})
  self.raw=b'\n'.join(d.e.canonical(r) for r in rows)+b'\n';(self.adapter/'adapter.jsonl').write_bytes(self.raw);self.prefix=hashlib.sha256(self.raw).hexdigest()
  self.review={'decision':'dispose_output_preserve_financial_unknown','journal_sha256':self.prefix,'request_body_sha256':d.e.sha(body),'reviewer':'authored independent review fixture'}
 def quarantine(self,**kw):
  # Authored proof substitute only for isolated ledger fault tests; separate
  # read-only historical preflight exercises the real native proof.
  module=m.load_module('qproof','engine-generation-quarantine.py')
  with patch.object(module,'native_proof',lambda *args:None),patch.object(m,'load_module',return_value=module):
   return self.ledger.quarantine_generation(self.adapter,self.request,self.prefix,self.review,**kw)
 def test_dry_run_append_restart_and_duplicate(self):
  old=self.ledger.path.read_bytes();self.quarantine(dry_run=True);self.assertEqual(old,self.ledger.path.read_bytes())
  event=self.quarantine();self.assertEqual(self.quarantine(),event);s=self.ledger.status();self.assertEqual(s['unknown_calls'],[self.key]);self.assertEqual(s['quarantined_unknown_calls'],[self.key]);self.assertEqual(s['known_cost_microusd'],0);self.assertEqual(s['committed_microusd'],50000);self.assertTrue(s['stopped'])
  self.assertEqual((self.adapter/'adapter.jsonl').read_bytes(),self.raw)
  again=m.Ledger(self.ledger.path,self.man,m.digest(self.man));self.assertEqual(again.status(),s)
  with self.assertRaises(ValueError):again.require_output_eligible(self.trial)
  with self.assertRaises(ValueError):again.settle('bill',self.key,0)
  with self.assertRaises(ValueError):again.reserve('call',self.trial,'fresh',m.digest(self.body))
 def test_tool_bearing_or_substituted_request_rejected(self):
  for b in [{**self.body,'tools':[]},{**self.body,'messages':[{'role':'user','content':[{'type':'text','text':'fixture'}]}]},{**self.body,'messages':[{'role':'user','content':'different body'}]}]:
   self.write_journal(body=b)
   with self.assertRaises(ValueError):self.quarantine()
 def test_incomplete_execution_and_completed_generation_rejected(self):
  self.write_journal(extra_pending='execution_fixture')
  with self.assertRaises(ValueError):self.quarantine()
  self.write_journal(completed=True)
  with self.assertRaises(ValueError):self.quarantine()
 def test_changed_prefix_review_or_source_rejected(self):
  (self.adapter/'adapter.jsonl').write_bytes(self.raw+b'\n')
  with self.assertRaises(ValueError):self.quarantine()
  self.write_journal();self.review['journal_sha256']='f'*64
  with self.assertRaises(ValueError):self.quarantine()
  self.write_journal();self.source['state_root']='invalid';self.write_journal()
  with self.assertRaises(ValueError):self.quarantine()
 def test_active_launch_refused(self):
  with open(self.adapter/'launch.lock','w') as f:
   fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
   with self.assertRaises(BlockingIOError):self.quarantine()
 def test_outer_launcher_lock_refused(self):
  with open(self.adapter.parent/'launch.lock','w') as f:
   fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
   with self.assertRaises(BlockingIOError):self.quarantine()
 def test_journal_changed_at_lock_acquisition_rejected(self):
  original=fcntl.flock
  def race(fd,flags):
   if flags==fcntl.LOCK_EX|fcntl.LOCK_NB:(self.adapter/'adapter.jsonl').write_bytes(self.raw+b'\n')
   return original(fd,flags)
  with patch.object(fcntl,'flock',side_effect=race):
   with self.assertRaisesRegex(ValueError,'prefix changed'):self.quarantine()
 def test_unknown_execution_and_financial_unlock_rejected(self):
  with self.ledger.locked() as (f,rows):
   key=m.digest({'trial':self.trial,'request':'synthetic-execution','kind':'execution'})
   self.ledger.append_locked(f,rows,{'kind':'execution','key':key,'trial':self.trial,'binding':'f'*64})
  with self.assertRaises(ValueError):self.quarantine()
 def test_event_cannot_forge_financial_settlement(self):
  event=self.quarantine(dry_run=True);event['value']['financial_status']='verified'
  with self.ledger.locked() as (_,rows):
   with self.assertRaises(ValueError):self.ledger.fold(rows+[self.ledger.row(rows,event)])
 def test_adapter_native_and_generated_guard_late_output(self):
  self.quarantine();base=m.load_module('qbase','engine-project-cycle-adapter.py');web=m.load_module('qweb','engine-browser-adapter.py');obj=SimpleNamespace(ledger=self.ledger,trial=self.trial)
  with self.assertRaises(ValueError):base.Adapter.generated(obj)
  with self.assertRaises(ValueError):base.Adapter.native(obj,'advance',{'late':'output'})
  with self.assertRaises(ValueError):web.Adapter.native(obj,'advance',{'late':'output'})
if __name__=='__main__':unittest.main()

class RiskAcceptance(Quarantine):
 def accept(self,mutate=None):
  import datetime
  proof=self.quarantine();status=self.ledger.status();t=datetime.datetime.now(datetime.timezone.utc)
  disclosure={'unknown_key':self.key,'proof':proof,'manifest_sha256':self.ledger.hash,'max_calls':status['effective_max_calls'],'max_execution_groups':status['effective_max_execution_groups'],'max_committed_microusd':status['effective_cap_microusd'],'baseline_committed_microusd':status['committed_microusd']}
  if mutate:mutate(disclosure)
  decision={'choice':'approve','disclosure_sha256':m.digest(disclosure),'at':t.isoformat(),'expires_at':(t+datetime.timedelta(minutes=120)).isoformat()}
  return self.ledger.accept_generation_risk(self.key,decision,disclosure)
 def test_explicit_approval_retains_unknown_and_old_output_block(self):
  self.accept();s=self.ledger.status();self.assertFalse(s['stopped']);self.assertEqual(s['unknown_calls'],[self.key]);self.assertEqual(s['committed_microusd'],50000)
  with self.assertRaises(ValueError):self.ledger.require_output_eligible(self.trial)
  other='f'*64;self.ledger.enroll(other,'e'*64);self.ledger.reserve('call',other,'fresh','a'*64)
  self.assertTrue(self.ledger.status()['stopped']) # new unknown is never covered
 def test_smaller_or_substituted_disclosure_refused(self):
  with self.assertRaisesRegex(ValueError,'allowance'):self.accept(lambda d:d.update(max_calls=99))
 def test_expiry_stops_new_effects_without_erasing_unknown(self):
  self.accept()
  with patch.object(m.time,'time',return_value=m.time.time()+7201):
   self.assertTrue(self.ledger.status()['stopped'])
   with self.assertRaises(ValueError):self.ledger.reserve('call','f'*64,'fresh','a'*64)
