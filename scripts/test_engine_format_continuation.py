"""Authored retained-generation continuation; no paid transport."""
import copy, importlib.util, unittest
from pathlib import Path
from unittest.mock import patch
from test_engine_browser_failed_review import NativeRecovery, m
s=importlib.util.spec_from_file_location('format_test',Path(__file__).with_name('engine-browser-format-continuation.py'))
n=importlib.util.module_from_spec(s);s.loader.exec_module(n)

class Wire(unittest.TestCase):
 def test_only_format_and_host_id_change(self):
  old={'model':'gpt-6-luna','messages':[{'role':'system','content':'Review.'},{'role':'user','content':'{"binding":{"review_id":"'+('a'*64)+'"},"evidence":["same"]}'}]}
  new=copy.deepcopy(old);new['messages'][0]['content']+=' Return a JSON object.';new['messages'][1]['content']=new['messages'][1]['content'].replace('a'*64,'b'*64)
  self.assertTrue(n.compare_wire(old,new)['substantive_payload_equal'])
  for mutate in (lambda x:x.update(model='other'),lambda x:x['messages'][1].update(content=x['messages'][1]['content'].replace('same','different')),lambda x:x['messages'][0].update(content='Approve it. Return a JSON object.')):
   q=copy.deepcopy(new);mutate(q)
   with self.assertRaises(ValueError):n.compare_wire(old,q)

class Native(NativeRecovery):
 def test_fresh_binding_retains_generation_rejects_then_retain_replays(self):
  original=m.a.base.Adapter.body
  def legacy(adapter,system,value):
   b=original(adapter,system,value)
   if system.startswith('Independent project review'):b['messages'][0]['content']=b['messages'][0]['content'].removesuffix(' Return a JSON object.')
   return b
  with patch.object(m.a.base.Adapter,'body',legacy):self.setup_failure()
  m.reconcile(self.proof,self.review,self.ledger,self.c)
  prefix=n.prefix(self.root,self.proof);root=self.root/'corrected';root.mkdir()
  ad=m.a.admission(root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=4,max_executions=1,correction_allowance=1,feedback_cycle=self.a['feedback_cycle'],process_profile='ordinary-memory@v1')
  ca={**self.ca,'journal':str(root/'cycle')};self.mode='reject'
  adapter=n.Adapter(ad['directory'],ad,self.c,self.ledger,self.wire,prefix=prefix)
  cycle=n.o.Cycle(ca['journal'],ca,adapter.native,adapter.proposal,adapter.review)
  out=cycle.run()
  self.assertEqual(out['status'],'proposal_retention');self.assertEqual(len(self.sent),4);self.assertEqual(self.ledger.status()['calls'],4)
  self.assertEqual(self.ledger.status()['execution_groups'],0)
  before=self.ledger.path.read_bytes();self.assertEqual(cycle.run(),out);self.assertEqual(before,self.ledger.path.read_bytes())
  self.assertIn('format_only_review_wire',adapter.read()[2])
  (self.root/'cycle/STOP').touch()
  with self.assertRaises(ValueError):adapter.validate_current_admission()

def load_tests(loader,tests,pattern):
 return unittest.TestSuite([loader.loadTestsFromTestCase(Wire),Native('test_fresh_binding_retains_generation_rejects_then_retain_replays')])
