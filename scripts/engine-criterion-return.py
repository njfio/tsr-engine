"""Opt-in counted independent criterion assessment of an authentic ordinary return.
No operator acceptance, generated evidence, automatic goals, or private source export.
"""
import copy,json

def validate(profile):
 if profile is None:return
 if not isinstance(profile,dict) or set(profile)-{'version','policy','policy_sha256','imported_returns','consistency_reassessment'} or not {'version','policy','policy_sha256','imported_returns'}<=set(profile) or profile['version'] not in ('public-criterion-return@v1','public-criterion-return@v2'):raise ValueError('criterion profile')
 if 'consistency_reassessment' in profile and profile['consistency_reassessment']!='single-fresh@v1':raise ValueError('criterion reassessment policy')
 for key in ('policy','policy_sha256'):
  if not isinstance(profile[key],str) or len(profile[key])!=64 or any(c not in '0123456789abcdef' for c in profile[key]):raise ValueError('criterion policy binding')
 if not isinstance(profile['imported_returns'],list) or len(profile['imported_returns'])!=len(set(profile['imported_returns'])):raise ValueError('criterion import identities')

def assessment_contract(frame,profile):
 """Explicit published scope, never NLP extraction or a new requirement."""
 scoped=profile['version']=='public-criterion-return@v2';parts=frame.get('assessment_scope')
 if scoped!=(parts is not None):raise ValueError('criterion scope/profile version mismatch')
 if not scoped:return frame['requirements'],{'version':'legacy-unspecified','instruction':'Original legacy requirements are not domain-separated. Do not infer that an engine-history or permission instruction is a webpage feature. If the artifact obligation cannot be distinguished, report unknown rather than inventing one.'}
 if not isinstance(parts,dict) or set(parts)!=set(frame['requirements']):raise ValueError('complete criterion scope required')
 criteria={};invariants={}
 for key,items in parts.items():
  if not isinstance(items,list) or not items or any(not isinstance(p,dict) or set(p)!={'domain','text'} or p['domain'] not in ('artifact','engine_invariant') or not isinstance(p['text'],str) or not p['text'].strip() for p in items):raise ValueError('criterion scope part')
  if ''.join(p['text'] for p in items)!=frame['requirements'][key]:raise ValueError('criterion scope changes original text')
  selected=[p['text'] for p in items if p['domain']=='artifact']
  if not selected:raise ValueError('engine-only criterion cannot be artifact-assessed')
  criteria[key]='\n'.join(selected);invariants[key]=[p['text'] for p in items if p['domain']=='engine_invariant']
 return criteria,{'version':'artifact-assessment-scope@v1','original_requirements':frame['requirements'],'published_parts':parts,'engine_invariants_not_assessed':invariants,'instruction':'Findings assess ONLY the artifact portions in criteria. Met is not fulfillment of the entire mixed requirement. Engine invariants remain immutable controller obligations, not webpage features and not evidence of artifact unmet desire. This assessment cannot grant permission, accept the task, or assess preservation of engine history from editable source. The exact split is published policy, not model discretion.'}

def capture_system(capture):
 version=capture.get('contract_version','criterion-evidence@v1')
 if version=='criterion-evidence@v1':return LEGACY_SYSTEM
 if version=='criterion-evidence@v2':return SYSTEM
 if version=='criterion-evidence@v3':return CONTEXT_SYSTEM
 if version=='criterion-evidence@v4':return CONSISTENT_SYSTEM
 if version=='criterion-evidence@v5':return CONTROL_SYSTEM
 if version=='criterion-evidence@v6':return SUPPLEMENTAL_SYSTEM
 if version=='criterion-evidence@v7':return CUMULATIVE_SYSTEM
 raise ValueError('unknown criterion capture contract')


