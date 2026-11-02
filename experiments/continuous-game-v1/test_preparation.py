"""Authored local preparation-contract fixtures, never live model evidence."""
import copy,importlib.util,pathlib,unittest
p=pathlib.Path(__file__).with_name('continue.py');s=importlib.util.spec_from_file_location('continuation',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Preparation(unittest.TestCase):
 def setUp(self):
  self.roots=[{'id':'root','requirement':'Original want'}];child={'id':'child','requirement':'Generated wanting','eligible':True,'original_contributors':['root']}
  self.state={'coding':{'snapshot':'source','relational':{'inputs':{'parents':self.roots+[child],'acceptance_parents':self.roots,'generated_preferences':[child],'generated_preference_statuses':[{'id':'child','status':'open'}]}}}}
 def test_generated_child_preserved(self):m.verify_identity_continuity(self.state,copy.deepcopy(self.state),self.roots)
 def test_root_only_legacy(self):
  v=copy.deepcopy(self.state);v['coding']['relational']['inputs']={'parents':self.roots};m.verify_identity_continuity(v,v,self.roots)
 def test_changed_source(self):
  v=copy.deepcopy(self.state);v['coding']['snapshot']='other'
  with self.assertRaises(ValueError):m.verify_identity_continuity(self.state,v,self.roots)
 def test_child_dropped(self):
  v=copy.deepcopy(self.state);v['coding']['relational']['inputs']['parents']=self.roots
  with self.assertRaises(ValueError):m.verify_identity_continuity(self.state,v,self.roots)
 def test_lineage_changed(self):
  v=copy.deepcopy(self.state);v['coding']['relational']['inputs']['generated_preferences'][0]['original_contributors']=['foreign']
  with self.assertRaises(ValueError):m.verify_identity_continuity(self.state,v,self.roots)
 def test_obligation_changed(self):
  v=copy.deepcopy(self.state);v['coding']['relational']['inputs']['acceptance_parents']=[{'id':'root','requirement':'Easier'}]
  with self.assertRaises(ValueError):m.verify_identity_continuity(self.state,v,self.roots)
if __name__=='__main__':unittest.main()
