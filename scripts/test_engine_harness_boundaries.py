"""No-network local controls for capacity, provenance and preflight."""
import importlib.util,json,tempfile,unittest
from pathlib import Path
def load(name):
 s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+'.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
class Boundaries(unittest.TestCase):
 def test_schema_has_no_hidden_default_text_cap_or_empty_deletion_ban(self):
  m=load('engine-project-output');m.check('',{'type':'string'});m.check('x'*9000,{'type':'string'})
  with self.assertRaises(ValueError):m.check(' ',{'type':'string','minLength':1})
 def test_retention_uses_explicit_response_allocation_and_still_withholds_secret(self):
  m=load('engine-response-evidence')
  with tempfile.TemporaryDirectory() as t:
   f=m.FailureEvidence(t,'failed-response-1',limit=131072);raw=b'x'*100000
   f.capture(None,raw,None,False);proof=f.retain('test');self.assertFalse(proof['parts']['content']['truncated']);self.assertEqual((Path(t)/'failed-response-1/content.bin').read_bytes(),raw);f.clear()
  with tempfile.TemporaryDirectory() as t:
   f=m.FailureEvidence(t,'failed-response-1',limit=131072);f.capture(None,b'{"secret":"private"}',None,False);proof=f.retain('test');self.assertTrue(proof['parts']['content']['withheld']);f.clear()
 def test_exact_campaign_preflight_is_read_only_and_refuses_full_budget(self):
  m=load('engine-b91-campaign')
  with tempfile.TemporaryDirectory() as t:
   p=Path(t)/'ledger';manifest=m.manifest('a'*64,p,fixture=True);l=m.Ledger(p,manifest,m.digest(manifest),create=True);l.enroll('b'*64,'c'*64)
   body={'model':m.MODEL,'stream':False,'max_completion_tokens':32768,'messages':[{'role':'user','content':'Authored'}]}
   before=p.read_bytes();result=l.run_call('b'*64,'preflight',body,240,None,payload_limit=m.capacity.PAYLOAD_BYTES,preflight_only=True)
   self.assertFalse(result['mutated']);self.assertEqual(p.read_bytes(),before)
   for i in range(100):
    key=l.reserve('call','b'*64,str(i),'d'*64);l.settle('bill',key,0)
   before=p.read_bytes()
   with self.assertRaises(ValueError):l.run_call('b'*64,'exhausted',body,240,None,payload_limit=m.capacity.PAYLOAD_BYTES,preflight_only=True)
   self.assertEqual(p.read_bytes(),before)
if __name__=='__main__':unittest.main()