def resume_reassessment(adapter,name,body):
 """Consume a durable unused ticket only; never infer an unknown call outcome."""
 if adapter.admission.get('criterion_return_profile',{}).get('consistency_reassessment')!='single-fresh@v1':return False
 if (adapter.root/'STOP').exists() or (adapter.root/'terminal-output.json').exists():raise ValueError('criterion reassessment stopped or terminal')
 bound=adapter.recovery_load('criterion_resume_bound','engine-bound-output.py')
 wire,_=bound.prepare_review_body(body,aliases=True)
 source=adapter.recovery_load('criterion_correction_hash','engine-bounded-correction.py').digest(wire)
 rows,pending,done=adapter.read();events=[e for e in adapter.corrections.events() if e['stage']==name and e.get('feedback',{}).get('category')=='criterion_consistency']
 if not events:return False
 if len(events)!=1:raise ValueError('ambiguous criterion reassessment ticket')
 event=events[0];ident=event['failed_request'];key='format_correction_'+str(event['ordinal'])
 current=adapter.native('request',None);request_source={k:current[k] for k in ('state_root','source_snapshot','scope')}
 requests=[k.removeprefix('request_body_') for k in done if k.startswith('request_body_')]
 status=adapter.ledger.status()
 if status.get('stopped') or status.get('blocking_unknown_calls') or status.get('blocking_unknown_executions'):raise ValueError('criterion reassessment unresolved or stopped')
 permitted=[wire]+[adapter.corrections.body(wire,e['feedback']) for e in adapter.corrections.events() if e['ordinal']<event['ordinal'] and e['stage']==name and e['source']==source and e.get('request_source')==request_source and 'feedback' in e]
 if event['source']!=source or event.get('request_source')!=request_source or not requests or requests[-1]!=ident or done['request_body_'+ident]['body'] not in permitted or 'settle_'+ident not in done or not done.get('rejected_'+ident,{}).get('not_admitted'):raise ValueError('criterion reassessment evidence/settlement changed; no redispatch')
 record={'event':event,'feedback':event['feedback'],'source':source,'request_source':request_source}
 if pending:
  # This exact local copy is recoverable from its fsynced budget event. No
  # external dispatch, execution or other pending record is reconciled here.
  if set(pending)!={key}:raise ValueError('criterion reassessment has unresolved effect')
  adapter.append({'kind':'result','name':key,'intent':adapter.recovery_sha(pending[key]),'value':record})
 adapter.op(key,lambda:record)
 if adapter.read()[2][key]!=record:raise ValueError('criterion reassessment record drift')
 return True


def recover_reassessment_record(adapter):
 _,pending,done=adapter.read()
 if len(pending)!=1 or not next(iter(pending)).startswith('format_correction_'):return 0
 events=[e for e in adapter.corrections.events() if 'format_correction_'+str(e['ordinal']) in pending and e.get('feedback',{}).get('category')=='criterion_consistency']
 if len(events)!=1:raise ValueError('pending correction lacks exact criterion ticket')
 name=events[0]['stage'];capture=done.get(name.removesuffix('_review')+'_input')
 if capture is None or capture.get('contract_version') not in ('criterion-evidence@v4','criterion-evidence@v5','criterion-evidence@v6','criterion-evidence@v7'):raise ValueError('criterion reassessment capture missing')
 if not resume_reassessment(adapter,name,adapter.body(capture_system(capture),capture['payload'])):raise ValueError('criterion reassessment not admitted')
 return 1

def verify_retained(adapter,name,capture,review):
 done=adapter.read()[2];ident=review['request'];record=done.get('bound_output_'+ident)
 if record is None or record['association']['kind']!=name+'_review':raise ValueError('criterion invocation missing')
 bound=adapter.recovery_load('criterion_bound','engine-bound-output.py')
 if review['input_sha256']!=bound.digest(capture['payload']):raise ValueError('criterion request payload drift')
 wire=record.get('reference_request')
 body=done.get('request_body_'+ident,{}).get('body')
 if wire is None or wire!=body:raise ValueError('criterion retained request mismatch')
 source=record['source']
 if source!=capture['request_source'] or source['source_snapshot']!=capture['frame']['snapshot']:raise ValueError('criterion request source mismatch')
 bound.verify_envelope(record,admission=adapter.admission,ident=ident,dispatch=done['dispatch_'+ident],settled=done.get('settle_'+ident),source=source)
 # Reconstruct exactly the public review wire using the same representation
 # transform as checked(), including transport-owned singleton metadata.
 expected,metadata=bound.prepare_review_body(adapter.body(capture_system(capture),capture['payload']),aliases=True)
 candidates=[expected]
 correction_source=adapter.recovery_load('criterion_retained_correction_hash','engine-bounded-correction.py').digest(expected)
 for key,value in done.items():
  if key.startswith('format_correction_') and value['source']==correction_source and value['request_source']==source:
   candidates.append(adapter.corrections.body(expected,value['feedback']))
 if wire not in candidates or metadata!=record['host_metadata']:raise ValueError('criterion wire differs from captured evidence')
 if bound.digest(review['value'])!=record['native_content_sha256']:raise ValueError('criterion response substitution')
 return record

