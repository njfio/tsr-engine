"""Authored accounting-only proofs; helper substituted, no execution or live ledger."""
import copy,json,tempfile,unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
import importlib.util
spec=importlib.util.spec_from_file_location('disposition_campaign',Path(__file__).with_name('engine-b91-campaign.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Disposition(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.path=Path(self.tmp.name)/'ledger';self.man=m.manifest('a'*64,self.path,fixture=True);self.ledger=m.Ledger(self.path,self.man,m.digest(self.man),create=True)
  self.trial='b'*64;self.ad='c'*64;self.request='authored';self.ledger.enroll(self.trial,self.ad);self.key=self.ledger.reserve('execution',self.trial,self.request,self.ad);self.ledger.settle('return',self.key,'unknown')
  row=json.loads(self.path.read_text().splitlines()[-1]);self.binding={'key':self.key,'trial':self.trial,'admission_sha256':self.ad,'request_id':self.request,'original_return_row':row['hash']}
  self.evidence={'fixture_only':True,'binding':self.binding,'proven_not_started':True}
  self.review={'version':'independent-execution-disposition-review@v1','decision':'confirmed_not_started','evidence_sha256':m.digest(self.evidence),'reviewer':'authored separate reviewer','report_sha256':'d'*64}
  def validate(value):
   if not value.get('fixture_only') or value.get('proven_not_started') is not True:raise ValueError('ambiguous evidence')
   return value['binding']
  self.helper=SimpleNamespace(validate_evidence=validate,validate_current_evidence=validate)
  self.patch=patch.object(m,'load_module',return_value=self.helper);self.patch.start();self.addCleanup(self.patch.stop)
 def apply(self):return self.ledger.reconcile_execution_not_started(self.key,self.evidence,self.review)
 def test_exact_append_retains_unknown_and_consumed_count_restart(self):
  old=self.path.read_bytes();status=self.apply();self.assertTrue(self.path.read_bytes().startswith(old));self.assertEqual(status['execution_groups'],1);self.assertEqual(status['execution_not_started'],[self.key]);self.assertEqual(status['historical_unknown_executions'],[self.key]);self.assertEqual(status['unknown_executions'],[]);self.assertFalse(status['stopped'])
  reloaded=m.Ledger(self.path,self.man,m.digest(self.man));self.assertEqual(reloaded.status(),status)
  with self.assertRaises(ValueError):self.apply()
  with self.assertRaises(ValueError):reloaded.reserve('execution',self.trial,self.request,self.ad)
  self.assertEqual(reloaded.status()['execution_groups'],1)
 def test_other_unknown_remains_blocking(self):
  with self.ledger.locked() as (f,rows):self.ledger.append_locked(f,rows,{'kind':'execution','key':'e'*64,'trial':self.trial,'binding':self.ad})
  status=self.apply();self.assertTrue(status['stopped']);self.assertEqual(status['unknown_executions'],['e'*64])
 def test_cross_key_source_old_return_and_review_rejected(self):
  before=self.path.read_bytes()
  for name,value in [('key','f'*64),('trial','f'*64),('admission_sha256','f'*64),('request_id','other'),('original_return_row','f'*64)]:
   bad=copy.deepcopy(self.evidence);bad['binding'][name]=value
   review={**self.review,'evidence_sha256':m.digest(bad)}
   with self.assertRaises(ValueError):self.ledger.reconcile_execution_not_started(self.key,bad,review)
  for name,value in [('decision','returned'),('evidence_sha256','f'*64),('reviewer',''),('report_sha256','bad')]:
   with self.assertRaises(ValueError):self.ledger.reconcile_execution_not_started(self.key,self.evidence,{**self.review,name:value})
  self.assertEqual(self.path.read_bytes(),before)
 def test_ambiguous_and_current_proof_failure_do_not_append(self):
  before=self.path.read_bytes();bad={**self.evidence,'proven_not_started':False}
  with self.assertRaises(ValueError):self.ledger.reconcile_execution_not_started(self.key,bad,{**self.review,'evidence_sha256':m.digest(bad)})
  self.helper.validate_current_evidence=lambda *_:(_ for _ in ()).throw(ValueError('changed current source'))
  with self.assertRaises(ValueError):self.apply()
  self.assertEqual(self.path.read_bytes(),before)
 def test_returned_execution_is_never_reclassified(self):
  rows=[json.loads(x) for x in self.path.read_text().splitlines()];event={**rows[-1]['event'],'outcome':'returned'};rows[-1]=self.ledger.row(rows[:-1],event);self.path.write_text('\n'.join(m.canonical(x).decode() for x in rows)+'\n')
  self.evidence['binding']['original_return_row']=rows[-1]['hash'];self.review['evidence_sha256']=m.digest(self.evidence)
  with self.assertRaises(ValueError):self.apply()
if __name__=='__main__':unittest.main()
