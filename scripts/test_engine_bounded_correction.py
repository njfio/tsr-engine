import importlib.util,tempfile,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('bc',Path(__file__).with_name('engine-bounded-correction.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Correction(unittest.TestCase):
 def result(self,code='malformed_json'):
  return {'value':None,'receipt':{'status':'settled','parse_location':'content','error_code':code,'usd':'0.01'}}
 def test_structured_diagnostic_rejects_untrusted_fields_and_wrong_types(self):
  import json
  value={'version':'native-shape-diagnostic@v1','code':'unsupported_field','path':'$.protocol','valid_locations':['$.response.protocol'],'expected_type':'object'}
  encode=lambda v:'Error: TSR_SHAPE_V1:'+json.dumps(v)
  self.assertEqual(m.native_feedback(encode(value)),value)
  for change in [{'raw_output':'private'},{'max_items':'one'},{'nonblank':'true'},{'expected_type':'invented'},{'path':'$.bad secret'},{'valid_locations':['$; secret']},{'code':'semantic_rejection'}]:
   self.assertIsNone(m.native_feedback(encode({**value,**change})))
 def test_default_zero_and_exhaustion(self):
  with tempfile.TemporaryDirectory() as t:
   path=Path(t)/'journal'
   with self.assertRaisesRegex(ValueError,'exhausted'):m.Budget(path,{},0).feedback('inquiry','root',self.result(),'r1')
   b=m.Budget(path,{},1);b.feedback('inquiry','root',self.result(),'r1')
   with self.assertRaisesRegex(ValueError,'exhausted'):b.feedback('review','root',self.result(),'r2')
 def test_restart_does_not_reuse_allowance_or_identity(self):
  with tempfile.TemporaryDirectory() as t:
   path=Path(t)/'journal';m.Budget(path,{'a':1},2).feedback('review','root',self.result(),'r1')
   with self.assertRaisesRegex(ValueError,'already reserved'):m.Budget(path,{'a':1},2).feedback('review','root',self.result(),'r1')
   with self.assertRaisesRegex(ValueError,'binding'):m.Budget(path,{'a':2},2).events()
 def test_unknown_semantic_and_identity_failures_no_retry(self):
  with tempfile.TemporaryDirectory() as t:
   b=m.Budget(Path(t)/'j',{},2)
   for r in [self.result('model_mismatch'),self.result('semantic_rejection'),{'value':None,'receipt':{'status':'unknown'}},{**self.result(),'resource_stop':'unknown_usage'}]:
    with self.assertRaisesRegex(ValueError,'not a known'):b.feedback('review','root',r,'r')
   self.assertEqual(b.events(),[])
 def test_only_structural_native_errors_not_lineage_are_correctable(self):
  with tempfile.TemporaryDirectory() as t:
   b=m.Budget(Path(t)/'j',{},1);r={'value':{},'receipt':{'status':'settled','usd':'0'}}
   for diagnostic in ['Error: response.state_root: exact value/identity required','Error: semantic rejection','Error: response.parents: required lineage missing']:
    with self.assertRaises(ValueError):b.feedback('review','root',r,'r',diagnostic)
   f=b.feedback('review','root',r,'r','Error: response.ids: JSON array required\n')
   self.assertEqual(f['code'],'response.ids: JSON array required')
 def test_feedback_is_full_fresh_no_raw_output_and_preserves_original(self):
  with tempfile.TemporaryDirectory() as t:
   b=m.Budget(Path(t)/'j',{},1);r=self.result('duplicate_key');r['raw']='secret untrusted response'
   f=b.feedback('inquiry','root',r,'r');body={'messages':[{'role':'user','content':'unchanged source'}]};new=b.body(body,f)
   self.assertEqual(len(body['messages']),1);self.assertEqual(new['messages'][:1],body['messages']);self.assertNotIn('secret',str(new))
class Capacity(unittest.TestCase):
 def test_settled_length_empty_or_partial_regenerates_without_output_repair(self):
  import copy
  admission={'version':'browser-project-cycle-adapter@v4','completion_tokens':32768,'incomplete_regeneration':True}
  r={'value':None,'receipt':{'status':'settled','usd':'0.00361','http_status':200,'response_status':'received','finish_reason':'length','error_code':'incomplete_output','parse_location':'wire','usage_status':'reported'}}
  with tempfile.TemporaryDirectory() as t:
   b=m.Budget(Path(t)/'j',admission,2)
   for n,raw in enumerate(('', '{"partial":')):
    f=b.feedback('proposal','same-source',{**r,'raw':raw},str(n))
    self.assertEqual(f['code'],'incomplete_output');self.assertNotIn('{"partial":',str(f))
    original={'messages':[{'role':'user','content':'fixed source and task'}]}
    self.assertEqual(b.body(original,f)['messages'][:-1],original['messages'])
   with self.assertRaisesRegex(ValueError,'exhausted'):b.feedback('proposal','same-source',r,'3')
   with self.assertRaisesRegex(ValueError,'already reserved'):m.Budget(Path(t)/'j',admission,2).feedback('proposal','same-source',r,'0')
  for change in ({'status':'unknown'},{'usd':None},{'finish_reason':'content_filter'},{'error_code':'model_mismatch'},{'http_status':503},{'usage_status':'inconsistent'}):
   with tempfile.TemporaryDirectory() as t:
    b=m.Budget(Path(t)/'j',admission,2)
    with self.assertRaises(ValueError):b.feedback('proposal','same-source',{'value':None,'receipt':{**r['receipt'],**change}},'x')
    self.assertEqual(b.events(),[])
  with tempfile.TemporaryDirectory() as t:
   with self.assertRaises(ValueError):m.Budget(Path(t)/'j',{},2).feedback('proposal','same-source',r,'x')
if __name__=='__main__':unittest.main()