LEGACY_SYSTEM='Independently assess the original published criteria against this exact current source and authentic finite receipt. Follow each frozen prerequisite. Include exactly its complete coverage in each finding. Unknown is valid; do not turn unobserved enjoyment into failure. Passing finite tests does not establish broad semantics. Static evidence may establish a specific missing behavior. This review creates neither user acceptance nor authority, and is independent of whether a narrative reflection was accepted. Return the supplied JSON schema.'
SYSTEM='Independently assess the original published criteria against this exact current source and authentic finite receipt. Follow each frozen prerequisite and assessment_scope. Engine invariants and permissions are not artifact features; scoped met means only the published artifact portion is met. Include exactly its complete coverage in each finding. Unknown is valid; do not turn unobserved enjoyment into failure. Passing finite tests does not establish broad semantics. Static evidence may establish a specific missing behavior. This review creates neither user acceptance nor authority, and is independent of whether a narrative reflection was accepted. Return the supplied JSON schema.'


CONTEXT_SYSTEM=SYSTEM+' Interpret the criteria using published_context: its exact current condition and attribution are task context, not independently observed evidence or extra requirements. Respect explicit absence as well as presence in the published condition. Never infer future conditions or treat missing observation as unmet desire.'


VERDICT_RULES={
 'accept':'Every finding must be met.',
 'reject':'At least one finding must be not_met.',
 'unknown':'At least one finding must be unknown. Unobserved broader fulfillment belongs in limitations; do not use an unknown overall verdict with only met findings.',
 'abstain':'Explicit refusal to assess is valid; it does not accept the project or grant authority.'}
CONSISTENT_SYSTEM=CONTEXT_SYSTEM+' The overall verdict and individual findings must satisfy verdict_consistency. Check their agreement before returning. Do not invent uncertainty, unmet findings or acceptance to satisfy the contract; report only the assessment supported by the supplied evidence. Abstention remains valid.'
class CriterionConsistencyFailure(ValueError):
 code='criterion_verdict_findings_inconsistent'
 def __init__(self,verdict,states):
  self.diagnostic='Error: TSR_REVIEW_CONSISTENCY_V1:'+json.dumps({'version':'criterion-consistency-diagnostic@v1','code':self.code,'verdict':verdict,'finding_states':sorted(states),'rule':VERDICT_RULES[verdict],'disposition':'Response rejected unchanged. Which field was intended is unknown; no automatic normalization or format correction.'},sort_keys=True)
  super().__init__('criterion verdict/findings inconsistent: '+VERDICT_RULES[verdict])
def validate_assessment(value,binding,shape,schema):
 # Same strict native rules, with an explicit diagnostic. Not a shape failure.
 shape.check(value,schema)
 if value['review_id']!=binding['review_id']:raise ValueError('criterion review binding')
 states={f['finding'] for f in value['findings'].values()};verdict=value['verdict']
 if ((verdict=='accept' and states!={'met'}) or (verdict=='reject' and 'not_met' not in states) or (verdict=='unknown' and 'unknown' not in states)):
  raise CriterionConsistencyFailure(verdict,states)
 for f in value['findings'].values():
  if f['basis']=='observed_test_receipt' and not set(f['evidence'])&set(binding['observed_receipts']):raise ValueError('criterion observation citation missing')
 return value


