"""Unlimited-policy fixtures: no provider, target execution, or real ledger writes."""
import copy,json,unittest
from test_engine_b91_campaign import Campaign,m

class Unlimited(Campaign):
 def policy(self,keys=None):
  rows=[json.loads(x) for x in self.path.read_text().splitlines()]
  return {'version':'browser-unlimited-policy@v1','manifest_sha256':m.digest(self.manifest),'ledger_prefix_head':rows[-1]['hash'],'provider':'CheaperInference','model':m.MODEL,'source_class':'newly-authored-nonprivate-browser-project','authority':copy.deepcopy(m.UNLIMITED_AUTHORITY),'quarantined_unknown_keys':keys or []}
 def grant(self):
  status=self.ledger.status();rows=[json.loads(x) for x in self.path.read_text().splitlines()]
  value={'version':m.UNLIMITED_GRANT,'manifest_sha256':m.digest(self.manifest),'ledger_prefix_head':rows[-1]['hash'],'reservation_microusd':m.RESERVE,'authority_sha256':'a'*64,'reviewer':'independent authored fixture','suite_manifest_sha256':'d'*64,'source_class':'newly-authored-nonprivate-browser-project','session_trial_sha256':'e'*64,'session_admission_sha256':'f'*64,'baseline':{'calls':status['calls'],'execution_groups':status['execution_groups'],'settled_microusd':status['known_cost_microusd']},'unlimited_policy_sha256':status['unlimited_policy_sha256']}
  value.update({k:None for k in ('previous_max_calls','max_calls','previous_max_execution_groups','max_execution_groups','cap_microusd','session_max_calls','session_max_execution_groups','session_cap_microusd','expires_at')})
  return value
 def test_explicit_policy_preserves_counts_fresh_scope_restart(self):
  self.call('before');before=self.path.read_bytes();policy=self.policy();self.ledger.enable_unlimited(policy)
  self.assertTrue(self.path.read_bytes().startswith(before));s=self.ledger.status();self.assertEqual(s['calls'],1);self.assertEqual(s['known_cost_microusd'],500);self.assertIsNone(s['remaining_calls']);self.assertIsNone(s['effective_cap_microusd'])
  with self.assertRaises(ValueError):self.call('old-scope')
  grant=self.grant();self.ledger.grant_calls(grant,m.digest(grant));self.ledger.enroll('e'*64,'f'*64,suite_sha256='d'*64)
  self.ledger.reserve('call','e'*64,'new','a'*64);self.ledger.settle('bill',m.digest({'trial':'e'*64,'request':'new','kind':'call'}),500)
  again=m.Ledger(self.path,self.manifest,m.digest(self.manifest));self.assertEqual(again.status()['calls'],2);self.assertFalse(again.status()['stopped'])
  again.enable_unlimited(policy);self.assertEqual(again.status()['calls'],2)
 def test_null_totals_cross_old_caps_with_complete_receipts(self):
  self.ledger.enable_unlimited(self.policy());grant=self.grant();self.ledger.grant_calls(grant,m.digest(grant));self.ledger.enroll('e'*64,'f'*64,suite_sha256='d'*64)
  rows=[json.loads(x) for x in self.path.read_text().splitlines()]
  def event(value):rows.append(self.ledger.row(rows,value))
  for i in range(101):
   key=m.digest({'trial':'e'*64,'request':str(i),'kind':'call'})
   event({'kind':'call','key':key,'trial':'e'*64,'binding':'a'*64,'reserved_microusd':m.RESERVE});event({'kind':'bill','key':key,'microusd':m.RESERVE})
  for i in range(41):
   key=m.digest({'trial':'e'*64,'request':str(i),'kind':'execution'})
   event({'kind':'execution','key':key,'trial':'e'*64,'binding':'a'*64});event({'kind':'return','key':key,'outcome':'returned'})
  status=self.ledger.fold(rows)[0]
  self.assertEqual((status['calls'],status['execution_groups'],status['known_cost_microusd']),(101,41,5050000));self.assertFalse(status['stopped'])
 def test_scope_authority_prefix_and_finite_sentinel_rejected(self):
  policy=self.policy()
  for key,value in [('provider','other'),('model','premium'),('ledger_prefix_head','f'*64),('authority',{})]:
   bad=copy.deepcopy(policy);bad[key]=value
   with self.assertRaises(ValueError):self.ledger.enable_unlimited(bad)
  self.ledger.enable_unlimited(policy);grant=self.grant();grant['session_max_calls']=2**63
  with self.assertRaises(ValueError):self.ledger.grant_calls(grant,m.digest(grant))
 def test_unquarantined_unknown_and_execution_not_unblocked(self):
  self.call('unknown',None)
  with self.assertRaises(ValueError):self.ledger.enable_unlimited(self.policy(self.ledger.status()['unknown_calls']))
  self.assertTrue(self.ledger.status()['stopped'])

from test_engine_generation_quarantine import Quarantine
class UnlimitedQuarantine(Quarantine):
 def test_disposed_unknown_retained_and_new_unknown_blocks(self):
  self.quarantine()
  rows=[json.loads(x) for x in self.ledger.path.read_text().splitlines()]
  policy={'version':'browser-unlimited-policy@v1','manifest_sha256':m.digest(self.man),'ledger_prefix_head':rows[-1]['hash'],'provider':'CheaperInference','model':m.MODEL,'source_class':'newly-authored-nonprivate-browser-project','authority':copy.deepcopy(m.UNLIMITED_AUTHORITY),'quarantined_unknown_keys':[self.key]}
  self.ledger.enable_unlimited(policy)
  status=self.ledger.status();self.assertEqual(status['unknown_calls'],[self.key]);self.assertEqual(status['committed_microusd'],50000);self.assertFalse(status['stopped'])
  # Reuse only authored fixture grant builder with this independent ledger.
  helper=object.__new__(Unlimited);helper.path=self.ledger.path;helper.manifest=self.man;helper.ledger=self.ledger
  grant=helper.grant();self.ledger.grant_calls(grant,m.digest(grant));self.ledger.enroll('e'*64,'f'*64,suite_sha256='d'*64)
  with self.assertRaises(ValueError):self.ledger.reserve('call',self.trial,'old-output','a'*64)
  self.ledger.reserve('call','e'*64,'fresh','a'*64)
  self.assertTrue(self.ledger.status()['stopped']);self.assertEqual(self.ledger.status()['blocking_unknown_calls'],[m.digest({'trial':'e'*64,'request':'fresh','kind':'call'})])
  with self.assertRaises(ValueError):self.ledger.reserve('call','e'*64,'repeat','a'*64)
 def test_unknown_mutation_still_blocks_policy(self):
  self.quarantine()
  with self.ledger.locked() as (f,rows):
   self.ledger.append_locked(f,rows,{'kind':'trial','key':'e'*64,'admission':'f'*64})
  with self.ledger.locked() as (f,rows):
   self.ledger.append_locked(f,rows,{'kind':'execution','key':'1'*64,'trial':'e'*64,'binding':'2'*64})
  rows=[json.loads(x) for x in self.ledger.path.read_text().splitlines()]
  policy={'version':'browser-unlimited-policy@v1','manifest_sha256':m.digest(self.man),'ledger_prefix_head':rows[-1]['hash'],'provider':'CheaperInference','model':m.MODEL,'source_class':'newly-authored-nonprivate-browser-project','authority':copy.deepcopy(m.UNLIMITED_AUTHORITY),'quarantined_unknown_keys':[self.key]}
  with self.assertRaises(ValueError):self.ledger.enable_unlimited(policy)

if __name__=='__main__':unittest.main()
