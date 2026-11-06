"""Qualitative return interpretation over existing counted transport and native log.
Not a selector, goal generator, utility score or acceptance authority. A frozen
expectation is optional. Rejected interpretations stay criticism, never guidance.
"""
import copy,json

CRITERIA={
 'evidence_grounding':'Separate the frozen anticipation, actual version-bound return, attributed assessments and untested inferences. Do not infer player fulfillment from a browser check.',
 'evidence_coverage':'Account for separately supplied current-source attributed reports and their explicit coverage/limitations. They are fallible reports, not native receipts. Do not erase available reported content by treating native smoke assertions as the only supplied channel. Relevance, disagreement and unknown are legitimate when explained; citation alone does not establish learning.',
 'calibration':'Supported/contradicted/unknown and any revised expectation must match the available evidence and limitations. Retention and uncertainty are valid.',
 'contrast_and_authority':'Distinguish what remains wanted from prediction accuracy. Preserve contributing desires/history and all action permissions. Possibilities are optional hypotheses, not admitted desires or execution orders. A later ordinary decision may propose honestly attributed generated wanting through separate independent review; this interpretation cannot admit it.'}

LEGACY_GENERATION_SYSTEM='Interpret this frozen expectation in light of the authentic return. Return the supplied JSON schema. You are the generator, not the independent assessor. Preserve uncertainty and original contributors. Separate prediction accuracy from desired-versus-actual contrast. A null revised_expectation retains it; an empty possibilities array is valid. Do not invent desires, player preferences, observations or authority. Return only an interpretation, not a self-assessment verdict or findings.'
GENERATION_SYSTEM='Interpret this frozen expectation using every relevant supplied channel: native return, current source and separately citable attributed observer reports. Respect each channel’s coverage and limitations; a native smoke check not asserting text does not erase text reported by a separate observer. Return the supplied JSON schema. You are the generator, not the independent assessor. Preserve uncertainty and original contributors. Separate prediction accuracy from desired-versus-actual contrast. A null revised_expectation retains it; an empty possibilities array is valid. Do not invent desires, player preferences, observations or authority. Return only an interpretation, not a self-assessment verdict or findings.'

DEVELOPMENT_SYSTEM=GENERATION_SYSTEM.replace('Do not invent desires, player preferences, observations or authority.', 'Do not invent user preferences, observations, fulfillment or authority. Optional possibilities may suggest generated wanting with explicit uncertainty; only the separate ordinary proposal and independent admission path can make it a native desire.')

# v4: the same evidence rules, without framing an empty answer as the expected one.
# Possibilities and a revised expectation are weighed on the evidence like retaining both.
BALANCED_SYSTEM=DEVELOPMENT_SYSTEM.replace('A null revised_expectation retains it; an empty possibilities array is valid.',
 'Revise the expectation when the return bears on it, and offer possibilities, including possible generated wanting, when the return or source suggests them; a null revised_expectation or an empty possibilities array is equally valid when nothing does. Unknown is not unmet.')
assert BALANCED_SYSTEM!=DEVELOPMENT_SYSTEM
def capture_system(capture):
 version=capture.get('contract_version','qualitative-evidence@v1')
 if version=='qualitative-evidence@v1':return LEGACY_GENERATION_SYSTEM
 if version=='qualitative-evidence@v2':return GENERATION_SYSTEM
 if version=='qualitative-evidence@v3':return DEVELOPMENT_SYSTEM
 if version=='qualitative-evidence@v4':return BALANCED_SYSTEM
 raise ValueError('unknown reflection capture contract')

