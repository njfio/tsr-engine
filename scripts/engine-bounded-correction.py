"""Durable shared format-correction budget. No dispatch, billing or action replay.
Caller retains original response/charge. Only validated terminal format defects
qualify; valid semantic decisions and unknown effects never do. An explicit
criterion-only policy permits one contradictory-assessment regeneration. Zero by default.
"""
import json,os,hashlib,copy,re
from pathlib import Path
SHAPE_CODES=frozenset({'unsupported_field','required_field','string_required','array_required','object_required','number_required','integer_required','boolean_required','null_required','text_bound','array_length','unique_file_edits'})
def native_feedback(raw):
 if not isinstance(raw,str) or len(raw.encode())>8192:return None
 prefix='Error: TSR_SHAPE_V1:'
 if not raw.startswith(prefix):return None
 try:
  def unique(pairs):
   d={}
   for k,v in pairs:
    if k in d:raise ValueError('duplicate diagnostic')
    d[k]=v
   return d
  v=json.loads(raw[len(prefix):],object_pairs_hook=unique)
 except (ValueError,TypeError):return None
 allowed={'version','code','path','allowed_fields','required_fields','valid_locations','unknown_key_ordinal','field_name_withheld','expected_type','max_utf8_bytes','nonblank','min_items','max_items','actual_utf8_bytes'}
 if not isinstance(v,dict) or set(v)-allowed or v.get('version')!='native-shape-diagnostic@v1' or v.get('code') not in SHAPE_CODES|{'representation_size'}:return None
 path=r'\$(?:\.[A-Za-z_][A-Za-z_0-9]*|\[[0-9]*\])*'
 if not isinstance(v.get('path'),str) or not re.fullmatch(path,v['path']):return None
 for key in ('allowed_fields','required_fields','valid_locations'):
  if key in v and (not isinstance(v[key],list) or len(v[key])>64 or any(not isinstance(i,str) or len(i)>256 or not re.fullmatch(path if key=='valid_locations' else r'[A-Za-z_][A-Za-z_0-9]*',i) for i in v[key])):return None
 for key in ('max_utf8_bytes','min_items','max_items','unknown_key_ordinal','actual_utf8_bytes'):
  if key in v and (type(v[key]) is not int or not 0<=v[key]<=(8388608 if key=='actual_utf8_bytes' else 1048576)):return None
 for key in ('nonblank','field_name_withheld'):
  if key in v and type(v[key]) is not bool:return None
 if 'expected_type' in v and v['expected_type'] not in ('object','array','string','number','integer','boolean','null'):return None
 return v
def consistency_feedback(raw):
 prefix='Error: TSR_REVIEW_CONSISTENCY_V1:'
 if not isinstance(raw,str) or not raw.startswith(prefix) or len(raw.encode())>8192:return None
 try:v=json.loads(raw[len(prefix):])
 except ValueError:return None
 if not isinstance(v,dict) or v.get('version')!='criterion-consistency-diagnostic@v1' or v.get('code')!='criterion_verdict_findings_inconsistent':return None
 states=v.get('finding_states');verdict=v.get('verdict')
 if not isinstance(states,list) or not states or any(x not in ('met','not_met','unknown') for x in states) or len(states)!=len(set(states)):return None
 inconsistent=(verdict=='accept' and set(states)!={'met'} or verdict=='reject' and 'not_met' not in states or verdict=='unknown' and 'unknown' not in states)
 if not inconsistent:return None
 return {'category':'criterion_consistency','code':'criterion_verdict_findings_inconsistent','verdict':verdict,'finding_states':sorted(states),'instruction':'The settled response contradicts its own findings. Produce one complete fresh independent assessment of the identical original evidence and criteria. Do not infer which field was intended or mechanically change the verdict. Valid acceptance, rejection, uncertainty and abstention remain available. No new facts, tool execution, authority or desired result. A further invalid response ends this attempt.'}
