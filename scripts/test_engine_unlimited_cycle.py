"""Authored unlimited-policy control flow; no provider or external effects."""
import importlib.util,unittest,copy
from pathlib import Path
import test_engine_ongoing_cycle as fixture
m=fixture.m
class UnlimitedCycle(fixture.OngoingCycle):
 def setUp(self):
  super().setUp();self.a.update(unlimited_policy_sha256='a'*64,max_decisions=None,max_executions=None,expires_at=None);self.a['feedback_cycle']['max_continuations']=None
 def test_missing_or_mixed_authority_rejected(self):
  for change in ({'unlimited_policy_sha256':'bad'},{'max_decisions':200},{'expires_at':'2999-01-01T00:00:00Z'}):
   old=copy.deepcopy(self.a);self.a.update(change)
   with self.assertRaises(ValueError):self.runner()
   self.a=old
 def test_stop_before_dispatch(self):
  runner=self.runner();(self.root/'cycle/STOP').touch()
  with self.assertRaises(Exception):runner.run()
  self.assertFalse(self.world.load()['calls'])
 def test_continuation_budget_does_not_renew_itself(self):
  # Bounded policy remains distinct; unlimited cannot smuggle a hidden continuation cap.
  self.a['feedback_cycle']['max_continuations']=1
  with self.assertRaises(ValueError):self.runner()
 def test_review_rejection_allowance_needs_unlimited_authority(self):
  self.a.update(local_source_profile='browser-scoped-source@v1',max_review_rejections=12)
  try:self.runner()
  except ValueError as error:self.assertNotIn('rejection',str(error))
  for bad in (13,True,-1):
   self.a['max_review_rejections']=bad
   with self.assertRaisesRegex(ValueError,'rejection'):self.runner()
  self.a.update(max_review_rejections=4,unlimited_policy_sha256=None,max_decisions=8,max_executions=2,expires_at='2999-01-01T00:00:00Z');self.a['feedback_cycle']['max_continuations']=1
  with self.assertRaisesRegex(ValueError,'rejection'):self.runner()
 def test_adapter_admission_rejection_allowance(self):
  spec=importlib.util.spec_from_file_location('unlimited_browser_adapter',Path(__file__).with_name('engine-browser-adapter.py'));adapter=importlib.util.module_from_spec(spec);spec.loader.exec_module(adapter)
  for kw in ({'max_review_rejections':4},{'max_review_rejections':13,'unlimited_policy_sha256':'a'*64},{'max_review_rejections':True,'unlimited_policy_sha256':'a'*64}):
   with self.assertRaisesRegex(ValueError,'review rejection allowance'):adapter.admission(None,None,None,None,None,**kw)
  a=object.__new__(adapter.Adapter);a.admission={'max_review_rejections':12,'origin':{}}
  a.read=lambda:(None,None,{'review_rejection_'+str(i):{} for i in range(11)});a.require_rejection_room()
  a.read=lambda:(None,None,{'review_rejection_'+str(i):{} for i in range(12)})
  with self.assertRaises(Exception):a.require_rejection_room()
 def test_admission_revalidation_keeps_the_admitted_rejection_allowance(self):
  spec=importlib.util.spec_from_file_location('revalidate_browser_adapter',Path(__file__).with_name('engine-browser-adapter.py'));adapter=importlib.util.module_from_spec(spec);spec.loader.exec_module(adapter)
  seen={}
  class Controller:
   directory='/authored'
   def cli(self,*_):return {}
  a=object.__new__(adapter.Adapter);a.root='/authored';a.controller=Controller();a.ledger=type('L',(),{'manifest':{}})()
  a.admission={'origin':{'directory':'/authored','public':{},'initial_sources':{}},'expires_at':None,'max_decisions':None,'max_executions':None,'correction_allowance':0,'feedback_events':[],'max_review_rejections':12}
  def rebuilt(*_,**kw):
   seen.update(kw);return {}
  adapter.admission=rebuilt
  with self.assertRaisesRegex(ValueError,'drift'):a.validate_current_admission()
  self.assertEqual(seen['max_review_rejections'],12)
if __name__=='__main__':unittest.main()
