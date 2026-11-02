"""Two independently reviewed components from one owned model deliberation.
No effect dispatch; source-bound derivations change host metadata only.
"""
import copy

DEVELOPMENT={'relational','public_desire','provisional_preference','success_preference','contrast_preference'}
ARTIFACT={'exact_edit','edit_set','existing'}

def contract():
 return {'envelope':'{deliberation:{artifact:<one edits/rationale payload or {retain:reason}>,development:<one supplied advisory payload or null>,artifact_dependency:independent|requires_development}}',
 'semantics':'One deliberation, separate decisions. Development is reviewed first. Artifact edits receive a separate review against the resulting current state before execution. Artifact retention does not assert unchanged meaning. Choose requires_development if the artifact intent depends on admission; rejection then leaves the artifact unexecuted. independent explicitly keeps the artifact intent even if development is rejected. Unknown external effects always stop. New wanting and development are optional. No invented experience, user preference or authority.'}

def shape(value):
 if not isinstance(value,dict) or set(value)!={'deliberation'}:raise ValueError('deliberation envelope')
 x=value['deliberation']
 if not isinstance(x,dict) or set(x)!={'artifact','development','artifact_dependency'} or x['artifact_dependency'] not in ('independent','requires_development'):raise ValueError('explicit component dependency required')
 if not isinstance(x['artifact'],dict) or (x['development'] is not None and not isinstance(x['development'],dict)):raise ValueError('component objects required')
 if x['development'] is None and x['artifact_dependency']!='independent':raise ValueError('dependency without development')
 return x

def expand(value,refs,request,expand_one):
 x=shape(value);out=copy.deepcopy(x)
 for name in ('artifact','development'):
  item=x[name]
  if item is not None and set(item)!={'retain'}:
   out[name]=expand_one({'alternatives':[item]},refs,request)['alternatives'][0]
 return {'deliberation':out}

def guard(request):
 c=request['coding'];i=c['relational']['inputs']
 # Resource charges may advance, but admission validation and native preview
 # independently recheck remaining authority. No caller-selected omissions.
 return {'snapshot':c['snapshot'],'requirements_revision':i['requirements_revision'],
         'files':[{k:f.get(k) for k in ('path','source_digest','editable')} for f in c['files']],
         'acceptance_parents':copy.deepcopy(i.get('acceptance_parents',i['parents'])),
         'parents':copy.deepcopy(i['parents']),
         'guidance':copy.deepcopy(c['relational'].get('guidance',[])),
         'history':copy.deepcopy(c['relational'].get('history',[]))}

def unchanged(before,after,development=None,admission=None):
 for key in ('snapshot','requirements_revision','files','acceptance_parents'):
  if before[key]!=after[key]:raise ValueError('deliberation source/obligation drift: '+key)
 # Only the exact admitted component can alter meaning/lineage. All other
 # existing contributor and relationship content remains byte-identical.
 proposal=development['proposals'][0] if development else {}
 changed_id=proposal.get('proposal',{}).get('identity') if proposal.get('kind')=='public_desire' and admission else None
 old={v['id']:v for v in before['parents']};new={v['id']:v for v in after['parents']}
 if changed_id:
  admitted={v['id']:v for v in admission['public']['active']}
  if new.get(changed_id)!=admitted.get(changed_id):raise ValueError('changed meaning differs from exact admitted result')
 for ident,parent in old.items():
  if ident not in new or (new[ident]!=parent and ident!=changed_id):raise ValueError('deliberation contributor drift')
 allowed_new=set()
 if admission and admission['advance_result'].get('preference_admitted'):
  allowed_new={p['id'] for p in admission['generated_preferences'] if p['admission']==admission['event']}
 if set(new)-set(old)!=allowed_new:raise ValueError('unrelated new contributor')
 if allowed_new:
  admitted={p['id']:p for p in admission['parents']}
  if any(new[k]!=admitted.get(k) for k in allowed_new):raise ValueError('new contributor differs from exact admission')
 prior={v['admission']:v for v in before['history']};later={v['admission']:v for v in after['history']}
 revises=proposal.get('proposal',{}).get('revises') if proposal.get('kind')=='relational' and admission else None
 for ident,row in prior.items():
  current=later.get(ident)
  if current is None or any(current.get(k)!=row.get(k) for k in ('admission','binding','proposal','withdrawn')):raise ValueError('relationship content/status drift')
  if current.get('superseded')!=row.get('superseded') and ident!=revises:raise ValueError('unrelated relationship supersession')
 additions={admission['event']} if admission and proposal.get('kind')=='relational' else set()
 if set(later)-set(prior)!=additions:raise ValueError('unrelated relationship admission')
 if additions:
  admitted={v['admission']:v for v in admission['history']}
  for ident in additions | ({revises} if revises else set()):
   if later.get(ident)!=admitted.get(ident):raise ValueError('relationship differs from exact admitted result')
 # current/guidance flags are native folds: admitting any development may move
 # prior guidance to history. Keep the exact old proposals visible to the reviewer.
 if any(v not in after['history'] or not v.get('current') for v in after['guidance']):raise ValueError('unowned current guidance')

