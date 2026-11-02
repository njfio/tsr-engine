"""Prospective UI browser-session preparation. No inference or container execution.
Called only after explicit local Start, settled accounting, and inactive prior run.
Uses existing native authority/budget helpers; never edits the campaign ledger.
"""
import datetime,json,shutil,uuid
from pathlib import Path
import hashlib

def prepare(store,p,ledger_bundle=None,input_events=None):
 from server import ROOT,load,save,now
 a=load('ui_prepare_adapter','engine-browser-adapter.py')
 campaign,m,ledger,upgrade=ledger_bundle or store.ledger()
 status=ledger.status();unlimited=status.get('unlimited_policy_sha256')
 approval=store.decisions.approval(p)
 risk=not unlimited and any(k in status['unknown_calls'] for k in status.get('risk_acceptances',{}))
 if risk and (not approval or not any(v['decision_sha256']==m.digest(approval) for k,v in status['risk_acceptances'].items() if k in status['unknown_calls'])):raise ValueError('This project has no matching human continuation decision')
 allowance=approval['disclosure'] if risk else None
 max_calls=None if unlimited else min(200,status['effective_max_calls']-status['calls']) if risk else (None if unlimited else 200)
 max_operations=None if unlimited else min(100,status['effective_max_execution_groups']-status['execution_groups']) if risk else (None if unlimited else 100)
 max_cost=None if unlimited else status['effective_cap_microusd']-status['committed_microusd'] if risk else 10000000
 if not unlimited and (max_calls<2 or max_operations<1 or max_cost<=0):raise ValueError('Approved allowance exhausted')
 corrections=20 if unlimited else min(20,max_calls-2);decisions=None if unlimited else (max_calls-corrections)//2
 if not unlimited:corrections=max_calls-2*decisions
 if status['stopped']:raise ValueError('Unresolved campaign effects; preparation refused')
 source=store.project_native(p,create=True);meta=store.meta(p)
 out=store.path(p)/'sessions'/str(uuid.uuid4());out.mkdir(parents=True);shutil.copytree(source,out/'native')
 c=store.native(out/'native');budget=c.budget()
 if any(budget[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')):raise ValueError('Native effects require reconciliation')
 allocation=200 if unlimited else max_calls # Initial legacy accounting shape; explicit native unlimited policy supersedes cumulative ceilings.
 limits={'model_calls':budget['model_calls']+allocation,'model_tokens':budget['tokens_charged']+allocation*(a.base.capacity.INPUT_TOKENS+32768),'wall_ms':budget['elapsed_wall_ms']+7200000,'patch_bytes':budget['patch_bytes_charged']+104857600,'external_cost_microusd':budget['external_cost_charged_microusd']}
 life={} if meta.get('initial_text') and not meta.get('registered_packet') else c.cli('project-lifecycle-view')
 if budget.get('unlimited_mode') and budget.get('unlimited_policy_sha256')!=unlimited:raise ValueError('Existing unlimited policy differs; no implicit policy replacement')
 if life.get('admission') and not budget.get('unlimited_mode'):
  c.json_cli(('project-resource-extend',),{'state_root':c.request()['state_root'],'previous':life['admission'],'limits':limits,'tool_ms_ceiling':budget['tool_ms_charged']+7200000,'reason':'Explicit local UI Start, fixed browser-project policy; preserve lineage and history'},'--request')
 elif not life.get('admission'):
  c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':7200000,'reason':'Explicit local UI Start; fixed browser-project profile','non_tool':limits},'--admission')
  c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root'])
 if unlimited and not budget.get('unlimited_mode'):c.cli('project','resource-unlimited-admit','--state-root',c.request()['state_root'],'--policy-sha256',unlimited)
 # Rebind only authenticated public guidance after explicit scoped resource admission.
 # Never recover meanings from private requirements or treat absent eligibility as erased history.
 q=c.request();status_native=c.cli('project','acceptance','status')
 history=c.cli('project','acceptance','published-history')
 if history['state_root']!=q['state_root'] or history['current_source']!=q['coding']['snapshot']:raise ValueError('Contributor history changed during preparation')
 current=[v for v in history['publications'] if v['latest'] and v['current_revision']]
 if len(current)!=1 or history['eligibility'].get('status')=='intent_or_scope_changed':raise ValueError('This project needs a reviewed scope update before continuing; saved messages are intact.')
 contributors=current[0]['contributors']
 if not contributors or any(v['status'] not in ('open','fulfilled') for v in contributors):raise ValueError('A contributor is not eligible for continuation; no automatic reactivation.')
 guidance={v['requirement']:v['meaning'] for v in contributors}
 queued=input_events if input_events is not None else [e for e in store.events(p) if e['type'] in ('input','assessment')]
 for event in queued:
  if event.get('type')=='assessment':
   if event.get('attribution') not in ('independent_visual_review','independent_gameplay_assessment') or event.get('report_sha256')!=hashlib.sha256(event['text'].encode()).hexdigest() or not event.get('source_snapshot',{}).get('native_snapshot'):raise ValueError('Independent assessment provenance missing')
   continue # Evidence informs choice; it never rewrites contributing desires.
  marker='\nUser input ['+event['id']+']: '+event['text']
  guidance={key:meaning if marker in meaning else meaning+marker for key,meaning in guidance.items()}
 c.json_cli(('project','acceptance','feedback'),{'requirements_revision':status_native['requirements_revision'],'decision_record':status_native['decision_record'],'guidance':guidance})
 q=c.request();coding=q['coding'];inputs=coding['relational']['inputs'];config=c.cli('project','config')
 events=[{'id':e['id'],'text':e['text'],'attribution':e.get('attribution'),'scope_note':e.get('note'),'source_snapshot':e.get('source_snapshot'),**({'record_type':'independent_assessment','report_sha256':e['report_sha256'],'applicability':'current_source' if e['source_snapshot']['native_snapshot']==coding['snapshot'] else 'historical_source_not_current_validation'} if e.get('type')=='assessment' else {})} for e in queued]
 source_admission=Path(meta.get('registered_packet') or meta['source'])/'adapter-admission.json'
 prior_brief=json.loads(source_admission.read_text())['origin']['public']['public_brief'] if source_admission.exists() else meta.get('initial_text','')
 prior_records=[]
 paths=[Path(meta['source'])]
 if meta.get('registered_packet'):paths.append(Path(meta['registered_packet']))
 for previous in dict.fromkeys(paths):
  journal=previous/'adapter/adapter.jsonl'
  if not journal.exists():continue
  prior_ad=json.loads((previous/'adapter-admission.json').read_text())
  import types
  holder=types.SimpleNamespace(file=journal,admission=prior_ad,fixture=bool(campaign.get('fixture_only')) if previous==Path(meta.get('registered_packet') or '') else False,row=a.d.Driver.row,journal_limit=a.base.capacity.JOURNAL_BYTES)
  _,pending,done=a.d.Driver.read(holder)
  if pending:raise ValueError('Prior context has unresolved effects; no automatic export')
  for key,value in done.items():
   if key.startswith(('independent_assessment_','review_rejection_','reviewed_')):
    prior_records.append({'record':key,'journal':str(journal.relative_to(ROOT)),'source_admission_sha256':a.e.sha(prior_ad),'applicability':'historical; current-source claims require exact matching snapshot/file hashes','value':value})
 from context import package
 brief=package(prior_brief,prior_records,events)
 public={'version':'public-browser-origin@v1','scope':q['next_attention']['scope'],'requirements_revision':inputs['requirements_revision'],'parents':inputs.get('acceptance_parents',inputs['parents']),'public_preferences':[a.e.sha(x) for x in inputs.get('generated_preferences',[])],'public_meanings':[a.e.sha(x) for x in inputs.get('public_desires',{}).get('history',[])],'protected':{f['path']:f['source_digest'] for f in coding['files'] if not f['editable']},'editable':[f['path'] for f in coding['files'] if f['editable']],'public_brief':brief,'source_snapshots':[coding['snapshot']],'public_relationships':[a.e.sha(x) for x in coding['relational']['history']],'public_returns':[a.e.sha(x) for x in inputs.get('returns',[])]}
 if coding.get('pursuit',{}).get('history'):public['public_pursuits']=[a.e.sha(x) for x in coding['pursuit']['history']]
 origin={'directory':str(c.directory),'public':public,'initial_sources':{f['path']:f['source'] for f in coding['files'] if f['editable']}}
 expires=None if unlimited else approval['expires_at'] if risk else (datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(minutes=120)).isoformat();authority_id='ui-start-'+out.name
 policy={'version':'authorized-feedback-cycle@v2','authorization_id':authority_id,'max_continuations':decisions}
 observer=ROOT/'experiments/jungle-arcade-v1/observe.cjs'
 ad=a.admission(out/'adapter',origin,config,campaign,expires,max_decisions=decisions,max_executions=max_operations,correction_allowance=corrections,feedback_cycle=policy,transient_recovery={'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':120},independent_assessment={'version':'source-bound-browser-assessment@v1','script':str(observer),'sha256':a.d.file_sha(observer),'limitations':'Generic control/render observations, not comprehensive gameplay, aesthetics, enjoyment, or user acceptance.'},unlimited_policy_sha256=unlimited)
 ca={**({'unlimited_policy_sha256':unlimited} if unlimited else {}),'version':'local-project-cycle@v3','journal':str(out/'cycle'),'scope':public['scope'],'max_decisions':decisions,'max_executions':max_operations,'local_source_profile':'browser-scoped-source@v1','stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'feedback_cycle':policy,'expires_at':expires,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
 preview=a.web.project(q,public,context_budget=a.base.capacity.PAYLOAD_BYTES-65536)
 authority={'version':'local-ui-browser-session@v1','authorization_id':authority_id,'at':now(),'source_root':q['state_root'],'source_snapshot':coding['snapshot'],'input_ids':[e['id'] for e in events],'max_calls':max_calls,'max_operations':max_operations,'max_microusd':max_cost,'budget_mode':'unlimited' if unlimited else 'limited','unlimited_policy_sha256':unlimited,'human_decision':approval,'expires_at':expires,'final_user_acceptance':False,'baseline':status,'scope':'Existing user-requested isolated browser project only; no host actions or publication. Explicit cumulative budget policy retained.'}
 def no_dispatch(*args):raise RuntimeError('Preparation never dispatches')
 adapter=a.Adapter(ad['directory'],ad,c,ledger,no_dispatch if campaign.get('fixture_only') else None);grant=adapter.scope_grant(a.e.sha(authority),'local-user-start:reviewed-ui-browser-profile',max_cost,extend_authority=not risk)
 for name,value in {'adapter-admission':ad,'cycle-admission':ca,'campaign-upgrade':upgrade,'scope-authority':authority,'scope-grant':grant,'public-preview':preview,'prepared-manifest':{'adapter_sha256':a.e.sha(ad),'grant_sha256':m.digest(grant),'baseline':status,'expires_at':expires,'source_pins':ad['implementation'],'binary_sha256':ad['binary_sha256'],'input_ids':[e['id'] for e in events]}}.items():save(out/(name+'.json'),value)
 # Point only the isolated UI working lineage at its new copy. No delivered artifact changes.
 with store.lock(p):
  meta['native']=str(c.directory);meta['registered_packet']=str(out);save(store.path(p)/'project.json',meta);store.cache.pop(p,None)
  for event in queued:
   if event.get('type')=='assessment':
    if not any(x['type']=='assessment_projected' and x['id']==event['id'] for x in store.events(p)):
     store.append(p,{'id':event['id'],'type':'assessment_projected','role':'system','state':'prepared','text':'Independent assessment included with its source binding; not player feedback or native desire admission.','snapshot':coding['snapshot'],'report_sha256':event['report_sha256'],'model_consumed':False})
    continue
   if not any(x['type']=='native_input' and x['id']==event['id'] for x in store.events(p)):
    store.append(p,{'id':event['id'],'type':'native_input','role':'system','state':'native_admitted','text':'Your message is included in this prepared engine session. It has not yet been consumed by the model.','native_root':q['state_root'],'snapshot':coding['snapshot'],'source':'source-bound UI session preparation','model_consumed':False})
 return out