CONTROL_SYSTEM=CONSISTENT_SYSTEM+' authenticated_control_observations contains only details retained under the named native test_receipt, after ownership and exact source validation. Ordered inputs and observed DOM text are finite behavior evidence; probe passed/met flags describe configured protocol checks, not fulfillment of your criteria. Missing, abbreviated or partial observations do not establish impossibility or unmet desire. Assess only what the observed sequence and current source support. Independent games are not a defect unless the published criteria explicitly require interaction. Do not infer enjoyment, hidden state, full reachability, a new desire or user acceptance.'

def control_observations(request,frame):
 """Project an already ownership-validated native request, never an attributed report.

 Called only after adapter.project validates export and execution lineage. Keep
 exact native case bytes/order; no output normalization or gameplay verdict.
 """
 coding=request['coding'];receipt=frame.get('receipt');snapshot=frame['snapshot']
 result={'version':'criterion-control-observations@v1','parent_evidence_id':'test_receipt','receipt':receipt,'snapshot':snapshot,'availability':'unavailable','cases':[], 'limitations':'Finite configured controls and DOM observations only. Protocol pass is not criterion fulfillment. Missing or partial coverage remains unknown; no human fulfillment or new action authority.'}
 if coding.get('kind')!='browser_project@v1':return result
 if coding['snapshot']!=snapshot:raise ValueError('criterion control source drift')
 observed=frame.get('observed_tests',{})
 if receipt is None or observed.get('receipt')!=receipt or observed.get('snapshot')!=snapshot:raise ValueError('criterion control receipt binding')
 matches=[r for r in coding['relational']['inputs'].get('returns',[]) if r.get('receipt')==receipt]
 if len(matches)>1:raise ValueError('duplicate criterion control receipt')
 if not matches:return result
 record=matches[0]
 if record.get('snapshot')!=snapshot:raise ValueError('stale criterion control receipt')
 cases=record.get('observations',[])
 if not isinstance(cases,list):raise ValueError('criterion control observations malformed')
 fields={'abbreviated','argv','case_id','exit','expected_stdout','passed','stderr','stdout'}
 if any(not isinstance(c,dict) or set(c)!=fields for c in cases):raise ValueError('criterion control case contract')
 if len({c['case_id'] for c in cases})!=len(cases):raise ValueError('duplicate criterion control case')
 result.update(availability='available' if cases else 'unavailable',cases=copy.deepcopy(cases))
 return result

def verify_controls(adapter,capture):
 if capture.get('contract_version') not in ('criterion-evidence@v5','criterion-evidence@v6','criterion-evidence@v7'):return
 # Re-run the normal authenticated projection; do not trust a report supplied by
 # the caller, nor silently replace captured details after interruption.
 request,_=adapter.project(adapter.native('request',None))
 if capture.get('contract_version') in ('criterion-evidence@v6','criterion-evidence@v7'):validate_supplemental_evidence(request,capture['frame'])
 if capture['payload'].get('authenticated_control_observations')!=control_observations(request,capture['frame']):raise ValueError('criterion control evidence changed')


SUPPLEMENTAL_SYSTEM=CONTROL_SYSTEM+' A prospectively published observed_supplemental prerequisite names an exact independently reviewed probe mapping and authenticated control receipt. Its raw evidence is a separate source-bound channel, not the original test_receipt and not SupplementalFinite success. Assess only the explicit artifact criterion from that mapped finite trace; all unavailable mapped observations require unknown. Passing the probe never automatically means met. Do not generalize a witnessed finite result to universal or human fulfillment.'

CUMULATIVE_SYSTEM=SUPPLEMENTAL_SYSTEM+' An observed_supplemental_set names the complete chronological set of current mapped observations. Read every listed receipt, including adverse, partial and unavailable observations. Do not replace cumulative coverage with the latest or most favorable trace. Conflicting behavioral returns and finite uncertainty must remain explicit; missing protocol observations require unknown. The order in prerequisites is the native receipt-import order, not lexical evidence-key order.'
def supplemental_bindings(policy):
 if policy['kind']=='observed_supplemental':return [{'admission':policy['admission'],'receipt':policy['receipt']}]
 if policy['kind']=='observed_supplemental_set':return policy['observations']
 return []

