"""Authored structural restart guards; not native relational emergence evidence."""
import importlib.util,tempfile,unittest
from pathlib import Path
def load(name,file):
 s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
cycle=load('guard_cycle','engine-project-cycle.py');assessment=load('guard_assessment','engine-browser-assessment.py')
class Boundaries(unittest.TestCase):
 def test_relational_advances_do_not_shift_execution_binding(self):
  done={'advance_0':{'next':{'branch':'propose','binding':'relationship'}},'advance_1':{'next':{'branch':'resume','binding':'exact-reviewed-edit'}}}
  self.assertEqual(cycle.pending_resume_binding(done,0),'exact-reviewed-edit')
  done['resume_0']={};done['advance_2']={'next':{'branch':'propose','binding':'reviewed-development'}};done['advance_3']={'next':{'branch':'resume','binding':'second-edit'}}
  self.assertEqual(cycle.pending_resume_binding(done,1),'second-edit')
  with self.assertRaises(Exception):cycle.pending_resume_binding(done,0)
  done['advance_4']={'next':{'branch':'propose','binding':'unrelated'}}
  with self.assertRaises(Exception):cycle.pending_resume_binding(done,1)
 def test_checker_traversal_and_symlink_rejected(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d)/'root';root.mkdir();outside=Path(d)/'outside.cjs';outside.write_text('fixture');policy={'version':'source-bound-browser-assessment@v1','script':str(root/'..'/'outside.cjs'),'sha256':assessment.sha(outside),'limitations':'authored'}
   with self.assertRaises(ValueError):assessment.validate(policy,root)
   link=root/'link.cjs';link.symlink_to(outside);policy['script']=str(link)
   with self.assertRaises(ValueError):assessment.validate(policy,root)
if __name__=='__main__':unittest.main()
