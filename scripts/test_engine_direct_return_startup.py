"""Authored startup orchestration, no provider calls or external execution."""
import importlib.util,tempfile,unittest
from pathlib import Path
from types import SimpleNamespace
s=importlib.util.spec_from_file_location('direct_startup',Path(__file__).with_name('engine-native-cycle.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Startup(unittest.TestCase):
 def test_fresh_matched_arms_and_inert_reentry(self):
  for ordinary in (False,True):
   with tempfile.TemporaryDirectory() as d:
    events=[];settled=[]
    def complete():
     if not settled:events.append('assessment');settled.append(True)
    a=SimpleNamespace(admission={'criterion_return_profile':{},'process_profile':'ordinary-memory@v1' if ordinary else None},root=Path(d),complete_pending_return=complete)
    a.admission['criterion_return_profile']={'version':'authored'}
    c=SimpleNamespace(read=lambda:([{}],{},{}));ca={'journal':d}
    m.complete_fresh_direct_return(a,c,ca);events.append('first proposal')
    m.complete_fresh_direct_return(a,c,ca)
    self.assertEqual(events,['assessment','first proposal'])
 def test_existing_pending_and_terminal_owned_by_cycle(self):
  a=SimpleNamespace(admission={'criterion_return_profile':{'version':'authored'}},complete_pending_return=lambda:self.fail('replayed'))
  for state in (([{},{}],{},{}),([{}],{'pending':1},{}),([{}],{},{'terminal':{'status':'proposal_retention'}})):
   m.complete_fresh_direct_return(a,SimpleNamespace(read=lambda:state),{})
 def test_stop_and_interruption_leave_no_fake_completion(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);a=SimpleNamespace(admission={'criterion_return_profile':{'version':'authored'}},root=root,complete_pending_return=lambda:(_ for _ in ()).throw(RuntimeError('retained interruption')));c=SimpleNamespace(read=lambda:([{}],{},{}));ca={'journal':d}
   with self.assertRaisesRegex(RuntimeError,'retained'):m.complete_fresh_direct_return(a,c,ca)
   (root/'checkpoint-0').mkdir();(root/'checkpoint-0/STOP').touch()
   with self.assertRaisesRegex(ValueError,'stopped'):m.complete_fresh_direct_return(a,c,ca)
 def test_no_profile_and_feedback_path_unchanged(self):
  for ad in ({},{'criterion_return_profile':{'version':'authored'},'feedback_events':[{}]}):
   m.complete_fresh_direct_return(SimpleNamespace(admission=ad),None,None)
if __name__=='__main__':unittest.main()
