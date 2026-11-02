"""AUTHORED single-user decision fixtures; no real approval or provider effects."""
import tempfile,unittest,copy,time
from pathlib import Path
import server as ui
from decisions import Decisions
class Store:
 def __init__(self,root):self.root=root;self.events_list=[{'id':'message','type':'input','text':'AUTHORED input'}];self.sent=[]
 def events(self,p):return self.events_list
 def send(self,p,b):
  if not any(v[1]['id']==b['id'] for v in self.sent):self.sent.append((p,b))
  return {'state':'preparing','message':'AUTHORED dispatch'}
class Fixture(Decisions):
 def __init__(self,s):super().__init__(s);self.revision=0;self.effects=[]
 def disclosure(self,p):return {'project':p,'revision':self.revision,'remaining_microusd':123,'fixture_only':True,'resume_input_id':'message','input_ids':['message']}
 def apply(self,d):
  if ui.sha(d) not in self.effects:self.effects.append(ui.sha(d))
class DecisionTests(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.s=Store(Path(self.tmp.name));self.d=Fixture(self.s)
 def body(self,choice='approve'):
  q=self.d.present('fixture',{});return {'choice':choice,'challenge':q['challenge'],'digest':q['digest']}
 def test_present_does_not_approve_and_decline_preserves_input(self):
  b=self.body('decline');self.assertEqual(self.d.rows(),[]);self.d.decide('fixture',b);self.assertFalse(self.s.sent);self.assertEqual(len(self.s.events_list),1);self.assertEqual(self.d.decide('fixture',b)['state'],'recorded')
 def test_approve_exact_once_and_restart_history(self):
  b=self.body();self.d.decide('fixture',b);self.d.decide('fixture',b);self.assertEqual(len(self.s.sent),1);self.assertEqual(len(self.d.effects),1)
  reboot=Fixture(self.s);self.assertEqual(reboot.decide('fixture',b)['state'],'recorded');self.assertEqual(reboot.approval('fixture')['choice'],'approve')
 def test_restart_uses_only_approved_input(self):
  b=self.body();self.d.decide('fixture',b);self.s.events_list.append({'id':'later','type':'input','text':'AUTHORED later draft'})
  self.d.resume_recorded();self.assertEqual([b['id'] for _,b in self.s.sent],['message'])
 def test_stale_cross_project_missing_and_conflicting_replay(self):
  b=self.body();self.d.revision=1
  with self.assertRaisesRegex(ValueError,'changed'):self.d.decide('fixture',b)
  b=self.body()
  with self.assertRaises(ValueError):self.d.decide('other',b)
  self.d.decide('fixture',b)
  with self.assertRaises(ValueError):self.d.decide('fixture',{**b,'choice':'decline'})
  with self.assertRaises(ValueError):self.d.decide('fixture',{'choice':'approve'})
 def test_expired_challenge_and_tampered_history(self):
  b=self.body();self.d.challenges[b['challenge']]['until']=0
  with self.assertRaises(ValueError):self.d.decide('fixture',b)
  b=self.body();self.d.decide('fixture',b);f=self.s.root/'human-decisions.jsonl';f.write_text(f.read_text().replace('approve','decline'))
  with self.assertRaisesRegex(ValueError,'integrity'):self.d.rows()
if __name__=='__main__':unittest.main()
