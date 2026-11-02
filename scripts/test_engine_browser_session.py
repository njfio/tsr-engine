"""Read-only historical fold + authored prospective events; never writes a grant."""
import importlib.util,json,os,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('campaign',Path(__file__).with_name('engine-b91-campaign.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
@unittest.skipUnless(os.getenv('TSR_BROWSER_SESSION_LEDGER'),'explicit frozen ledger input')
class Session(unittest.TestCase):
 def setUp(self):
  self.rows=[json.loads(x) for x in Path(os.environ['TSR_BROWSER_SESSION_LEDGER']).read_text().splitlines()]
  self.l=object.__new__(m.Ledger);self.l.manifest=self.rows[0]['event']['value'];self.l.hash=m.digest(self.l.manifest)
  self.g={'version':'native-browser-session@v1','session_trial_sha256':'d'*64,'session_admission_sha256':'e'*64,'manifest_sha256':self.l.hash,'ledger_prefix_head':self.rows[-1]['hash'],'previous_max_calls':272,'max_calls':462,'previous_max_execution_groups':74,'max_execution_groups':172,'cap_microusd':10225523,'reservation_microusd':50000,'authority_sha256':'a'*64,'reviewer':'Authored test, not actual admission','suite_manifest_sha256':'b'*64,'source_class':'newly-authored-nonprivate-browser-project'}
 def append(self,rows,event):return rows+[m.Ledger.row(rows,event)]
 def grant(self,g=None):
  g=g or self.g;return self.append(self.rows,{'kind':'evaluation_grant','key':m.digest(g),'value':g})
 def test_exact_session_retains_history_and_ten_dollar_reservation(self):
  st=self.l.fold(self.grant())[0]
  self.assertEqual((st['calls'],st['execution_groups'],st['known_cost_microusd']),(262,72,225523))
  self.assertEqual((st['remaining_calls'],st['remaining_execution_groups']),(200,100))
  self.assertEqual(st['effective_cap_microusd']-st['committed_microusd'],10000000)
  self.assertEqual(200*m.RESERVE,10000000)
 def test_cap_source_prefix_and_duplicate_grant_reject(self):
  for patch in [{'max_calls':463},{'max_execution_groups':173},{'cap_microusd':10225524},{'ledger_prefix_head':'c'*64}]:
   with self.assertRaises(ValueError):self.l.fold(self.grant({**self.g,**patch}))
  rows=self.grant();g={**self.g,'ledger_prefix_head':rows[-1]['hash']}
  with self.assertRaises(ValueError):self.l.fold(self.append(rows,{'kind':'evaluation_grant','key':m.digest(g),'value':g}))
 def test_unknown_reservation_stops_and_prior_trials_cannot_spend_session(self):
  rows=self.grant();trial='d'*64
  rows=self.append(rows,{'kind':'trial','key':trial,'admission':'e'*64,'evaluation_suite_sha256':'b'*64})
  rows=self.append(rows,{'kind':'call','key':'f'*64,'trial':trial,'binding':'1'*64,'reserved_microusd':50000})
  st=self.l.fold(rows)[0];self.assertTrue(st['stopped']);self.assertEqual(st['committed_microusd'],275523)
  _,trials,_,_=self.l.fold(self.rows);old=next(iter(trials))
  with self.assertRaises(ValueError):self.l.fold(self.append(self.grant(),{'kind':'call','key':'2'*64,'trial':old,'binding':'3'*64,'reserved_microusd':50000}))
 def test_same_suite_second_or_historical_trial_cannot_use_session(self):
  rows=self.grant()
  for key,admission in [('9'*64,'e'*64),('d'*64,'9'*64)]:
   with self.assertRaises(ValueError):self.l.fold(self.append(rows,{'kind':'trial','key':key,'admission':admission,'evaluation_suite_sha256':'b'*64}))
  _,trials,_,_=self.l.fold(self.rows)
  old=next(iter(trials));g={**self.g,'session_trial_sha256':old,'session_admission_sha256':trials[old]['admission']}
  with self.assertRaises(ValueError):self.l.fold(self.grant(g))
 def test_over_reservation_charge_stops(self):
  rows=self.grant();rows=self.append(rows,{'kind':'trial','key':'d'*64,'admission':'e'*64,'evaluation_suite_sha256':'b'*64})
  rows=self.append(rows,{'kind':'call','key':'f'*64,'trial':'d'*64,'binding':'1'*64,'reserved_microusd':50000})
  rows=self.append(rows,{'kind':'bill','key':'f'*64,'microusd':50001})
  self.assertTrue(self.l.fold(rows)[0]['stopped'])
class DeadlineAndCorrections(unittest.TestCase):
 def test_browser_maximum_workload_deadline_and_legacy_headroom(self):
  launcher=m.load_module('launcher','engine-native-cycle.py')
  launcher.require_headroom({'version':'browser-project-cycle-adapter@v3','max_calls':200,'expires_at':7300},{},lambda x:x,now=100)
  with self.assertRaises(ValueError):launcher.require_headroom({'version':'browser-project-cycle-adapter@v3','max_calls':200,'expires_at':579},{},lambda x:x,now=100)
  with self.assertRaises(ValueError):launcher.require_headroom({'max_calls':200,'expires_at':7300},{},lambda x:x,now=100)
 def test_corrections_are_shared_finite_and_browser_only(self):
  import tempfile
  correction=m.load_module('correction','engine-bounded-correction.py')
  with tempfile.TemporaryDirectory() as root:
   with self.assertRaises(ValueError):correction.Budget(Path(root)/'old',{},20)
   b=correction.Budget(Path(root)/'new',{'version':'browser-project-cycle-adapter@v3'},20)
   result={'receipt':{'status':'settled','usd':'0.001','parse_location':'content','error_code':'malformed_json'}}
   for i in range(20):b.feedback('proposal','source',result,str(i))
   with self.assertRaises(ValueError):b.feedback('proposal','source',result,'21')
   self.assertEqual(len(b.events()),20)
if __name__=='__main__':unittest.main()
