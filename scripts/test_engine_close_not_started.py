"""Authored journal closure, no native/worker/provider effects."""
import contextlib,importlib.util,json,tempfile,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('close',Path(__file__).with_name('engine-close-not-started.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Close(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.p=Path(self.tmp.name);(self.p/'adapter').mkdir();(self.p/'cycle').mkdir();self.d=m.load('fixture_driver','engine-b91-driver.py');self.e=self.d.e
  self.ad={'directory':str(self.p/'adapter'),'independent_assessment':{'sha256':'b'*64}};self.ca={'journal':str(self.p/'cycle')};self.execution={'state':{'goal_completion':{'snapshot':'c'*64}},'receipts':['authored unavailable']}
  self.name='independent_assessment_'+self.e.sha({'snapshot':'c'*64,'source':{},'script':'b'*64});self.ev={'binding':{'admission_sha256':self.e.sha(self.ad),'key':'a'*64},'source':{}}
  self.disp={'kind':'execution_not_started','key':'a'*64,'evidence_sha256':m.ledger_digest(self.ev)}
  self.write('adapter',self.ad,[('execution_authored',self.execution)],self.name);self.write('cycle',self.ca,[],'resume_0')
  self.ev.update(records={'adapter':str(self.p/'adapter/adapter.jsonl'),'cycle':str(self.p/'cycle/cycle.jsonl')},artifacts={str(f):__import__('hashlib').sha256(f.read_bytes()).hexdigest() for f in (self.p/'adapter/adapter.jsonl',self.p/'cycle/cycle.jsonl')});self.disp['evidence_sha256']=m.ledger_digest(self.ev)
  for name,v in [('adapter-admission',self.ad),('cycle-admission',self.ca)]:(self.p/(name+'.json')).write_text(json.dumps(v))
  @contextlib.contextmanager
  def locked():yield None,[{'event':self.disp}]
  self.ledger=type('L',(),{'locked':staticmethod(locked)})();self.oldload=m.load
  from unittest.mock import patch
  self.patch=patch.object(m,'load',side_effect=lambda name,file:type('P',(),{'validate_evidence':staticmethod(lambda v:None)}) if file=='engine-execution-not-started.py' else self.oldload(name,file));self.patch.start();self.addCleanup(self.patch.stop)
 def write(self,name,ad,done,pending):
  rows=[self.d.Driver.row([],{'kind':'admission','value':ad,'fixture_only':False})]
  for key,value in done:
   i={'kind':'intent','name':key};rows.append(self.d.Driver.row(rows,i));rows.append(self.d.Driver.row(rows,{'kind':'result','name':key,'intent':self.e.sha(i),'value':value}))
  rows.append(self.d.Driver.row(rows,{'kind':'intent','name':pending}));(self.p/name/(name+'.jsonl')).write_text('\n'.join(json.dumps(x) for x in rows)+'\n')
 def test_close_replay_preserves_prefix_no_success_claim(self):
  f=self.p/'adapter/adapter.jsonl';before=f.read_bytes();result=m.close(self.p,self.ledger,self.ev);self.assertEqual(result['status'],'execution_not_started');self.assertFalse(result['completed_feedback_cycle']);self.assertTrue(f.read_bytes().startswith(before));after=f.read_bytes();self.assertEqual(m.close(self.p,self.ledger,self.ev),result);self.assertEqual(f.read_bytes(),after)
 def test_interrupted_closure_each_append_recovers(self):
  from unittest.mock import patch
  initial_a=(self.p/'adapter/adapter.jsonl').read_bytes();initial_c=(self.p/'cycle/cycle.jsonl').read_bytes()
  for stop_after in (1,2,3):
   with self.subTest(stop_after=stop_after):
    (self.p/'adapter/adapter.jsonl').write_bytes(initial_a);(self.p/'cycle/cycle.jsonl').write_bytes(initial_c);original=self.d.Driver.append;count=[0]
    def fail(holder,event):
     original(holder,event);count[0]+=1
     if count[0]==stop_after:raise RuntimeError('authored close interruption')
    # Each close loads the driver anew; intercept module loading with same driver.
    prior=m.load
    def loader(name,file):return self.d if file=='engine-b91-driver.py' else prior(name,file)
    with patch.object(m,'load',side_effect=loader),patch.object(self.d.Driver,'append',fail):
     with self.assertRaises(RuntimeError):m.close(self.p,self.ledger,self.ev)
    self.assertEqual(m.close(self.p,self.ledger,self.ev)['status'],'execution_not_started')
 def test_missing_disposition_blocks(self):
  self.disp['key']='d'*64
  with self.assertRaisesRegex(ValueError,'missing exact'):m.close(self.p,self.ledger,self.ev)
 def test_unrelated_pending_blocks(self):
  self.write('adapter',self.ad,[('execution_authored',self.execution)],'other_mutation')
  with self.assertRaisesRegex(ValueError,'reviewed journal prefix'):m.close(self.p,self.ledger,self.ev)
if __name__=='__main__':unittest.main()
