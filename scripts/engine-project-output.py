"""Shared native-project wire shape diagnostics. Semantic checks remain native.

Only structural defects qualify for format correction. Wrong identities, enums,
provenance and review judgments fail closed, never receive semantic coaching.
"""
import json
class FormatFailure(ValueError):
 def __init__(self,diagnostic):
  self.diagnostic=diagnostic
  super().__init__('known output format defect')
def fail(path,code,**fields):
 raise FormatFailure('Error: TSR_SHAPE_V1:'+json.dumps({'version':'native-shape-diagnostic@v1','path':path,'code':code,**fields},sort_keys=True))
def check(v,s,path='$',root=None):
 root=s if root is None else root
 if s is True:return
 if s is False:raise ValueError(path+': schema forbids a value')
 if not isinstance(s,dict):raise ValueError(path+': invalid schema')
 seen=set()
 while '$ref' in s:
  ref=s['$ref']
  if set(s)!={'$ref'} or not isinstance(ref,str) or not ref.startswith('#/$defs/') or '/' in ref[8:] or ref in seen:raise ValueError('invalid local contract reference')
  seen.add(ref);key=ref[8:]
  if key not in root.get('$defs',{}):raise ValueError('missing local contract definition')
  s=root['$defs'][key]
  if s is True:return
  if s is False:raise ValueError(path+': schema forbids a value')
  if not isinstance(s,dict):raise ValueError(path+': invalid referenced schema')
 if 'oneOf' in s:
  options=s['oneOf']
  if not isinstance(options,list) or not options:raise ValueError(path+': invalid oneOf schema')
  matches=0;errors=[]
  for option in options:
   try:check(v,option,path,root);matches+=1
   except ValueError as error:errors.append(error)
  if matches!=1:
   # Return a precise shared structural error when every branch agrees. A
   # wrong identity or ambiguous union remains terminal, never format coaching.
   if not matches and errors and all(isinstance(x,FormatFailure) for x in errors) and len({x.diagnostic for x in errors})==1:raise errors[0]
   details='; '.join(dict.fromkeys(str(x) if not isinstance(x,FormatFailure) else x.diagnostic for x in errors))
   raise ValueError(path+': expected exactly one schema alternative; matched '+str(matches)+(('; '+details) if details else ''))
 kind=s.get('type')
 if isinstance(kind,list):
  if not kind or len(kind)!=len(set(kind)):raise ValueError(path+': invalid type alternatives')
  check(v,{'oneOf':[{**s,'type':item} for item in kind]},path,root);return
 if kind not in (None,'object','array','string','number','integer','boolean','null'):raise ValueError(path+': unsupported schema type')
 if kind=='object':
  if not isinstance(v,dict):fail(path,'object_required',expected_type='object')
  props=s['properties'];required=s.get('required',[])
  if set(v)-set(props):fail(path,'unsupported_field',allowed_fields=list(props),required_fields=required)
  if not set(required)<=set(v):fail(path,'required_field',allowed_fields=list(props),required_fields=required)
  for k,value in v.items():check(value,props[k],path+'.'+k,root)
 elif kind=='array':
  if not isinstance(v,list):fail(path,'array_required',expected_type='array')
  if len(v)<s.get('minItems',0) or ('maxItems' in s and len(v)>s['maxItems']):fail(path,'array_length',min_items=s.get('minItems',0),**({'max_items':s['maxItems']} if 'maxItems' in s else {}))
  for i,value in enumerate(v):check(value,s['items'],path+'['+str(i)+']',root)
  if s.get('uniqueItems') and len({json.dumps(x,sort_keys=True) for x in v})!=len(v):raise ValueError('duplicate identities')
 elif kind=='string':
  if not isinstance(v,str):fail(path,'string_required',expected_type='string')
  minimum=s.get('minLength',0);maximum=s.get('maxLength')
  if (minimum and not v.strip()) or len(v.encode())<minimum or (maximum is not None and len(v.encode())>maximum):fail(path,'text_bound',**({'max_utf8_bytes':maximum} if maximum is not None else {}),nonblank=bool(minimum))
 elif kind in ('number','integer'):
  if kind=='integer' and type(v) is not int:fail(path,'integer_required',expected_type='integer')
  if type(v) not in (int,float) or not __import__('math').isfinite(v):fail(path,'number_required',expected_type='number')
  if v<s.get('minimum',float('-inf')) or v>s.get('maximum',float('inf')):raise ValueError('number outside admitted range')
 elif kind=='boolean':
  if type(v) is not bool:fail(path,'boolean_required',expected_type='boolean')
 elif kind=='null':
  if v is not None:fail(path,'null_required',expected_type='null')
 # Python equality treats True == 1; JSON identities must retain their types.
 def identical(a,b):return json.dumps(a,sort_keys=True,allow_nan=False)==json.dumps(b,sort_keys=True,allow_nan=False)
 if 'const' in s and not identical(v,s['const']):raise ValueError(path+': identity mismatch')
 if 'enum' in s and not any(identical(v,item) for item in s['enum']):raise ValueError(path+': unsupported enum/identity')
def proposal_shape(v,text_limit=512):
 if not isinstance(v,dict):fail('$','object_required',expected_type='object')
 if set(v)=={'abstain'}:check(v,{'type':'object','properties':{'abstain':{'type':'string','minLength':1,'maxLength':text_limit}},'required':['abstain']});return
 check(v,{'type':'object','properties':{'alternatives':{'type':'array','minItems':1,'maxItems':4,'items':{}}},'required':['alternatives']})
 for i,item in enumerate(v['alternatives']):
  if not isinstance(item,dict):fail('$.alternatives['+str(i)+']','object_required',expected_type='object')
def review_schema(binding,criteria,*,compact=False,text_limit=512):
 text={'type':'string','minLength':1,'maxLength':text_limit}
 finding={'type':'object','required':['finding','basis','evidence','explanation','limitations'],'properties':{
  'finding':{'enum':['met','not_met','unknown']},'basis':{'enum':['static_source','observed_test_receipt','unknown']+(['observed_information_receipt'] if 'information_receipts' in binding else [])},
  'evidence':{'type':'array','minItems':1,'maxItems':len(binding['evidence_ids']),'uniqueItems':True,'items':{'enum':binding['evidence_ids']}},'explanation':text,'limitations':text}}
 schema={'type':'object','required':['review_id','verdict','findings'],'properties':{
  'review_id':{'const':binding['review_id']},'verdict':{'enum':['accept','reject','unknown','abstain']},
  'findings':{'type':'object','required':list(criteria),'properties':{key:finding for key in criteria}}}}
 if compact:
  schema['$defs']={'finding':finding}
  schema['properties']['findings']['properties']={key:{'$ref':'#/$defs/finding'} for key in criteria}
 return schema
