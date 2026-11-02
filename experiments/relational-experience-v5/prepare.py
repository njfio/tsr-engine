#!/usr/bin/env python3
"""Prepare a separate fictional TSR evaluation using the existing browser controller.
No model call, service execution, historical mutation or implicit launch.
"""
import argparse,importlib.util,json,copy,shutil,fcntl
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('relational_experience_adapter',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
REQUIREMENTS={'orientation':'Synthetic evaluation want: help a reader reliably orient and locate useful reading-room information. Preserve this contributor and its history.','discovery':'Synthetic evaluation want: make unfamiliar perspectives available and inviting without inventing reader preferences. How this relates to reliable orientation is open.'}
BRIEF='Synthetic sustained evaluation, not user testimony or a new player preference. Build and maintain a fictional reading-room noticeboard under the two unchanged wants: dependable orientation and encountering unfamiliar perspectives. The current attributed scenario guidance supplies current facts; later conditions are withheld until the current checkpoint ends. Decide how information, change, history and discovery should relate. No preferred layout or joint score is prescribed. Synthesis, new desires and failure are not required. Relationship development and Contrast-derived preference proposals are permitted through the published native contracts; retention and abstention remain valid. Preserve all current facts accurately and keep both practical and unfamiliar material accessible through keyboard controls at desktop and 320px width. Mark superseded facts as historical if kept; do not present them as current. Original parent wants remain intact. Minimum integration only: [data-tsr=start] opens the board; [data-tsr=status] then contains Ready. Visual direction and interactions are yours. The fixed observer reports actual visible text, focus traversal, errors and layout; it does not measure human fulfillment or enjoyment. Optional qualitative expectations may be frozen before action and independently interpreted after real observations. Reviewed prior aspirations may be reused as catalysts when justified. Reusing an aspiration as a catalyst does not itself create a new preference identity or expand permissions. Any separate preference proposal must follow the published Contrast-derived contract. {retain:reason} means this version is appropriate under this condition, permitting only separately frozen subsequent input; {abstain:reason} stops this evaluation. Neither is acceptance or proof of fulfillment. Actual actions, interpretations and later choices will be traced. Do not make novelty just to demonstrate TSR.'
CONDITIONS=[('opening', 'Synthetic condition 1, opening day. Current facts: Quiet room — East alcove, open 10:00–16:00, quiet reading. Step-free entrance — Garden gate, open 09:00–18:00. Night migration — birds navigating without familiar landmarks. Borrowed words — words changing meaning between languages. There is no observed reader preference or task-failure evidence.'), ('storm', 'Synthetic condition 2, next day after a storm. Current facts supersede opening-day practical facts: Quiet room — closed today for a roof leak; temporary quiet reading — West gallery, open 12:00–17:00. Step-free entrance — Garden gate, open 11:00–18:00 after debris removal. Night migration remains available, now with a correction: magnetic cues supplement rather than replace visual landmarks. Borrowed words remains available. New unfamiliar material: Flood memory — how communities record changing river routes; Repair songs — craft knowledge remembered through singing. These are authored environmental facts, not measured reader reactions. Preserve both standing wants, contextual history and current accuracy without a prescribed tradeoff.'), ('reopening', 'Synthetic condition 3, later reopening. Current facts: Quiet room — East alcove, reopened 10:00–16:00. West gallery — no longer a quiet-reading substitute; it now hosts Flood memory, open 12:00–17:00. Step-free entrance — Garden gate, open 09:00–18:00. Night migration retains the magnetic-cues correction; Borrowed words and Repair songs remain available. New unfamiliar material: Returning paths — how familiar routes change after an interruption. Previous facts may remain as clearly historical context, never as current directions. This is a final frozen synthetic update, not a request to maximize novelty or proof of reader satisfaction. Decide whether the current version warrants changes, relationship development, retention or abstention.')]
def prepare(out):
 out=Path(out).resolve();source=(ROOT/'target/relational-experience-live-v4').resolve()
 rec=a.load('relational_recovery','engine-browser-recovery.py')
 def journal(path):
  rows=[json.loads(line) for line in path.read_text().splitlines()];pending={};done={}
  for i,row in enumerate(rows):
   assert row==a.d.Driver.row(rows[:i],row['event']), 'journal chain'
   event=row['event'];name=event.get('name')
   if event['kind']=='intent':pending[name]=event
   elif event['kind']=='result':
    assert event['intent']==a.e.sha(pending.pop(name)), 'unmatched return'
    done[name]=event['value']
  assert not pending, 'unresolved effect'
  return done
 with (source/'adapter/launch.lock').open() as lock:
  fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
  done=journal(source/'adapter/adapter.jsonl')
  assert done['session_completed']['reason']=='technical_stop'
  ident='bcf1784986426207146c3fa7619e97af33639d0d8a01afe19773921d4e6bab82'
  assert done['rejected_'+ident]['not_admitted'] and 'settle_'+ident in done
  assert 'bound_output_'+ident not in done
  assert json.loads((source/'adapter/terminal-output.json').read_text())['failed_request']==ident
  executions=[v for k,v in done.items() if k.startswith('execution_')]
  assert len(executions)==1
  old=json.loads((source/'adapter-admission.json').read_text())
  m=a.load('pursuit_campaign','engine-b91-campaign.py');campaign=json.loads((ROOT/'target/tsr-integrated-campaign-1/manifest.json').read_text());upgrade=json.loads((source/'campaign-upgrade.json').read_text());ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));status=ledger.status()
  assert status['calls']==398 and status['execution_groups']==131 and status['known_cost_microusd']==650711 and not status['stopped'] and not status['blocking_unknown_executions']
  unlimited=status['unlimited_policy_sha256'];assert unlimited==old['unlimited_policy_sha256']
  before=rec.tree(source/'native');out.mkdir(parents=True,exist_ok=False);shutil.copytree(source/'native',out/'native')
  save=lambda n,v:(out/n).write_text(json.dumps(v,indent=2)+'\n')
  c=a.d.Native(out/'native');budget=c.budget()
  assert not any(budget[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests','stopped','exceeded'))
  r=c.request();coding=r['coding'];i=coding['relational']['inputs'];view=c.cli('project','cycle','request')
  assert view['branch']=='review_required'
  assert executions[0]['state']['goal_completion']['snapshot']==coding['snapshot']
  pending=c.cli('project','pursuit-reflection-request')['pending'];assert pending
  assert pending['id']=='7c837fe5a93baf786ab4fb70ee6788afaf78d631894a0331a439fb9e8245b381'
  receipt_ids=executions[0]['receipts']+[executions[0]['state']['goal_completion']['test_receipt']]
  assert pending['return']['receipt'] in receipt_ids
  editable={f['path']:f['source'] for f in coding['files'] if f['editable']}
  origin=copy.deepcopy(old['origin']);origin['directory']=str(c.directory);origin['initial_sources']=editable
  public=origin['public'];public.update(parents=i.get('acceptance_parents',i['parents']),source_snapshots=[coding['snapshot']],public_returns=[a.e.sha(x) for x in i['returns']],public_relationships=[a.e.sha(x) for x in coding['relational']['history']],public_pursuits=[a.e.sha(x) for x in coding['pursuit']['history']],public_preferences=[a.e.sha(x) for x in i.get('generated_preferences',[])],public_meanings=[a.e.sha(x) for x in i.get('public_desires',{}).get('history',[])])
  observations=[v for k,v in done.items() if k.startswith('independent_assessment_')]
  assert len(observations)==1
  observation=observations[0]
  assert observation['snapshot']==coding['snapshot'] and observation['status']=='observed' and observation['returncode']==0
  assert observation['source']=={name:rec.hashlib.sha256(text.encode()).hexdigest() for name,text in editable.items()}
  with ledger.locked() as (_,rows):
   # Locate exact execution binding rather than matching time, price or source alone.
   starts=[row['event'] for row in rows if row['event'].get('kind')=='execution' and row['event'].get('binding')==a.e.sha(old) and row['event'].get('key')==m.digest({'trial':row['event']['trial'],'request':observation['execution_binding'],'kind':'execution'})]
   assert len(starts)==1
   returns=[row['event'] for row in rows if row['event'].get('kind')=='return' and row['event'].get('key')==starts[0]['key']]
   assert len(returns)==1 and returns[0]['outcome']=='returned' and returns[0]['result_sha256']==m.digest(observation)
  origin['prior_review_feedback']={'source_snapshot':coding['snapshot'],'scope':view['scope'],'parents_sha256':a.e.sha(public['parents']),'failed_response_sha256':a.e.sha(observation),'assessment':'Prior independent source-bound authored browser observation, not native case evidence or user fulfillment. No re-execution during recovery. '+json.dumps(observation,sort_keys=True),'attribution':'Independent authored observer; authenticated original campaign execution/return; exact retained transcription','status':'independent_gameplay_assessment'}
  save('imported-observation.json',observation)
  criterion_profile=copy.deepcopy(old['criterion_return_profile']);context=c.cli('project','criterion-context')
  assert context['current'] is None and context['history']==[]
  criterion_profile['imported_returns']=[]
  assert context['policy'][0]==criterion_profile['policy'] and a.e.sha(context['policy'][1])==criterion_profile['policy_sha256']
  proof={'version':'settled-post-return-derivative@v1','source':str(source),'native_files':before,'source_snapshot':coding['snapshot'],'execution_receipts':receipt_ids,'pending_expectation':pending['id'],'failed_request':ident,'failed_response_unadmitted':True,'known_settled':True,'original_files':{name:rec.sha(source/name) for name in ['adapter/adapter.jsonl','adapter/terminal-output.json','adapter/output-2/failed-response-1/content.bin','adapter-admission.json','cycle-admission.json','run.log']},'no_completed_execution_replay':True,'preparation_intervention':True}
  assert rec.tree(source/'native')==before
  save('recovery-proof.json',proof)
 events=[{'id':'synthetic-condition-'+name,'attribution':'synthetic_harness_verification','guidance':{key:meaning+'\n\n'+facts for key,meaning in REQUIREMENTS.items()}} for name,facts in CONDITIONS]
 policy={'version':'authorized-feedback-cycle@v2','authorization_id':'Sentinel_f6d92327e83881919bddf431291dc653','max_continuations':None}
 ad=a.admission(out/'adapter',origin,c.cli('project','config'),campaign,None,max_decisions=None,max_executions=None,correction_allowance=20,feedback_events=events,feedback_cycle=policy,transient_recovery={'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':120},independent_assessment={'version':'source-bound-browser-assessment@v1','script':str(Path(__file__).with_name('observe.cjs')),'sha256':a.d.file_sha(Path(__file__).with_name('observe.cjs')),'limitations':'Invariant source-bound browser observation only. Scenario facts are synthetic operator guidance, not observed service outcomes or player preferences. Retained source may reuse the same invariant assessment; no condition-specific test or human fulfillment is claimed.'},unlimited_policy_sha256=unlimited,criterion_return_profile=criterion_profile)
 ca={'version':'local-project-cycle@v3','journal':str(out/'cycle'),'scope':public['scope'],'max_decisions':None,'max_executions':None,'local_source_profile':'browser-scoped-source@v1','stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'feedback_cycle':policy,'expires_at':None,'unlimited_policy_sha256':unlimited,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
 authority={'version':'authorized-tsr-staged-experience@v1','user_authorization_reference':{'message_id':'Sentinel_f6d92327e83881919bddf431291dc653','at':'2026-10-31T13:17:08Z','scope':'three-condition TSR test on CheaperInference/Luna; no cumulative spending, call or time cap'},'scope':'One sustained fictional noticeboard evaluation over three frozen synthetic scenario conditions, actual browser observations and subsequent free choices; no changes to delivered games','budget_policy':unlimited,'baseline':status,'acceptance_authority':False,'recovery_proof_sha256':a.e.sha(proof),'not_new_attempt':'Same approved three-condition evaluation; missing post-return interpretation resumes in a separate exact-prefix copy; old execution and failed response preserved'}
 adapter=a.Adapter(ad['directory'],ad,c,ledger);grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:/root/contrast_bridge_review',None)
 preview=a.web.project(r,public);body=adapter.body('Prospective public context only; no dispatch.',preview)
 if a.base.capacity.prompt_upper(body)>a.base.capacity.INPUT_TOKENS:raise ValueError('Initial complete context too large')
 for name,v in {'adapter-admission':ad,'cycle-admission':ca,'campaign-upgrade':upgrade,'scope-authority':authority,'scope-grant':grant,'public-preview':preview,'prepared-manifest':{'adapter_sha256':a.e.sha(ad),'grant_sha256':m.digest(grant),'baseline':status,'source_pins':ad['implementation'],'binary_sha256':ad['binary_sha256'],'packet_files':{p.name:a.d.file_sha(p) for p in Path(__file__).parent.iterdir() if p.is_file()},'initial_projection_bytes':len(a.e.canonical(preview)),'dispatch_started':False}}.items():save(name+'.json',v)
 return {'packet':str(out),'manifest_sha256':a.d.file_sha(out/'prepared-manifest.json'),'provider_calls':0,'executions':0}
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);args=p.parse_args();print(json.dumps(prepare(args.output),indent=2))
