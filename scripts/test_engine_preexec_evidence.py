"""Authored process-failure evidence, not live execution proof."""
import copy,importlib.util,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('preexec',Path(__file__).with_name('engine-execution-not-started.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Evidence(unittest.TestCase):
 def setUp(self):
  trace="Traceback (most recent call last):\nengine-browser-assessment.py\np=run();\nsubprocess.run(cmd,capture_output=True,timeout=90)\nwith Popen(*popenargs, **kwargs) as process:\nself._execute_child(args, executable, preexec_fn, close_fds,\nraise child_exception_type(errno_num, err_msg, err_filename)\nFileNotFoundError: [Errno 2] No such file or directory: 'docker'\n"
  records={k:'/authored/'+k for k in ('admission','adapter','cycle','ledger')}
  self.e={'version':'execution-not-started-evidence@v1','binding':{k:'a'*64 for k in ('key','trial','admission_sha256','request_id','original_return_row')},'proof':{'version':'retained-python-preexec-error@v1','errno':2,'executable':'docker','traceback':trace,'log_path':'/authored/log'},'artifacts':{p:'b'*64 for p in ['/authored/log',*records.values()]},'missing_outputs':['/authored/receipt.json'],'worker_pid':999999,'source':{'/authored/app.js':'c'*64},'records':records}
 def test_exact_preexec_shape(self):self.assertEqual(m.validate_evidence(self.e),self.e['binding'])
 def test_postspawn_timeout_rejected(self):
  self.e['proof']['traceback']=self.e['proof']['traceback'].replace("FileNotFoundError: [Errno 2] No such file or directory: 'docker'",'TimeoutExpired: process may have run')
  with self.assertRaises(ValueError):m.validate_evidence(self.e)
 def test_generic_error_not_enough(self):
  self.e['proof']['traceback']="FileNotFoundError: [Errno 2] No such file or directory: 'docker'"
  with self.assertRaises(ValueError):m.validate_evidence(self.e)
 def test_missing_process_boundary_rejected(self):
  self.e['proof']['traceback']=self.e['proof']['traceback'].replace('self._execute_child(args, executable, preexec_fn, close_fds,','')
  with self.assertRaises(ValueError):m.validate_evidence(self.e)
 def test_other_executable_not_implicitly_allowed(self):
  self.e['proof']['executable']='sh'
  with self.assertRaises(ValueError):m.validate_evidence(self.e)
 def test_shape_is_not_current_proof(self):
  with self.assertRaises((ValueError,OSError)):m.validate_current_evidence(self.e)
if __name__=='__main__':unittest.main()
