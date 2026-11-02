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
if __name__=='__main__':unittest.main()
