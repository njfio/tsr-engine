import unittest, importlib.util
from pathlib import Path
s=importlib.util.spec_from_file_location('retained_browser',Path(__file__).with_name('engine-browser-adapter.py'));a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
class RetainedVerifier(unittest.TestCase):
 def test_exact_predecessor_and_current_only(self):
  current=Path(a.__file__).with_name('engine-browser-harness.cjs').read_text()
  prior=current.replace("page.on('pageerror',e=>error('page error: '+String(e.message).slice(0,160)));","page.on('pageerror',()=>error('page error'));").replace("page.on('console',m=>{if(m.type()==='error')error('console error: '+String(m.text()).slice(0,160));});","page.on('console',m=>{if(m.type()==='error')error('console error');});")
  self.assertTrue(a.supported_harness(current));self.assertTrue(a.supported_harness(prior))
  self.assertFalse(a.supported_harness(prior+'\n'));self.assertFalse(a.supported_harness(prior.replace("await route.abort()","await route.continue()")));self.assertFalse(a.supported_harness(None))