def project(adapter,request,context):
 history=request['coding'].get('pursuit',{}).get('history',[])
 if not history:return
 done=adapter.read()[2]
 snapshots=set(adapter.admission['origin']['public']['source_snapshots'])
 snapshots.update(v['snapshot'] for k,v in done.items() if k.startswith('executed_source_'))
 generated=[v['response'] for k,v in done.items() if k.startswith('generated_')]
 owned_receipts=set()
 for k,v in done.items():
  if k.startswith('execution_'):
   x=v.get('execution',v);owned_receipts.update(x.get('receipts',[]))
   receipt=x.get('state',{}).get('goal_completion',{}).get('test_receipt')
   if receipt:owned_receipts.add(receipt)
 parents={p['id'] for p in context['parents']}
 imported=adapter.admission['origin']['public'].get('public_pursuits',[])
 if not isinstance(imported,list) or len(imported)!=len(set(imported)):raise ValueError('pursuit import identities')
 for h in history:
  if adapter.recovery_sha(h) in imported:continue  # Exact reviewed public admission; native reconstruction still checks every join.
  base=copy.deepcopy(h);base['reviewed_reflections']=[];base['reflection_reviews']=[]
  inherited_base=adapter.recovery_sha(base) in imported
  f=h['frozen'];expectation=f['expectation']
  if not inherited_base and (f['source_before'] not in snapshots or not set(expectation['parents'])<=parents):raise ValueError('unowned pursuit source/contributors')
  if not inherited_base and not any(r.get('pursuit_expectations',{}).get(str(f['index']))==expectation for r in generated):raise ValueError('unowned generated expectation')
  if not inherited_base and h['return'] and h['return']['receipt'] not in owned_receipts:raise ValueError('unowned expectation return')
  for review in h['reflection_reviews']:
   match=done.get('pursuit_assessment_'+h['id'])
   # During apply recovery native state may be one durable record ahead. The
   # exact retained review/interpretation identities must still be available.
   generated_return=done.get('pursuit_interpretation_'+h['id'])
   if match is None or generated_return is None or review['reviewer']!='separate-runtime-invocation:'+match['request'] or review['generator']!='separate-runtime-invocation:'+generated_return['request']:raise ValueError('unowned expectation interpretation review')
   expected=generated_return['value']['interpretation']
   if any(review['reflection'][k]!=v for k,v in expected.items()) or review['decision']!=match['value']['verdict'] or review['review_note']!=json.dumps(match['value'],sort_keys=True):raise ValueError('substituted interpretation or assessment')
   for retained in (match,generated_return):
    envelope=done.get('bound_output_'+retained['request'])
    if envelope is None or envelope['native_content_sha256']!=adapter.recovery_sha(retained['value']):raise ValueError('interpretation response provenance missing')
 context['pursuit_history']=copy.deepcopy(request['coding']['pursuit'])
 context['pursuit_history']['use']='Use admitted interpretations as fallible guidance for later possibilities; rejected/unknown assessments are criticism only. No new action, relationship or revised expectation is compulsory. Receipt citation does not establish causal learning.'

def _validate(value,binding,shape,schema):
 shape.check(value,schema)
 findings=value['findings'];states={x['finding'] for x in findings.values()}
 if value['review_id']!=binding['review_id']:raise ValueError('reflection review binding')
 if value['verdict']=='accept' and states!={'met'}:raise ValueError('reflection acceptance requires all criteria met')
 if value['verdict']=='reject' and 'not_met' not in states:raise ValueError('reflection rejection requires criticism')
 if value['verdict']=='unknown' and 'unknown' not in states:raise ValueError('reflection uncertainty missing')
 for f in findings.values():
  if f['basis']=='observed_test_receipt' and not set(f['evidence'])&set(binding['observed_receipts']):raise ValueError('reflection observation citation missing')
 return value

def _validate_interpretation(value,binding,shape,schema):
 shape.check(value,schema)
 if value['review_id']!=binding['review_id']:raise ValueError('reflection interpretation binding')
 return value

def _stage(adapter,name,system,payload,*,interpretation_only=False,validator=None):
 """Reuse only an authenticated exact retained response; never repeat a settled call."""
 from importlib.util import spec_from_file_location,module_from_spec
 from pathlib import Path
 spec=spec_from_file_location('pursuit_bound',Path(__file__).with_name('engine-bound-output.py'));bound=module_from_spec(spec);spec.loader.exec_module(bound)
 spec=spec_from_file_location('pursuit_shape',Path(__file__).with_name('engine-project-output.py'));shape=module_from_spec(spec);spec.loader.exec_module(shape)
 validate=validator or (_validate_interpretation if interpretation_only else _validate)
 done=adapter.read()[2]
 if name in done:
  if done[name]['input_sha256']!=bound.digest(payload):raise ValueError('retained reflection payload drift')
  adapter.verify_owned_output(done[name]['request'])
  return done[name]
 body=adapter.body(system,payload)
 matches=[(k.removeprefix('bound_output_'),v) for k,v in done.items() if k.startswith('bound_output_') and v['association']['kind']==name]
 if len(matches)>1:raise ValueError('multiple settled reflection responses')
 if matches:
  ident,record=matches[0];adapter.verify_owned_output(ident)
  wire=record['reference_request'];original=json.loads(wire['messages'][1]['content'])
  if original['binding']['review_id']!=payload['binding']['review_id']:raise ValueError('retained reflection evidence drift')
  value=bound.expand_review(bound.attach_review(record['model_content'],record['host_metadata']),wire)
  validate(value,payload['binding'],shape,payload['response_schema'])
 else:
  started=[k for k,v in done.items() if k.startswith('request_body_') and json.loads(v['body']['messages'][1]['content']).get('binding',{}).get('review_id')==payload['binding']['review_id']]
  if started and not (hasattr(adapter,'can_resume_checked') and adapter.can_resume_checked(name,body)) and not (validator is not None and hasattr(adapter,'resume_criterion_reassessment') and adapter.resume_criterion_reassessment(name,body)):raise ValueError('reflection response stage interrupted before durable semantic binding; reconcile retained response, no redispatch')
  ident,value,_=adapter.checked(name,body,lambda v:validate(v,payload['binding'],shape,payload['response_schema']),review=True)
 record={'request':ident,'value':value,'input_sha256':bound.digest(payload)}
 return adapter.op(name,lambda:record)