def validate_supplemental_evidence(request,frame):
 rows=request['coding'].get('relational',{}).get('inputs',{}).get('supplemental',[])
 expected={}
 for criterion,policy in frame.get('prerequisites',{}).items():
  if policy['kind']=='observed_supplemental_set':
   complete=[{'admission':r['admission'],'receipt':r['receipt']} for r in rows if criterion in r.get('mapped_cases',{}) and 'browser_cases' in r]
   if not complete or policy['observations']!=complete:raise ValueError('incomplete, reordered or conflicting cumulative evidence')
 bindings=[b for p in frame.get('prerequisites',{}).values() for b in supplemental_bindings(p)]
 for policy in bindings:
  key='supplemental:'+policy['admission']+':'+policy['receipt']
  value=json.loads(frame['evidence'][key])
  matches=[r for r in rows if r.get('admission')==policy['admission'] and r.get('receipt')==policy['receipt']]
  if len(matches)!=1 or any(value.get(k)!=matches[0].get(k) for k in ('admission','receipt','snapshot','checked_snapshot','bundle_digest','source_episode_root')) or value.get('snapshot')!=frame['snapshot']:raise ValueError('unowned or stale supplemental criterion evidence')
  allowed={k for k,p in frame['prerequisites'].items() if policy in supplemental_bindings(p)}
  row=matches[0];mapped={k:v for k,v in row.get('mapped_cases',{}).items() if k in allowed}
  if set(mapped)!=allowed or value.get('mapped_cases')!=mapped:raise ValueError('supplemental mapped coverage changed')
  ids={case for cases in mapped.values() for case in cases}
  raw=row.get('browser_cases')
  if not isinstance(raw,list) or value.get('browser_cases')!=[c for c in raw if c.get('case_id') in ids]:raise ValueError('supplemental raw observations changed or outside scope')
  expected[key]=value
 for key in frame['evidence']:
  if key.startswith('supplemental:') and key not in expected:raise ValueError('unpublished supplemental evidence')
 return set(expected)

def published_context(context,frame):
 value=context.get('published_context')
 if not isinstance(value,dict) or value.get('version')!='criterion-published-context@v1':raise ValueError('current criterion published context required')
 guidance=value.get('guidance');provenance=value.get('provenance',{})
 if not isinstance(guidance,dict) or set(guidance)!=set(frame['requirements']) or any(not isinstance(x,str) or not x.strip() for x in guidance.values()):raise ValueError('criterion guidance coverage')
 for key,expected in (('publication',frame['publication']),('requirements_revision',frame['requirements_revision']),('current_source',frame['snapshot'])):
  if provenance.get(key)!=expected:raise ValueError('criterion guidance binding mismatch')
 if provenance.get('status') not in ('current_publication','unchanged_guidance_rebound') or provenance.get('grants_authority') is not False or provenance.get('review_carried') is not False:raise ValueError('criterion guidance eligibility')
 return copy.deepcopy(value)

def verify(adapter,context):
 profile=adapter.admission.get('criterion_return_profile')
 if profile is None:return
 validate(profile);policy=context['policy']
 if policy is None or policy[0]!=profile['policy'] or adapter.recovery_sha(policy[1])!=profile['policy_sha256']:raise ValueError('criterion publication changed')
 assessment_contract(policy[1],profile)
 done=adapter.read()[2];matched=set()
 for row in context['history']:
  imported=adapter.recovery_sha(row)
  if imported in profile['imported_returns']:matched.add(imported);continue
  a=row['assessment'];name='criterion_'+adapter.recovery_sha(a['frame'])
  capture=done.get(name+'_input');review=done.get(name+'_review')
  if capture is None or review is None or capture['frame']!=a['frame']:raise ValueError('unowned criterion return')
  verify_retained(adapter,name,capture,review)
  if a['reviewer']!='separate-runtime-invocation:'+review['request'] or a['generator']!=capture['generator'] or a['response_digest']!=adapter.recovery_sha(review['value']) or a['decision']!=review['value']['verdict'] or a['findings']!=native_findings(review['value'],a['frame']):raise ValueError('criterion response binding mismatch')

 if matched!=set(profile['imported_returns']):raise ValueError('missing imported criterion history')

