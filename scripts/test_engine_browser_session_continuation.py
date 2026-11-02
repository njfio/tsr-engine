"""Frozen historical ledger read; prospective extension events are in memory only."""
import importlib.util,json,os,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('m',Path(__file__).with_name('engine-b91-campaign.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
@unittest.skipUnless(os.getenv('TSR_BROWSER_CONTINUATION_LEDGER'),'frozen settled baseline')
class Grant(unittest.TestCase):
 def setUp(self):
  self.rows=[json.loads(x) for x in Path(os.environ['TSR_BROWSER_CONTINUATION_LEDGER']).read_text().splitlines()];self.l=object.__new__(m.Ledger);self.l.manifest=self.rows[0]['event']['value'];self.l.hash=m.digest(self.l.manifest)
  self.g={'version':'native-browser-session@v2','session_trial_sha256':'d'*64,'session_admission_sha256':'e'*64,'manifest_sha256':self.l.hash,'ledger_prefix_head':self.rows[-1]['hash'],'previous_max_calls':462,'max_calls':464,'previous_max_execution_groups':172,'max_execution_groups':174,'cap_microusd':10228567,'reservation_microusd':50000,'authority_sha256':'a'*64,'reviewer':'Authored only','suite_manifest_sha256':'b'*64,'source_class':'newly-authored-nonprivate-browser-project'}
 def append(self,rows,event):return rows+[m.Ledger.row(rows,event)]
 def granted(self,g=None):
  g=g or self.g;return self.append(self.rows,{'kind':'evaluation_grant','key':m.digest(g),'value':g})
 def test_retains_baseline_and_replaces_unused_allowance(self):
  st=self.l.fold(self.granted())[0];self.assertEqual((st['calls'],st['execution_groups'],st['known_cost_microusd']),(264,74,228567));self.assertEqual((st['remaining_calls'],st['remaining_execution_groups'],st['effective_cap_microusd']-st['committed_microusd']),(200,100,10000000))
 def test_old_trial_and_duplicate_grant_cannot_spend(self):
  rows=self.granted();old=[r['event']['key'] for r in self.rows if r['event']['kind']=='trial'][-1]
  with self.assertRaises(ValueError):self.l.fold(self.append(rows,{'kind':'call','key':'8'*64,'trial':old,'binding':'9'*64,'reserved_microusd':50000}))
  g={**self.g,'ledger_prefix_head':rows[-1]['hash']}
  with self.assertRaises(ValueError):self.l.fold(self.append(rows,{'kind':'evaluation_grant','key':m.digest(g),'value':g}))
 def test_wrong_source_cap_and_unknown_stop(self):
  for change in ({'cap_microusd':10228568},{'max_calls':465},{'ledger_prefix_head':'f'*64}):
   with self.assertRaises(ValueError):self.l.fold(self.granted({**self.g,**change}))
  rows=self.granted();rows=self.append(rows,{'kind':'trial','key':'d'*64,'admission':'e'*64,'evaluation_suite_sha256':'b'*64});rows=self.append(rows,{'kind':'call','key':'f'*64,'trial':'d'*64,'binding':'1'*64,'reserved_microusd':50000});self.assertTrue(self.l.fold(rows)[0]['stopped'])
if __name__=='__main__':unittest.main()
