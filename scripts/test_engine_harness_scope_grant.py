"""Authored scope-only sessions; callbacks are local fakes, never paid effects."""
import copy, importlib.util, json, os, tempfile, unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('harness_campaign',Path(__file__).with_name('engine-b91-campaign.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class HarnessGrant(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
  self.path=Path(self.tmp.name)/'ledger';self.manifest=m.manifest('a'*64,self.path,fixture=True)
  self.l=m.Ledger(self.path,self.manifest,m.digest(self.manifest),create=True)
  self.old='b'*64;self.l.enroll(self.old,'c'*64)
  self.body={'model':m.MODEL,'stream':False,'max_completion_tokens':8192,'messages':[{'role':'user','content':'Authored public fixture'}]}
  self.call(self.old,'initial')
  self.l.run_execution(self.old,'first','c'*64,lambda:{'fixture':True})
 def call(self,trial,name,body=None,payload_limit=32768):
  return self.l.run_call(trial,name,body or self.body,240,lambda *_:{'value':{},'receipt':{'status':'settled','usd':'0.000001','usage':{'prompt_tokens':10,'completion_tokens':20}}},payload_limit=payload_limit)
 def grant(self,trial='d'*64):
  status=self.l.status();rows=[json.loads(x) for x in self.path.read_text().splitlines()]
  return {'version':m.HARNESS_GRANT,'session_trial_sha256':trial,'session_admission_sha256':'e'*64,
   'manifest_sha256':self.l.hash,'ledger_prefix_head':rows[-1]['hash'],
   'previous_max_calls':status['effective_max_calls'],'max_calls':status['effective_max_calls'],
   'previous_max_execution_groups':status['effective_max_execution_groups'],'max_execution_groups':status['effective_max_execution_groups'],
   'cap_microusd':status['effective_cap_microusd'],'reservation_microusd':m.RESERVE,'authority_sha256':'f'*64,
   'reviewer':'Independent authored fixture reviewer','suite_manifest_sha256':'1'*64,
   'source_class':'newly-authored-nonprivate-browser-project',
   'baseline':{'calls':status['calls'],'execution_groups':status['execution_groups'],'settled_microusd':status['known_cost_microusd']},
   'session_max_calls':3,'session_max_execution_groups':1,'session_cap_microusd':150000}
 def apply(self,g=None):
  g=g or self.grant();self.l.grant_calls(g,m.digest(g));return g
 def enroll(self,g):self.l.enroll(g['session_trial_sha256'],g['session_admission_sha256'],suite_sha256=g['suite_manifest_sha256'])
 def test_scoped_session_preserves_history_caps_and_supports_context(self):
  before=self.path.read_bytes();g=self.apply();self.enroll(g)
  self.call(g['session_trial_sha256'],'large',{**self.body,'max_completion_tokens':32768},m.capacity.PAYLOAD_BYTES)
  self.assertTrue(self.path.read_bytes().startswith(before));st=self.l.status()
  self.assertEqual((st['calls'],st['execution_groups'],st['known_cost_microusd']),(2,1,2))
  self.assertEqual((st['effective_max_calls'],st['effective_max_execution_groups'],st['effective_cap_microusd']),(100,40,5000000))
  restarted=m.Ledger(self.path,self.manifest,m.digest(self.manifest));self.assertEqual(restarted.status(),st)
 def test_old_trial_wrong_admission_suite_and_duplicate_effect_refused(self):
  g=self.apply()
  with self.assertRaises(ValueError):self.call(self.old,'late')
  for admission,suite in [('2'*64,g['suite_manifest_sha256']),(g['session_admission_sha256'],'2'*64)]:
   with self.assertRaises(ValueError):self.l.enroll(g['session_trial_sha256'],admission,suite_sha256=suite)
  self.enroll(g);self.call(g['session_trial_sha256'],'new')
  with self.assertRaises(ValueError):self.call(g['session_trial_sha256'],'new')
 def test_no_new_budget_stale_prefix_or_source_scope(self):
  g=self.grant();before=self.path.read_bytes()
  changes=[{'max_calls':101},{'max_execution_groups':41},{'cap_microusd':5000001},{'ledger_prefix_head':'0'*64},{'source_class':'private-data'},{'baseline':{**g['baseline'],'settled_microusd':2}},{'session_max_calls':100},{'session_cap_microusd':5000000},{'session_trial_sha256':'bad'}]
  for change in changes:
   bad={**g,**change}
   with self.assertRaises(ValueError):self.apply(bad)
  self.assertEqual(self.path.read_bytes(),before)
 def test_new_session_cannot_reopen_prior_even_without_enrollment(self):
  g=self.apply()
  with self.assertRaises(ValueError):self.apply(self.grant(g['session_trial_sha256']))
  h=self.apply(self.grant('2'*64));self.enroll(h)
  with self.assertRaises(ValueError):self.enroll(g)
  self.call(h['session_trial_sha256'],'second')
 def test_unknown_call_and_execution_block_scope_replacement(self):
  for kind in ('call','execution'):
   with self.subTest(kind=kind):
    key=self.l.reserve(kind,self.old,kind,'2'*64)
    with self.assertRaises(ValueError):self.apply()
    self.l.settle('bill' if kind=='call' else 'return',key,1 if kind=='call' else 'returned')
  self.apply()
 def test_session_counts_and_reservation_cost_are_enforced(self):
  g=self.apply();self.enroll(g);trial=g['session_trial_sha256']
  for i in range(3):self.call(trial,str(i))
  with self.assertRaises(ValueError):self.call(trial,'fourth')
  self.l.run_execution(trial,'one',g['session_admission_sha256'],lambda:{'fixture':True})
  with self.assertRaises(ValueError):self.l.run_execution(trial,'two',g['session_admission_sha256'],lambda:self.fail('never execute'))
 def test_context_reservation_cannot_exceed_session_dollars(self):
  g=self.apply();self.enroll(g)
  # A quoted request can legitimately reserve above the per-call floor. The
  # aggregate session ceiling is still checked before the callback.
  q={'version':'b91-price-quote@v2','model':m.MODEL,'rates':{'input_per_million':'0.065000','output_per_million':'0.325000','cache_read_per_million':'0.065000','cache_write_per_million':'0.065000'},'ceilings':m.PRICE_CEILINGS,'input_token_price_threshold':-1,'retrieved_at':m.time.time(),'catalog_sha256':'3'*64}
  self.l.record_price(q)
  body={**self.body,'max_completion_tokens':32768,'messages':[{'role':'user','content':'a'*900000}]}
  with self.assertRaises(ValueError):self.l.run_call(g['session_trial_sha256'],'cost',body,240,lambda *_:self.fail('no dispatch'),payload_limit=m.capacity.PAYLOAD_BYTES)
  self.assertEqual(self.l.status()['calls'],1)
 def test_operational_capacity_checks_use_exact_new_trial(self):
  # The transport remains an authored callback; fixture_only=False exercises
  # the real capacity-admission branch without any network/provider use.
  self.path=Path(self.tmp.name)/'operational-shape-ledger'
  self.manifest=m.manifest('4'*64,self.path,fixture=False)
  self.l=m.Ledger(self.path,self.manifest,m.digest(self.manifest),create=True)
  g=self.apply();self.enroll(g)
  q={'version':'b91-price-quote@v2','model':m.MODEL,'rates':{'input_per_million':'0.065000','output_per_million':'0.325000','cache_read_per_million':'0.065000','cache_write_per_million':'0.065000'},'ceilings':m.PRICE_CEILINGS,'input_token_price_threshold':-1,'retrieved_at':m.time.time(),'catalog_sha256':'3'*64}
  self.l.record_price(q)
  body={**self.body,'max_completion_tokens':32768}
  self.call(g['session_trial_sha256'],'capacity',body,m.capacity.PAYLOAD_BYTES)
  with self.assertRaises(ValueError):self.call('5'*64,'foreign',body,m.capacity.PAYLOAD_BYTES)
  self.assertEqual(self.l.status()['calls'],1)
 def test_overrun_cannot_be_hidden_with_new_scope(self):
  self.l.run_call(self.old,'overrun',self.body,240,lambda *_:{'receipt':{'status':'settled','usd':'0.051'}})
  self.assertTrue(self.l.status()['overrun'])
  with self.assertRaises(ValueError):self.apply()

class FrozenHistory(unittest.TestCase):
 @unittest.skipUnless(os.environ.get('TSR_BROWSER_HARNESS_LEDGER'),'explicit read-only historical ledger path')
 def test_real_history_in_memory_only(self):
  p=Path(os.environ['TSR_BROWSER_HARNESS_LEDGER']);before=p.read_bytes();rows=[m.strict(x) for x in before.splitlines()]
  ledger=object.__new__(m.Ledger);ledger.manifest=rows[0]['event']['value'];ledger.hash=m.digest(ledger.manifest)
  status=ledger.fold(rows)[0]
  g={'version':m.HARNESS_GRANT,'session_trial_sha256':'d'*64,'session_admission_sha256':'e'*64,'manifest_sha256':ledger.hash,'ledger_prefix_head':rows[-1]['hash'],'previous_max_calls':status['effective_max_calls'],'max_calls':status['effective_max_calls'],'previous_max_execution_groups':status['effective_max_execution_groups'],'max_execution_groups':status['effective_max_execution_groups'],'cap_microusd':status['effective_cap_microusd'],'reservation_microusd':m.RESERVE,'authority_sha256':'f'*64,'reviewer':'Authored in-memory verification only','suite_manifest_sha256':'1'*64,'source_class':'newly-authored-nonprivate-browser-project','baseline':{'calls':status['calls'],'execution_groups':status['execution_groups'],'settled_microusd':status['known_cost_microusd']},'session_max_calls':12,'session_max_execution_groups':4,'session_cap_microusd':600000}
  prospective=rows+[m.Ledger.row(rows,{'kind':'evaluation_grant','key':m.digest(g),'value':g})]
  after=ledger.fold(prospective)[0]
  for key in ('calls','execution_groups','known_cost_microusd','effective_cap_microusd','effective_max_calls','effective_max_execution_groups'):self.assertEqual(after[key],status[key])
  self.assertEqual(p.read_bytes(),before)

if __name__=='__main__':unittest.main()