CODES=frozenset({'malformed_json','duplicate_key','nonfinite'})
def digest(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':')).encode()).hexdigest()
class Budget:
 def __init__(self,path,admission,allowance=0):
  maximum=20 if admission.get('version') in ('browser-project-cycle-adapter@v3','browser-project-cycle-adapter@v4','browser-project-cycle-adapter@v5','browser-project-cycle-adapter@v6','browser-project-cycle-adapter@v7') else 2
  if type(allowance) is not int or not 0<=allowance<=maximum:raise ValueError('correction allowance outside admitted profile')
  self.path=Path(path);self.binding=digest(admission);self.allowance=allowance
  self.consistency=admission.get('criterion_return_profile',{}).get('consistency_reassessment')=='single-fresh@v1'
  self.incomplete=admission.get('incomplete_regeneration',False)
  if type(self.incomplete) is not bool or (self.incomplete and (admission.get('version') not in ('browser-project-cycle-adapter@v4','browser-project-cycle-adapter@v5','browser-project-cycle-adapter@v6','browser-project-cycle-adapter@v7') or admission.get('completion_tokens')!=32768)):raise ValueError('incomplete regeneration requires explicit capacity profile')
  self.representation=admission.get('representation_regeneration',False)
  self.representation_profile=admission.get('representation_profile')
  if type(self.representation) is not bool:raise ValueError('explicit boolean representation admission required')
 def events(self):
  rows=[]
  if self.path.exists():
   for line in self.path.read_text().splitlines():
    v=json.loads(line)
    if v['admission']!=self.binding or v['ordinal']!=len(rows)+1:raise ValueError('correction journal binding')
    rows.append(v)
  if len(rows)>self.allowance:raise ValueError('correction budget drift')
  return rows
 def feedback(self,stage,source,result,request_id,native_diagnostic=None,request_source=None):
  receipt=result.get('receipt',{})
  rows=self.events()
  if any(v['stage']==stage and v.get('feedback',{}).get('category')=='criterion_consistency' for v in rows):raise ValueError('single criterion reassessment already consumed; no further response')
  consistency=consistency_feedback(native_diagnostic)
  if consistency and (not self.consistency or not re.fullmatch(r'criterion_[0-9a-f]{64}_review',stage) or receipt.get('usd') is None or receipt.get('usage_status')!='reported' or request_source is None):raise ValueError('criterion reassessment not admitted or unsettled')
  incomplete=(self.incomplete and result.get('value') is None and receipt.get('status')=='settled' and receipt.get('usd') is not None and receipt.get('http_status')==200 and receipt.get('response_status')=='received' and receipt.get('finish_reason')=='length' and receipt.get('error_code')=='incomplete_output' and receipt.get('parse_location')=='wire' and receipt.get('usage_status')=='reported')
  detail=native_feedback(native_diagnostic)
  if detail and detail['code']=='representation_size':
   browser=self.representation_profile=='browser-exact-edits@v2'
   eligible=(bool(re.fullmatch(r'\$\.alternatives\[0\]\.edits\[[0-3]\]\.(old_text|new_text|resulting_file)',detail['path'])) and detail.get('max_utf8_bytes')==1048576) if browser else detail['path']=='$.alternatives[0].edits[0].replacement' and detail.get('max_utf8_bytes')==2048
   if receipt.get('usd') is None or not self.representation or stage!='proposal' or not eligible or type(detail.get('actual_utf8_bytes')) is not int or detail['actual_utf8_bytes']<=detail.get('max_utf8_bytes',1048576):
    raise ValueError('representation regeneration not admitted/eligible')
  shape=(detail['path']+': '+detail['code']) if detail else None
  if shape is None and isinstance(native_diagnostic,str):
   match=re.fullmatch(r'Error: (response(?:\.[A-Za-z_][A-Za-z_0-9]*|\[[0-9]+\])*: (?:JSON array required|string required|object required|required field|unsupported field|array length|nonblank UTF-8 byte bound))\s*',native_diagnostic)
   if match:shape=match.group(1)
  if result.get('resource_stop') or receipt.get('status')!='settled' or not (consistency or incomplete or shape or (receipt.get('parse_location')=='content' and receipt.get('error_code') in CODES)):
   raise ValueError('not a known settled content-format failure; no correction')
  rows=self.events()
  if any(v['failed_request']==request_id for v in rows):raise ValueError('correction already reserved; reconcile interrupted stage')
  if len(rows)>=self.allowance:raise ValueError('format correction exhausted; retained diagnostics, terminal stop')
  feedback={**({'category':'representation_size'} if detail and detail['code']=='representation_size' else {}),'native_structure':detail,'code':consistency['code'] if consistency else shape or receipt['error_code'],'position':receipt.get('parse_position'),'instruction':('The provider stopped the preceding response at the admitted completion capacity, including reasoning. Return a full fresh valid response for the same task/source, using a smaller complete incremental edit if needed. Do not continue, concatenate or repair the partial output. No change to evidence, permissions or acceptance; abstention remains valid.' if incomplete else 'Return a complete fresh response within the reported representation byte limit. Preserve the identical task, source context, requirements, evidence and permissions. Do not truncate the prior response or waive behavior; an inability/abstention remains valid.' if detail and detail['code']=='representation_size' else 'Return a full fresh response satisfying the identical contract and source context. Correct only the reported JSON format defect. Do not change task, evidence, identifiers, permissions or intended semantics.')}
  if consistency:feedback=consistency
  v={'admission':self.binding,'ordinal':len(rows)+1,'stage':stage,'source':source,'failed_request':request_id,'failure':consistency['code'] if consistency else shape or receipt['error_code']}
  if request_source is not None:v.update(feedback=feedback,request_source=copy.deepcopy(request_source))
  # Caller holds run lock. Exclusive append and fsync precede any new request.
  fd=os.open(self.path,os.O_WRONLY|os.O_CREAT|os.O_APPEND|os.O_NOFOLLOW,0o600)
  try:os.write(fd,(json.dumps(v,sort_keys=True)+'\n').encode());os.fsync(fd)
  finally:os.close(fd)
  return feedback
 def body(self,original,feedback):
  v=copy.deepcopy(original)
  v['messages'].append({'role':'user','content':json.dumps({'protocol_correction':feedback},sort_keys=True)})
  return v
