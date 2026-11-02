"""Prospective ordinary review after a proven format-only request defect.

Retains generation under its original admission. New admission and immutable
prefix proofs govern the corrected request; no exhausted ticket is reset.
"""
import copy, importlib.util, json
from pathlib import Path
s=importlib.util.spec_from_file_location('format_recovery',Path(__file__).with_name('engine-browser-failed-review.py'))
r=importlib.util.module_from_spec(s);s.loader.exec_module(r)
o=r.ordinary;a=r.a;e=r.e;d=r.d

def prefix(parent,proof):
 p=Path(parent);r.verify_prefix(proof)
 rows,pending,done=r.history_module.history(p/'adapter/adapter.jsonl')
 r.require(not pending and rows[-1]['hash']==proof['adapter_head'],'parent must be frozen')
 ca=r.read(p/'cycle-admission.json');cr,cp,cd=r.history_module.history(p/'cycle/cycle.jsonl')
 r.require(cr[-1]['hash']==proof['cycle_head'] and list(cp)==[proof['stage']],'parent cycle changed')
 ids=[k[10:] for k,v in done.items() if k.startswith('generated_') and v['response']==proof['callback']['response']]
 r.require(len(ids)==1,'one original generated proposal required');ident=ids[0]
 keys=['generated_'+ident,'bound_output_'+ident,'dispatch_'+ident,'settle_'+ident,'request_body_'+ident]
 keys += [k for k in done if k.startswith('proposal_native_')]
 return {'proof':proof,'admission':r.read(p/'adapter-admission.json'),'cycle':ca,'generated_id':ident,'inherited':{k:done[k] for k in keys},'proposal_result':cd['proposal_'+proof['stage'][7:]]}

def verify(p):
 r.verify_prefix(p['proof']);root=Path(p['proof']['parent'])
 ar,ap,ad=r.history_module.history(root/'adapter/adapter.jsonl');cr,cp,cd=r.history_module.history(root/'cycle/cycle.jsonl')
 r.require(not ap and ar[-1]['hash']==p['proof']['adapter_head'] and cr[-1]['hash']==p['proof']['cycle_head'],'frozen parent journal changed')
 r.require(all(ad[k]==v for k,v in p['inherited'].items()),'retained output changed')
 r.check_stop(p['proof'])

def compare_wire(old,new):
 """Only format instruction and trusted review identifier may differ."""
 before=copy.deepcopy(old);after=copy.deepcopy(new)
 r.require(' Return a JSON object.' not in before['messages'][0]['content'] and after['messages'][0]['content'].count(' Return a JSON object.')==1 and after['messages'][0]['content'].replace(' Return a JSON object.','',1)==before['messages'][0]['content'],'non-format system change')
 after['messages'][0]['content']=before['messages'][0]['content']
 b=json.loads(before['messages'][1]['content']);n=json.loads(after['messages'][1]['content'])
 if 'reference_catalog' in b or 'reference_catalog' in n:
  a.base.bound_output.expand_review({'findings':{}},old)
  a.base.bound_output.expand_review({'findings':{}},new)
  n['reference_catalog']['binding_sha256']=b['reference_catalog']['binding_sha256']
 # Review ID is transport-owned and changes with the fresh admission.
 if 'review_id' in b.get('binding',{}):
  r.e.digest(n['binding']['review_id']);n['binding']['review_id']=b['binding']['review_id']
 r.require(n==b,'substantive review evidence or schema changed')
 after['messages'][1]['content']=before['messages'][1]['content']
 r.require(after==before,'model/settings/message structure changed')
 return {'old_sha256':e.sha(old),'new_sha256':e.sha(new),'difference':'explicit JSON instruction; fresh transport review identity only','substantive_payload_equal':True}

class Adapter(o.Adapter):
 def __init__(self,*args,prefix,**kwargs):self.prefix=prefix;super().__init__(*args,**kwargs)
 def read(self):
  rows,pending,done=super().read()
  return rows,pending,{**self.prefix['inherited'],**done}
 def validate_current_admission(self):verify(self.prefix);return super().validate_current_admission()
 def verify_owned_output(self,ident):
  p=self.prefix
  if ident!=p['generated_id']:return super().verify_owned_output(ident)
  verify(p);v=self.native('request',None);source={k:v[k] for k in ('state_root','source_snapshot','scope')};h=p['inherited']
  return a.base.bound_output.verify_envelope(h['bound_output_'+ident],admission=p['admission'],ident=ident,dispatch=h['dispatch_'+ident],settled=h['settle_'+ident],source=source)
 def proposal(self,view):
  if not any(k.startswith(('reviewed_','review_rejection_')) for k in self.read()[2]):
   r.require({k:view[k] for k in ('state_root','source_snapshot','scope')}==self.prefix['proof']['source'],'stale retained proposal')
   self.verify_owned_output(self.prefix['generated_id'])
   self.op('historical_proposal_reuse',lambda:{'original_admission':e.sha(self.prefix['admission']),'request':self.prefix['generated_id'],'proposal_sha256':e.sha(self.prefix['proposal_result']),'new_provider_call':False})
   result=self.prefix['proposal_result']
   return {'response':copy.deepcopy(result['response']),'receipt':{**copy.deepcopy(result['receipt']),'mode':result['callback_mode'],'status':'settled'}}
  return super().proposal(view)
 def checked(self,kind,body,validate,review=False):
  if kind=='review' and not any(k.startswith('reviewed_') for k in self.read()[2]):
   wire,_=a.base.bound_output.prepare_review_body(body,aliases=getattr(self,'review_reference_aliases',False))
   match=compare_wire(self.prefix['proof']['body'],wire)
   self.op('format_only_review_wire',lambda:match)
  return super().checked(kind,body,validate,review=review)