def project(adapter,request,context):
 success=request['coding']['relational']['inputs'].get('current_success')
 if adapter.admission.get('criterion_return_profile') is None:
  if success is not None:raise ValueError('scoped success requires admitted criterion evidence profile')
  return
 c=adapter.controller.cli('project','criterion-context');verify(adapter,c)
 if success is not None:
  current=c['current']
  if not current or current not in c['history']:raise ValueError('success assessment absent from authenticated history')
  a=current['assessment'];f=a['frame'];b=success['basis']
  expected={'assessment':current['id'],'receipt':f['receipt'],'publication':f['publication'],'requirements_revision':f['requirements_revision'],'source_snapshot':f['snapshot'],'criteria':sorted(k for k,v in a['findings'].items() if v['finding']=='met')}
  if any(b.get(k)!=v for k,v in expected.items()) or a['decision']=='abstain' or not expected['criteria'] or not f.get('assessment_scope'):
   raise ValueError('success differs from authenticated scoped assessment')
 context['criterion_returns']={'current':c['current'],'history':c['history'],'meaning':'Independent fallible assessments of explicitly published original criteria. Unknown is not unmet; scoped findings assess artifact portions only, never entire mixed requirements or engine invariants. No finding is user acceptance or new action authority.'}

def native_findings(value,frame):
 out={}
 for key,f in value['findings'].items():
  policy=frame['prerequisites'][key]['kind'];basis=f['basis']
  if basis=='unknown':b={'kind':'unknown'}
  elif policy=='static_semantic' and basis=='static_source':b={'kind':'static_semantic'}
  elif policy=='observed_tests' and basis=='observed_test_receipt':b={'kind':'observed_tests','observation':frame['observed_tests']}
  elif policy=='observed_supplemental_set' and basis=='observed_supplemental_set':b={'kind':'observed_supplemental_set','observations':copy.deepcopy(frame['prerequisites'][key]['observations'])}
  elif policy=='observed_supplemental' and basis=='observed_supplemental':b={'kind':'observed_supplemental','admission':frame['prerequisites'][key]['admission'],'receipt':frame['prerequisites'][key]['receipt']}
  elif policy=='mixed' and basis=='mixed':b={'kind':'mixed','observation':frame['observed_tests'],'static_finding':f['static_finding']}
  else:raise ValueError('criterion basis outside frozen prerequisite')
  if set(f['evidence'])!=set(frame['coverage'][key]):raise ValueError('incomplete criterion source coverage')
  out[key]={'finding':f['finding'],'basis':b,'evidence':sorted(f['evidence']),'explanation':f['explanation'],'limitations':f['limitations']}
 return out