def reflect(adapter):
 """Only after a known action return. No model-authored evidence or extra action."""
 from importlib.util import spec_from_file_location,module_from_spec
 from pathlib import Path
 spec=spec_from_file_location('pursuit_shape',Path(__file__).with_name('engine-project-output.py'));shape=module_from_spec(spec);spec.loader.exec_module(shape)
 c=adapter.controller;request=c.cli('project','pursuit-reflection-request');h=request['pending']
 if h is None:return None
 identity=h['id'];done=adapter.read()[2]
 # A finished assessment (including rejection) makes native pending null. Stop
 # and external uncertainty remain enforced by checked()/native transport.
 capture_name='pursuit_input_'+identity
 capture=done.get(capture_name)
 if capture is None:
  view=adapter.native('request',None);current_request,context=adapter.project(view)
  evidence={'context:'+adapter.recovery_sha(context):context,'receipt:'+h['return']['receipt']:h['return'],'expectation:'+identity:h['frozen']}
  spec=spec_from_file_location('pursuit_assessment_evidence',Path(__file__).with_name('engine-browser-assessment.py'));observer=module_from_spec(spec);spec.loader.exec_module(observer)
  for report in observer.reflection_evidence(adapter,current_request,context):evidence['assessment:'+report['report_sha256']]=report
  binding={'review_id':adapter.recovery_sha({'admission':adapter.admission,'expectation':identity,'evidence':evidence}),'evidence_ids':list(evidence),'observed_receipts':['receipt:'+h['return']['receipt']]}
  schema={'type':'object','required':['review_id','interpretation'],'properties':{'review_id':{'const':binding['review_id']}}}
  text={'type':'string','minLength':1,'maxLength':adapter.narrative_limit}
  interpretation={'type':'object','required':['conclusion','observed','revised_expectation','uncertainty','contrast','possibilities'],'properties':{'conclusion':{'enum':['supported','contradicted','unknown']},'observed':text,'revised_expectation':{'oneOf':[text,{'type':'null'}]},'uncertainty':text,'contrast':text,'possibilities':{'type':'array','maxItems':4,'items':text}}}
  schema['properties']['interpretation']=interpretation
  payload={'binding':binding,'criteria':copy.deepcopy(CRITERIA),'evidence':evidence,'response_schema':schema}
  capture=adapter.op(capture_name,lambda:{'contract_version':'qualitative-evidence@v4','expectation':h['frozen'],'return':h['return'],'snapshot':request['snapshot'],'payload':payload})
 if capture['expectation']!=h['frozen'] or capture['return']!=h['return'] or capture['snapshot']!=request['snapshot']:raise ValueError('frozen reflection input source drift')
 payload=capture['payload'];evidence=payload['evidence'];binding=payload['binding'];criteria=payload['criteria'];generation_system=capture_system(capture)
 generated=_stage(adapter,'pursuit_interpretation_'+identity,generation_system,payload,interpretation_only=True)
 candidate=generated['value']['interpretation']
 evidence={**evidence,'interpretation:'+generated['request']:candidate}
 review_binding={**binding,'review_id':adapter.recovery_sha({'binding':binding,'candidate':candidate}),'evidence_ids':list(evidence)}
 review_payload={'binding':review_binding,'criteria':criteria,'evidence':evidence,'response_schema':shape.review_schema(review_binding,criteria,text_limit=adapter.narrative_limit)}
 assessment=_stage(adapter,'pursuit_assessment_'+identity,'Independently assess this proposed interpretation against every criterion. Reject unsupported claims; unknown and abstain are valid. No obligation to endorse a changed interpretation or new possibility. Return the supplied JSON schema. This assessment cannot accept the project or grant actions.',review_payload)
 # Host-owned singleton bindings refer to the exact same frozen return, verified
 # again after both calls. Never repair any model-supplied source metadata.
 now=c.cli('project','pursuit-reflection-request')
 if now['pending']['id']!=identity or now['pending']['return']!=h['return'] or now['snapshot']!=request['snapshot']:raise ValueError('pursuit return changed during reflection')
 value={'state_root':now['state_root'],'reflection':{**candidate,'binding':now['binding'],'expectation':identity,'receipt':h['return']['receipt'],'source_after':now['snapshot']},'reviewer':'separate-runtime-invocation:'+assessment['request'],'generator':'separate-runtime-invocation:'+generated['request'],'note':json.dumps(assessment['value'],sort_keys=True),'decision':assessment['value']['verdict']}
 # Atomic native append. A timeout is uncertain local admission; never redispatch
 # a provider or execute. On restart native context authenticates any committed
 # assessment and request becomes pending:null.
 return c.json_cli(('project','pursuit-reflection-apply'),value)
