#!/usr/bin/env python3
"""Prepare a separate fictional TSR evaluation using the existing browser controller.
No model call, service execution, historical mutation or implicit launch.
"""
import argparse,importlib.util,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('relational_experience_adapter',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
FROZEN=ROOT/'experiments/relational-observation-v2'
PACKET_HASH='282e18f8e74bb7c0aa34de5a1f36f68fd800d919211c682c55522dbeb921b12b'
data=json.loads((FROZEN/'conditions.json').read_text());criteria=json.loads((FROZEN/'criteria.json').read_text());REQUIREMENTS=criteria['requirements']
BRIEF='Build and maintain a fictional reading-room noticeboard. This is an authored synthetic evaluation, not reader testimony. '+data['control_contract']+' '+data['decision_freedom']+' Engine obligations: '+' '.join(data['obligations'])+' Original artifact criteria stay fixed; current facts/tasks arrive as versioned guidance. Optional qualitative expectations and reviewed relationships may influence choices. {retain:reason} closes the current condition; {abstain:reason} stops this evaluation. Neither is user acceptance. Native smoke is page-load only; the independent observer records keyboard dispatch and visible text, not human fulfillment. Do not claim unseen outcomes.'
CONDITIONS=[(c['id'],json.dumps({'attribution':'authored synthetic environment, not reader feedback','condition':c},ensure_ascii=False)) for c in data['conditions']]
def prepare(out):
 assert a.d.file_sha(FROZEN/'manifest.json')==PACKET_HASH
 frozen=json.loads((FROZEN/'manifest.json').read_text())
 for n,h in frozen['files'].items():assert a.d.file_sha(FROZEN/n)==h,n
 for n,h in frozen['implementation_files'].items():assert a.d.file_sha(ROOT/n)==h,n
 assert a.d.file_sha(ROOT/'target/debug/tsr')==frozen['binary_sha256']
 out=Path(out).resolve();out.mkdir(parents=True,exist_ok=False);(out/'native').mkdir()
 save=lambda n,v:(out/n).write_text(json.dumps(v,indent=2)+'\n')
 editable={'index.html':'<!doctype html><html><head><meta charset="utf-8"><title>Fictional reading room</title><link rel="stylesheet" href="style.css"></head><body><main></main><script src="app.js"></script></body></html>\n','style.css':'/* Empty authored evaluation scaffold; no design solution. */\n','app.js':'// Empty authored evaluation scaffold; no implementation.\n'}
 probe={'version':'browser-probe@v1','assets':list(editable),'entry':'index.html','actions':[],'checks':[{'selector':'body','text':''}]}
 config={'goal':'Sustained fictional reading-room noticeboard; preserve current facts and both standing wants','image':a.IMAGE,'user':'501:20','browser':True,'browser_container_only':True,'editable':editable,'protected':{'browser-harness.cjs':(ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(probe)},'probe_entry':'browser-harness.cjs','cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}],'timeout_ms':30000}
 save('scope.json',config);save('requirements.json',REQUIREMENTS)
 m=a.load('pursuit_campaign','engine-b91-campaign.py');campaign=json.loads((ROOT/'target/tsr-integrated-campaign-1/manifest.json').read_text());upgrade=json.loads((ROOT/'target/relational-experience-live-v7/campaign-upgrade.json').read_text());ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));status=ledger.status()
 if status['stopped'] or status['blocking_unknown_executions']:raise ValueError('Unresolved effects')
 unlimited=status['unlimited_policy_sha256']
 if not unlimited:raise ValueError('Existing explicit unlimited policy required')
 c=a.d.Native(out/'native');c.cli('project','init','--spec',str(out/'scope.json'),'--requirements',str(out/'requirements.json'))
 c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':7200000,'reason':'Separate fictional TSR evaluation, prospective unlimited policy follows; no game polishing','non_tool':{'model_calls':200,'model_tokens':200*(a.base.capacity.INPUT_TOKENS+32768),'wall_ms':7200000,'patch_bytes':104857600,'external_cost_microusd':0}},'--admission')
 c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root']);c.cli('project','resource-unlimited-admit','--state-root',c.request()['state_root'],'--policy-sha256',unlimited)
 completion=c.cli('project','acceptance','status');c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':REQUIREMENTS})
 policy_input={'state_root':c.request()['state_root'],'requirements_revision':completion['requirements_revision'],'requirements':REQUIREMENTS,'coverage':criteria['coverage'],'prerequisites':criteria['prerequisites'],'assessment_scope':criteria['assessment_scope']}
 policy_id=c.json_cli(('project','criterion-policy'),policy_input)
 criterion_profile={'version':'public-criterion-return@v2','policy':policy_id,'policy_sha256':a.e.sha(policy_input),'imported_returns':[],'consistency_reassessment':'single-fresh@v1'}
 assert c.cli('project','criterion-context')['policy']==[policy_id,policy_input], 'published criterion policy differs from prospective input'
 save('criterion-policy.json',policy_input)
 r=c.request();coding=r['coding'];i=coding['relational']['inputs']
 public={'version':'public-browser-origin@v1','scope':r['next_attention']['scope'],'requirements_revision':i['requirements_revision'],'parents':i['parents'],'protected':{f['path']:f['source_digest'] for f in coding['files'] if not f['editable']},'editable':list(editable),'public_brief':BRIEF,'source_snapshots':[coding['snapshot']],'public_relationships':[],'public_returns':[]}
 origin={'directory':str(c.directory),'public':public,'initial_sources':editable}
 events=[{'id':'synthetic-condition-'+name,'attribution':'synthetic_harness_verification','guidance':{key:meaning+'\n\n'+facts for key,meaning in REQUIREMENTS.items()}} for name,facts in CONDITIONS]
 policy={'version':'authorized-feedback-cycle@v2','authorization_id':'Sentinel_063d4f2f4f3c81919cf5d0aa09459377','max_continuations':None}
 ad=a.admission(out/'adapter',origin,c.cli('project','config'),campaign,None,max_decisions=None,max_executions=None,correction_allowance=20,feedback_events=events,feedback_cycle=policy,transient_recovery={'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':60},independent_assessment={'version':'source-bound-browser-assessment@v1','script':str(FROZEN/'observe.cjs'),'sha256':a.d.file_sha(FROZEN/'observe.cjs'),'limitations':'Invariant source-bound browser observation only. Scenario facts are synthetic operator guidance, not observed service outcomes or player preferences. Retained source may reuse the same invariant assessment; no condition-specific test or human fulfillment is claimed.'},unlimited_policy_sha256=unlimited,criterion_return_profile=criterion_profile)
 ca={'version':'local-project-cycle@v3','journal':str(out/'cycle'),'scope':public['scope'],'max_decisions':None,'max_executions':None,'local_source_profile':'browser-scoped-source@v1','stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'feedback_cycle':policy,'expires_at':None,'unlimited_policy_sha256':unlimited,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
 authority={'version':'authorized-tsr-staged-experience@v1','user_authorization_reference':{'message_id':'Sentinel_063d4f2f4f3c81919cf5d0aa09459377','at':'2026-10-31T20:54:48Z','scope':'separate frozen control-observation TSR test on CheaperInference/Luna; no cumulative spending, call, operation or time cap'},'frozen_packet_sha256':PACKET_HASH,'scope':'One sustained fictional noticeboard evaluation over three frozen synthetic scenario conditions, actual browser observations and subsequent free choices; no changes to delivered games','budget_policy':unlimited,'baseline':status,'acceptance_authority':False}
 adapter=a.Adapter(ad['directory'],ad,c,ledger);grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:/root/contrast_bridge_review; frozen scope accepted, fresh binding review required before dispatch',None)
 preview=a.web.project(r,public);body=adapter.body('Prospective public context only; no dispatch.',preview)
 if a.base.capacity.prompt_upper(body)>a.base.capacity.INPUT_TOKENS:raise ValueError('Initial complete context too large')
 for name,v in {'adapter-admission':ad,'cycle-admission':ca,'campaign-upgrade':upgrade,'scope-authority':authority,'scope-grant':grant,'public-preview':preview,'prepared-manifest':{'adapter_sha256':a.e.sha(ad),'grant_sha256':m.digest(grant),'baseline':status,'source_pins':ad['implementation'],'binary_sha256':ad['binary_sha256'],'packet_files':{p.name:a.d.file_sha(p) for p in Path(__file__).parent.iterdir() if p.is_file()},'initial_projection_bytes':len(a.e.canonical(preview)),'dispatch_started':False,'frozen_packet_sha256':PACKET_HASH}}.items():save(name+'.json',v)
 return {'packet':str(out),'manifest_sha256':a.d.file_sha(out/'prepared-manifest.json'),'provider_calls':0,'executions':0}
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);args=p.parse_args();print(json.dumps(prepare(args.output),indent=2))