def reflect(adapter):
 if adapter.admission.get('criterion_return_profile') is None:return None
 c=adapter.controller;context=c.cli('project','criterion-context');verify(adapter,context)
 frame=context['frame']
 if frame is None or context['current'] is not None:return None
 # Existing export validator proves current editable source, contributors and
 # executed source/receipt ownership before this new projection can be sent.
 view=adapter.native('request',None);request,public=adapter.project(view)
 if frame['snapshot']!=request['coding']['snapshot']:raise ValueError('criterion source drift')
 sources={f['path']:f['source'] for f in request['coding']['files'] if f['editable']}
 supplemental_keys=validate_supplemental_evidence(request,frame)
 if any(key!='test_receipt' and key not in supplemental_keys and sources.get(key)!=text for key,text in frame['evidence'].items()):raise ValueError('criterion evidence outside public source')
 done=adapter.read()[2];name='criterion_'+adapter.recovery_sha(frame)
 capture=done.get(name+'_input')
 shape=adapter.recovery_load('criterion_shape','engine-project-output.py')
 if capture is None:
  evidence=frame['evidence'];binding={'review_id':adapter.recovery_sha({'admission':adapter.admission,'frame':frame}),'evidence_ids':list(evidence),'observed_receipts':['test_receipt',*sorted(supplemental_keys)]}
  schema=shape.review_schema(binding,frame['requirements'],text_limit=adapter.narrative_limit)
  for key,policy in frame['prerequisites'].items():
   finding=copy.deepcopy(schema['properties']['findings']['properties'][key]);kind=policy['kind']
   finding['properties']['basis']['enum']=['unknown',{'static_semantic':'static_source','observed_tests':'observed_test_receipt','mixed':'mixed','observed_supplemental':'observed_supplemental','observed_supplemental_set':'observed_supplemental_set'}[kind]]
   if kind=='mixed':finding['required'].append('static_finding');finding['properties']['static_finding']={'enum':['met','not_met','unknown']}
   schema['properties']['findings']['properties'][key]=finding
  criteria,scope=assessment_contract(frame,adapter.admission['criterion_return_profile'])
  current_context=published_context(context,frame)
  schema['properties']['verdict']['description']=' '.join(k+': '+v for k,v in VERDICT_RULES.items())
  payload={'verdict_consistency':VERDICT_RULES,'published_context':current_context,'binding':binding,'criteria':criteria,'assessment_scope':scope,'prerequisites':frame['prerequisites'],'coverage':frame['coverage'],'evidence':evidence,'response_schema':schema,'authenticated_control_observations':control_observations(request,frame)}
  # Source provenance, not a claimed originating model invocation.
  # Independent reviewer association is retained by the counted transport.
  generator='native-source:'+frame['snapshot']
  capture=adapter.op(name+'_input',lambda:{'contract_version':'criterion-evidence@v7','frame':frame,'payload':payload,'generator':generator,'request_source':{k:view[k] for k in ('state_root','source_snapshot','scope')}})
 if capture['frame']!=frame:raise ValueError('criterion capture drift')
 if capture.get('contract_version') in ('criterion-evidence@v3','criterion-evidence@v4','criterion-evidence@v5','criterion-evidence@v6','criterion-evidence@v7') and capture['payload']['published_context']!=published_context(context,frame):raise ValueError('criterion published context changed before review')
 verify_controls(adapter,capture)
 pursuit=adapter.recovery_load('criterion_counted_stage','engine-pursuit-reflection.py')
 options={'validator':validate_assessment} if capture.get('contract_version') in ('criterion-evidence@v4','criterion-evidence@v5','criterion-evidence@v6','criterion-evidence@v7') else {}
 review=pursuit._stage(adapter,name+'_review',capture_system(capture),capture['payload'],**options)
 findings=native_findings(review['value'],frame)
 now=c.cli('project','criterion-context');verify(adapter,now)
 if now['frame']!=frame:raise ValueError('criterion evidence changed during review')
 if capture.get('contract_version') in ('criterion-evidence@v3','criterion-evidence@v4','criterion-evidence@v5','criterion-evidence@v6','criterion-evidence@v7') and capture['payload']['published_context']!=published_context(now,frame):raise ValueError('criterion published context changed during review')
 verify_controls(adapter,capture)
 value={'state_root':now['state_root'],'binding':now['binding'],'frame':frame,'reviewer':'separate-runtime-invocation:'+review['request'],'generator':capture['generator'],'response_digest':adapter.recovery_sha(review['value']),'decision':review['value']['verdict'],'findings':findings}
 adapter.op(name+'_native_input',lambda:value)
 return adapter.op(name+'_apply',lambda:c.json_cli(('project','criterion-apply'),value))


def recover(adapter):
 rows,pending,done=adapter.read()
 if len(pending)!=1:return 0
 name,intent=next(iter(pending.items()))
 if not name.startswith('criterion_') or not name.endswith('_apply'):return 0
 status=adapter.ledger.status();budget=adapter.controller.budget()
 unknown=set(status.get('unknown_calls',[]))-set(status.get('quarantined_unknown_calls',[]))
 if (adapter.root/'STOP').exists() or status['stopped'] or unknown or status.get('blocking_unknown_executions',status.get('unknown_executions',[])) or any(budget.get(k,0) for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests')):raise ValueError('criterion recovery stopped or externally unresolved')
 value=done.get(name.removesuffix('_apply')+'_native_input')
 if value is None:raise ValueError('missing criterion admission input; no replay')
 context=adapter.controller.cli('project','criterion-context');verify(adapter,context)
 current=context['current']
 if current is None or current['assessment']!=value:raise ValueError('criterion admission not proven committed; no replay')
 adapter.append({'kind':'result','name':name,'intent':adapter.recovery_sha(intent),'value':current['id']})
 return 1