def rebind(response,request):
 result=copy.deepcopy(response)
 if set(result)=={'retain'}:return result
 # Only artifact components are rebound after development. Semantic advisory
 # components must be processed against the unchanged initial context.
 if any(p['kind'] not in ARTIFACT for p in result['proposals']):raise ValueError('only unchanged artifact metadata may be rebound')
 result['state_root']=request['state_root'];result['attention_binding']=request['next_attention']['binding']
 return result

def derive(adapter,value):
 adapter.validate_current_admission();adapter.ledger.require_output_eligible(adapter.trial)
 if any(not k.startswith(('derivation_component_','generated_component_')) for k in adapter.read()[1]) or (adapter.root/'STOP').exists():raise ValueError('pending effect or Stop')
 whole=value['response'];name=value['component']
 if name not in ('artifact','development'):raise ValueError('component kind')
 done=adapter.read()[2]
 matches=[(k[10:],v) for k,v in done.items() if k.startswith('generated_') and not k.startswith('generated_component_') and v['response']==whole]
 if len(matches)!=1:raise ValueError('unowned or ambiguous original deliberation')
 origin,record=matches[0];owned=adapter.base_bound_component_origin(origin)
 if owned['decoded_sha256']!=adapter.component_sha([whole,record['return_basis']]):raise ValueError('original deliberation association')
 parts=whole['deliberation'];original=parts[name]
 if original is None:raise ValueError('missing component')
 view=adapter.native('request',None);request,_=adapter.project(view);current=guard(request)
 if name=='development':
  if current!=parts['guard']:raise ValueError('development context changed before review')
  response=copy.deepcopy(original)
 else:response=rebind(original,request)
 basis=copy.deepcopy(record['return_basis'][name])
 proof={'version':'owned-deliberation-component@v1','origin':origin,'component':name,
        'original_response_sha256':adapter.component_sha(whole),'original_component_sha256':adapter.component_sha(original),
        'original_source':owned['source'],'target_source':{k:view[k] for k in ('state_root','source_snapshot','scope')},
        'target_attention_binding':request['next_attention']['binding'],'before_guard':parts['guard'],'after_guard':current,'response':response,'return_basis':basis,
        'development_outcome':copy.deepcopy(value.get('development_outcome')),
        'metadata_transformation':[] if response==original else ['state_root','attention_binding']}
 if name=='artifact' and parts['development'] is not None:
  capture=disposition(adapter,origin,whole,proof['development_outcome'])
  unchanged(parts['guard'],current,proof['development_outcome']['response'],capture)
 else:unchanged(parts['guard'],current)
 ident='component_'+adapter.component_sha(proof)
 local_record(adapter,'derivation_'+ident,proof)
 local_record(adapter,'generated_'+ident,{'response':response,'return_basis':basis})
 return {'response':response,'derivation':ident}

def verify(adapter,ident,response,basis,historical_context=None):
 done=adapter.read()[2];proof=done.get('derivation_'+ident)
 if not proof or ident!='component_'+adapter.component_sha(proof) or proof['response']!=response or proof['return_basis']!=basis:raise ValueError('component derivation substitution')
 origin=adapter.base_bound_component_origin(proof['origin']);record=done['generated_'+proof['origin']];whole=record['response'];parts=whole['deliberation'];name=proof['component']
 if origin['decoded_sha256']!=adapter.component_sha([whole,record['return_basis']]) or proof['original_response_sha256']!=adapter.component_sha(whole) or proof['original_component_sha256']!=adapter.component_sha(parts[name]):raise ValueError('component origin substitution')
 if proof['original_source']!=origin['source']:raise ValueError('component original source substitution')
 if proof['before_guard']!=parts['guard'] or record['return_basis'][name]!=basis:raise ValueError('component evidence substitution')
 if historical_context is None:
  current=adapter.native('request',None);request,_=adapter.project(current)
  if proof['after_guard']!=guard(request):raise ValueError('stale component guard')
 else:
  current=historical_context
  request={'state_root':current['state_root'],'next_attention':{'binding':proof['target_attention_binding']}}
 if proof['target_source']!={k:current[k] for k in ('state_root','source_snapshot','scope')}:raise ValueError('stale component review source')
 capture=disposition(adapter,proof['origin'],whole,proof['development_outcome']) if name=='artifact' and parts['development'] is not None else None
 unchanged(proof['before_guard'],proof['after_guard'],proof['development_outcome']['response'] if capture else None,capture)
 expected=rebind(parts[name],request) if name=='artifact' else parts[name]
 if response!=expected:raise ValueError('component semantic mutation')
 return {'derivation':proof,'original_association':origin,'decoded_sha256':adapter.component_sha([response,basis]),'association':{'kind':'proposal','attribution':'derived component; not another model response'}}


