"""Authoritative wire-schema shape controls; no transport or model behavior."""
import importlib.util
import json
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('project_schema',Path(__file__).with_name('engine-project-output.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class SchemaTypes(unittest.TestCase):
 def diagnostic(self,value,schema,path,code):
  with self.assertRaises(m.FormatFailure) as caught:m.check(value,schema)
  detail=json.loads(caught.exception.diagnostic.split('TSR_SHAPE_V1:',1)[1])
  self.assertEqual((detail['path'],detail['code']),(path,code))
 def test_integer_is_not_number_or_boolean(self):
  s={'type':'object','required':['offset'],'properties':{'offset':{'type':'integer','minimum':0}}}
  m.check({'offset':0},s)
  for v in (True,1.0,'1',None):self.diagnostic({'offset':v},s,'$.offset','integer_required')
  with self.assertRaises(ValueError):m.check({'offset':-1},s)
 def test_boolean_and_null_are_not_ignored(self):
  m.check(False,{'type':'boolean'});m.check(None,{'type':'null'})
  for v in (0,1,'false',None):self.diagnostic(v,{'type':'boolean'},'$','boolean_required')
  for v in (False,0,'null',[]):self.diagnostic(v,{'type':'null'},'$','null_required')
 def test_boolean_schemas_for_empty_evidence_catalog(self):
  s={'type':'array','items':False}
  m.check([],s)
  with self.assertRaisesRegex(ValueError,r'\$\[0\]: schema forbids'):m.check(['invented'],s)
  m.check({'any':'value'},True)
  with self.assertRaisesRegex(ValueError,'schema forbids'):m.check({'x':0},{'type':'object','properties':{'x':{'$ref':'#/$defs/none'}},'$defs':{'none':False}})
 def test_oneof_checks_paired_source_identities(self):
  options=[{'type':'object','required':['path','digest'],'properties':{'path':{'const':path},'digest':{'const':digest}}} for path,digest in [('a.js','a'*64),('b.js','b'*64)]]
  schema={'type':'array','items':{'oneOf':options}}
  for path,digest in [('a.js','a'*64),('b.js','b'*64)]:m.check([{'path':path,'digest':digest}],schema)
  with self.assertRaises(ValueError) as caught:m.check([{'path':'a.js','digest':'b'*64}],schema)
  self.assertNotIsInstance(caught.exception,m.FormatFailure)
  self.assertIn('$[0].digest: identity mismatch',str(caught.exception))
  self.diagnostic(['not an object'],schema,'$[0]','object_required')
 def test_oneof_requires_exactly_one_matching_branch(self):
  with self.assertRaisesRegex(ValueError,'matched 2'):m.check(1,{'oneOf':[{'type':'number'},{'type':'integer'}]})
  with self.assertRaises(ValueError):m.check({}, {'oneOf':[]})
 def test_nullable_reference_resolves_from_original_root(self):
  s={'type':'object','required':['revision'],'properties':{'revision':{'$ref':'#/$defs/revision'}},'$defs':{'revision':{'oneOf':[{'type':'null'},{'type':'string','maxLength':64}]}}}
  m.check({'revision':None},s);m.check({'revision':'a'*64},s)
  with self.assertRaises(ValueError):m.check({'revision':42},s)
  m.check(None,{'type':['null','boolean']});m.check(True,{'type':['null','boolean']})
  with self.assertRaises(ValueError):m.check(1,{'type':['null','boolean']})
 def test_identity_types_cannot_coerce(self):
  for schema in ({'const':1},{'enum':[1]}):
   m.check(1,schema)
   with self.assertRaises(ValueError):m.check(True,schema)
  with self.assertRaises(ValueError):m.check('x',{'type':'not-a-json-type'})
 def test_actual_return_evidence_schema_shape(self):
  spec=importlib.util.spec_from_file_location('return_evidence',Path(__file__).with_name('engine-b91-evidence.py'))
  evidence=importlib.util.module_from_spec(spec);spec.loader.exec_module(evidence)
  request={'state_root':'a'*64,'coding':{'cases':[],'files':[{'path':'app.js','editable':True,'source_digest':'b'*64}], 'relational':{'inputs':{'snapshot':'c'*64,'returns':[]}}}}
  schema=evidence.schema(request)
  value={'binding':evidence.catalog(request)['binding'],'observed':[],'inferred':[{'path':'app.js','digest':'b'*64,'kind':'source_prediction','statement':'Authored hypothesis'}],'unknown':[]}
  m.check(value,schema);evidence.validate(value,request)
  value['inferred'][0]['digest']='d'*64
  with self.assertRaises(ValueError):m.check(value,schema)
  value['inferred']=[];value['observed']=['0']
  with self.assertRaises(ValueError):m.check(value,schema)

if __name__=='__main__':unittest.main()
