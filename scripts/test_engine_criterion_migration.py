"""Authored offline migration faults; no provider, browser, or player evidence."""
import copy,importlib.util,tempfile,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('criterion_migration_test',Path(__file__).with_name('engine-criterion-migration.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)

class Migration(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
  self.j=object.__new__(m.a.d.Driver);self.j.root=Path(self.tmp.name);self.j.file=self.j.root/'migration.jsonl';self.j.admission={'authored':True};self.j.fixture=False
  m.a.e.private_save(self.j.file,self.j.row([],{'kind':'admission','value':self.j.admission,'fixture_only':False}))
  self.value={'state_root':'root','binding':{'source':'source'},'frame':{'snapshot':'source'},'findings':{'a':{'finding':'unknown'}}}
  self.context={**{k:copy.deepcopy(self.value[k]) for k in ('state_root','binding','frame')},'current':None};self.calls=0
 def admit(self,value):
  self.assertEqual(value,self.value);self.calls+=1;self.context['current']={'id':'receipt','assessment':copy.deepcopy(value)};return 'receipt'
 def run_step(self,hook=lambda *_:None,guard=lambda:None):return m.advance(self.j,self.value,lambda:copy.deepcopy(self.context),self.admit,guard,hook)
 def test_exact_admission_and_duplicate_restart(self):
  self.assertEqual(self.run_step(),'receipt');before=self.j.file.read_bytes();self.assertEqual(self.run_step(),'receipt');self.assertEqual(self.calls,1);self.assertEqual(before,self.j.file.read_bytes())
 def test_after_commit_interruption_uses_receipt_without_redispatch(self):
  def hook(phase):
   if phase=='after_effect':raise RuntimeError('authored crash')
  with self.assertRaises(RuntimeError):self.run_step(hook)
  self.assertEqual(self.run_step(),'receipt');self.assertEqual(self.calls,1);self.assertFalse(self.j.read()[1])
 def test_unproven_pre_effect_interruption_stays_blocked(self):
  def hook(phase):raise RuntimeError('authored interruption')
  with self.assertRaises(RuntimeError):self.run_step(hook)
  before=self.j.file.read_bytes()
  with self.assertRaisesRegex(ValueError,'not proven committed'):self.run_step()
  self.assertEqual(self.calls,0);self.assertEqual(before,self.j.file.read_bytes())
 def test_stop_and_unknown_external_effect_guard(self):
  for reason in ('Stop','unknown tool effect','unknown billing'):
   def guard():raise ValueError(reason)
   before=self.j.file.read_bytes()
   with self.assertRaisesRegex(ValueError,reason):self.run_step(guard=guard)
   self.assertEqual(before,self.j.file.read_bytes());self.assertEqual(self.calls,0)
 def test_stale_or_substituted_context(self):
  for key in ('state_root','binding','frame'):
   bad=copy.deepcopy(self.context);bad[key]='forged'
   with self.assertRaisesRegex(ValueError,'stale'):m.check_context(bad,self.value)
  bad=copy.deepcopy(self.context);bad['current']={'id':'foreign','assessment':{}}
  with self.assertRaisesRegex(ValueError,'different'):m.check_context(bad,self.value)
 def test_native_refusal_stays_pending(self):
  def refuse(value):raise ValueError('native refusal')
  with self.assertRaisesRegex(ValueError,'native refusal'):m.advance(self.j,self.value,lambda:self.context,refuse,lambda:None)
  with self.assertRaisesRegex(ValueError,'not proven committed'):self.run_step()
  self.assertEqual(self.calls,0)
 def test_corrupt_journal_refused(self):
  self.j.file.write_text(self.j.file.read_text().replace('authored','modified'))
  with self.assertRaisesRegex(ValueError,'history drift'):self.run_step()
 def test_python_contract_preserves_unknown_observed_basis(self):
  shape=m.a.load('migration_test_shape','engine-project-output.py');binding={'review_id':'a'*64,'evidence_ids':['source'],'observed_receipts':['source']}
  for basis in ('observed_supplemental','observed_supplemental_set'):
   schema=shape.review_schema(binding,{'a':'Authored criterion'})
   schema['properties']['findings']['properties']['a']['properties']['basis']['enum']=['unknown',basis]
   value={'review_id':'a'*64,'verdict':'unknown','findings':{'a':{'finding':'unknown','basis':basis,'evidence':['source'],'explanation':'Unavailable authenticated observations','limitations':'No empirical success or failure'}}};original=copy.deepcopy(value)
   self.assertEqual(m.h.validate_assessment(value,binding,shape,schema),original)
   frame={'coverage':{'a':['source']},'prerequisites':{'a':{'kind':basis,'admission':'b'*64,'receipt':'c'*64,'observations':[{'admission':'b'*64,'receipt':'c'*64}]}},'observed_tests':{}}
   result=m.h.native_findings(value,frame)['a'];self.assertEqual(result['finding'],'unknown');self.assertEqual(result['basis']['kind'],basis);self.assertEqual(value,original)
   bad=copy.deepcopy(value);bad['verdict']='accept'
   with self.assertRaises(m.h.CriterionConsistencyFailure):m.h.validate_assessment(bad,binding,shape,schema)

if __name__=='__main__':unittest.main()
