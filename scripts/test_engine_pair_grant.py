"""Authored paired accounting boundaries, no provider effects."""
import copy,importlib.util,tempfile,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('pair_campaign',Path(__file__).with_name('engine-b91-campaign.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Pair(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);p=Path(self.tmp.name)/'ledger';self.manifest=m.manifest('a'*64,p,fixture=True);self.ledger=m.Ledger(p,self.manifest,m.digest(self.manifest),create=True)
  with self.ledger.locked() as (_,rows):head=rows[-1]['hash']
  self.grant={'version':m.PAIR_GRANT,'manifest_sha256':self.ledger.hash,'ledger_prefix_head':head,'previous_max_calls':100,'max_calls':100,'previous_max_execution_groups':40,'max_execution_groups':40,'cap_microusd':m.CAP,'reservation_microusd':m.RESERVE,'authority_sha256':'b'*64,'reviewer':'authored independent review fixture','suite_manifest_sha256':'c'*64,'source_class':'newly-authored-nonprivate-browser-project','session_trial_sha256':'d'*64,'session_admission_sha256':'e'*64,'baseline':{'calls':0,'execution_groups':0,'settled_microusd':0},'session_max_calls':4,'session_max_execution_groups':2,'session_cap_microusd':200000,'arms':[{'trial':x*64,'admission':y*64,'suite':z*64,'max_calls':2,'max_operations':1,'cap_microusd':100000} for x,y,z in [('1','2','3'),('4','5','6')]]}
 def test_equal_fresh_grant_and_both_arms_enroll(self):
  self.ledger.grant_calls(self.grant,m.digest(self.grant))
  for a in self.grant['arms']:self.ledger.enroll(a['trial'],a['admission'],suite_sha256=a['suite'])
  self.assertEqual(self.ledger.status()['calls'],0)
  with self.assertRaises(ValueError):self.ledger.enroll('7'*64,'8'*64,suite_sha256='9'*64)
 def test_unequal_duplicate_stale_and_reopened_fail(self):
  for mutate in [lambda g:g['arms'][0].update(max_calls=3),lambda g:g['arms'][1].update(trial=g['arms'][0]['trial']),lambda g:g.update(ledger_prefix_head='0'*64)]:
   g=copy.deepcopy(self.grant);mutate(g)
   with self.assertRaises(ValueError):self.ledger.grant_calls(g,m.digest(g))
 def test_live_capacity_preflight_exact_arms_no_network(self):
  import time
  path=Path(self.tmp.name)/'live-preflight-ledger'
  manifest=m.manifest('a'*64,path,fixture=False)
  self.ledger=m.Ledger(path,manifest,m.digest(manifest),create=True)
  with self.ledger.locked() as (_,rows):self.grant.update(manifest_sha256=self.ledger.hash,ledger_prefix_head=rows[-1]['hash'])
  self.ledger.grant_calls(self.grant,m.digest(self.grant))
  for a in self.grant['arms']:self.ledger.enroll(a['trial'],a['admission'],suite_sha256=a['suite'])
  quote={'version':'b91-price-quote@v2','model':m.MODEL,'input_token_price_threshold':-1,'rates':{k:'0.01' for k in m.PRICE_CEILINGS},'ceilings':dict(m.PRICE_CEILINGS),'retrieved_at':time.time(),'catalog_sha256':'a'*64}
  self.ledger.record_price(quote)
  body={'model':m.MODEL,'max_completion_tokens':32768,'stream':False,'response_format':{'type':'json_object'},'messages':[{'role':'user','content':'Authored preflight only'}]}
  before=self.ledger.path.read_bytes()
  for a in self.grant['arms']:
   result=self.ledger.run_call(a['trial'],'f'*64,body,240,lambda *x:self.fail('no network'),payload_limit=m.capacity.PAYLOAD_BYTES,preflight_only=True)
   self.assertTrue(result['admissible'])
  with self.assertRaises(ValueError):self.ledger.run_call('9'*64,'f'*64,body,240,None,payload_limit=m.capacity.PAYLOAD_BYTES,preflight_only=True)
  self.assertEqual(before,self.ledger.path.read_bytes())
 def test_swapped_bindings_and_per_arm_calls_and_unknown_reservations(self):
  self.ledger.grant_calls(self.grant,m.digest(self.grant))
  a,b=self.grant['arms']
  for admission,suite in [(b['admission'],a['suite']),(a['admission'],b['suite'])]:
   with self.assertRaises(ValueError):self.ledger.enroll(a['trial'],admission,suite_sha256=suite)
  for arm in (a,b):self.ledger.enroll(arm['trial'],arm['admission'],suite_sha256=arm['suite'])
  first=self.ledger.reserve('call',a['trial'],'a'*64,'b'*64)
  self.assertTrue(self.ledger.status()['stopped'])
  with self.assertRaises(ValueError):self.ledger.reserve('call',b['trial'],'b'*64,'b'*64)
  self.ledger.settle('bill',first,0)
  second=self.ledger.reserve('call',a['trial'],'c'*64,'b'*64);self.ledger.settle('bill',second,0)
  with self.assertRaises(ValueError):self.ledger.reserve('call',a['trial'],'d'*64,'b'*64)
  third=self.ledger.reserve('call',b['trial'],'e'*64,'b'*64);self.ledger.settle('bill',third,0)
  self.assertEqual(self.ledger.status()['calls'],3)
 def test_one_arm_cannot_borrow_other_operation_allowance(self):
  self.ledger.grant_calls(self.grant,m.digest(self.grant))
  for a in self.grant['arms']:self.ledger.enroll(a['trial'],a['admission'],suite_sha256=a['suite'])
  a,b=self.grant['arms'];self.ledger.run_execution(a['trial'],'a'*64,a['admission'],lambda:{'ok':True})
  with self.assertRaises(ValueError):self.ledger.run_execution(a['trial'],'b'*64,a['admission'],lambda:self.fail('must not execute'))
  self.ledger.run_execution(b['trial'],'c'*64,b['admission'],lambda:{'ok':True});self.assertEqual(self.ledger.status()['execution_groups'],2)
if __name__=='__main__':unittest.main()