def disposition(adapter,origin,whole,outcome):
 done=adapter.read()[2];parts=whole['deliberation']
 if not isinstance(outcome,dict) or set(outcome)!={'response','judgment','advance'}:raise ValueError('missing development disposition')
 judgment=outcome['judgment'];rid=judgment['reviewer'].removeprefix('separate-runtime-invocation:')
 reviewed=done.get('reviewed_'+rid);bound=adapter.base_bound_component_origin(rid)
 if not reviewed or bound['association']['kind']!='review' or adapter.component_sha(reviewed['report'])!=bound['native_content_sha256'] or 'candidate:'+judgment['response_digest'] not in reviewed['binding']['evidence_ids']:raise ValueError('unowned development review')
 actual={'accept':'approve'}.get(reviewed['report'].get('verdict'),reviewed['report'].get('verdict'))
 if judgment['decision']!=actual:raise ValueError('development decision substitution')
 gid=reviewed.get('generated_request','');generated=done.get('generated_'+gid);proof=done.get('derivation_'+gid)
 if not generated or generated['response']!=outcome['response'] or not proof or proof['origin']!=origin or proof['component']!='development' or proof['original_component_sha256']!=adapter.component_sha(parts['development']) or proof['response']!=parts['development'] or gid!='component_'+adapter.component_sha(proof):raise ValueError('cross-deliberation development substitution')
 if parts['artifact_dependency']=='requires_development' and actual!='approve':raise ValueError('artifact depends on unadmitted development')
 if actual!='approve':
  if outcome['advance'] is not None:raise ValueError('rejection with admission')
  return None
 receipt=outcome['advance']
 if not isinstance(receipt,dict):raise ValueError('missing native admission')
 capture=done.get(('qualitative_' if receipt.get('qualitative_admitted') else 'meaning_admission_')+receipt['proposal_record'])
 if not capture or capture.get('advance_result')!=receipt or capture.get('advance_input',{}).get('response')!=outcome['response']:raise ValueError('unowned development admission receipt')
 if any(capture['advance_input']['review'].get(k)!=v for k,v in judgment.items()):raise ValueError('admission/review substitution')
 return capture

def local_record(adapter,name,value):
 rows,pending,done=adapter.read()
 if name in done:
  if done[name]!=value:raise ValueError('component record drift')
  return value
 if pending:
  if set(pending)!={name} or pending[name].get('local_record_profile')!='component-value@v1' or pending[name].get('local_value')!=value:raise ValueError('different unfinished local component record')
  intent=pending[name]
 else:
  intent={'kind':'intent','name':name,'at':__import__('datetime').datetime.now(__import__('datetime').timezone.utc).isoformat(),'local_record_profile':'component-value@v1','local_value':copy.deepcopy(value)}
  adapter.append(intent)
 adapter.append({'kind':'result','name':name,'intent':adapter.component_sha(intent),'value':value})
 return value


def capture_relationship(adapter,value,result):
 key='qualitative_'+result['proposal_record']
 def capture():
  history=adapter.controller.request()['coding']['relational']['history']
  if len([r for r in history if r['admission']==result['relational_admission']])!=1:raise ValueError('missing admitted relationship')
  return {'event':result['relational_admission'],'advance_input':copy.deepcopy(value),'advance_result':copy.deepcopy(result),'history':copy.deepcopy(history)}
 pending=adapter.read()[1]
 if pending:
  if set(pending)!={key}:raise ValueError('unrelated pending relationship capture')
  if adapter.controller.json_cli(('project','cycle','meaning-advance-receipt'),value)!=result:raise ValueError('relationship capture receipt drift')
  adapter.append({'kind':'result','name':key,'intent':adapter.component_sha(pending[key]),'value':capture()})
 else:adapter.op(key,capture)
 return result
