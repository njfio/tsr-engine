"""One launch, no stages; no provider calls here. Usage: TSR_LIVE_LABEL=<label> TSR_SPEND_CAP_MICROUSD=<cap|none> TSR_LIVE_COMMIT=<sha> prepare.py <authorization.json>."""
import copy,importlib.util,json,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('game_pair_adapter',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
LABEL=os.environ.get('TSR_LIVE_LABEL','r1');CAP=None if os.environ['TSR_SPEND_CAP_MICROUSD']=='none' else int(os.environ['TSR_SPEND_CAP_MICROUSD']);PROTOCOL=ROOT/'experiments/continuous-game-v4/protocol.json';PACKET=a.d.file_sha(PROTOCOL)
FILES=['experiments/want-loop-v1/'+n for n in ('run.py','prepare.py','report.py')]+['experiments/continuous-game-v4/protocol.json','scripts/engine-abilities.py','scripts/engine-reading.py','scripts/engine-browser-harness.cjs']
FACT_KEYS=('operational_roots','criteria','minimum_obligations','unknown_rule','shared_consequences','excluded_claims')
def prepare(auth_path):
 AUTH=json.loads(Path(auth_path).read_text());assert set(AUTH)=={'approved','at','text','scope'} and type(AUTH['approved']) is bool
 out=ROOT/('target/want-loop-'+LABEL);out.mkdir(exist_ok=False);save=lambda path,value:path.write_text(json.dumps(value,indent=2)+'\n')
 save(out/'authorization.json',AUTH)
 protocol=json.loads(PROTOCOL.read_text());assert a.d.file_sha(ROOT/protocol['derived_from']['path'])==protocol['derived_from']['sha256']
 # The protocol's feedback and observation_policy describe operator-run batches. No operator runs anything here, so the brief doesn't claim one does.
 facts={k:copy.deepcopy(protocol[k]) for k in FACT_KEYS}
 facts['excluded_claims']=[x.replace('from one pair','from one run') for x in facts['excluded_claims']]
 brief=protocol['user_input']['text']+'\n\nExperiment-authored operational context, not additional player preferences:\n'+json.dumps(facts,ensure_ascii=False,sort_keys=True)
 wants={k:v+'\nArtifact criterion: '+protocol['criteria'][k] for k,v in protocol['operational_roots'].items() if k!='attribution'}
 editable={'index.html':'<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><link rel="stylesheet" href="style.css"></head><body><main></main><script src="app.js"></script></body></html>\n','app.js':'// Empty project scaffold; no game implementation.\n','style.css':'/* Empty project scaffold. */\n'}
 probe={'version':'browser-probe@v1','assets':list(editable),'entry':'index.html','actions':[],'checks':[{'selector':'body','text':''}]}
 config={'goal':'Browser project under the published crossing and growth roots','image':a.IMAGE,'user':'501:20','browser':True,'browser_container_only':True,'editable':editable,'protected':{'browser-harness.cjs':(ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(probe)},'probe_entry':'browser-harness.cjs','cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}],'timeout_ms':30000}
 m=a.load('continuous_campaign','engine-b91-campaign.py');campaign_root=ROOT/('target/want-loop-campaign-'+LABEL);campaign_root.mkdir(exist_ok=False)
 campaign=m.manifest(a.e.sha({'authorization':AUTH,'packet':PACKET}),campaign_root/'ledger.jsonl');save(campaign_root/'manifest.json',campaign)
 ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),create=True)
 rows=[m.strict(line) for line in Path(campaign['ledger_path']).read_bytes().splitlines()]
 policy={'version':'browser-unlimited-policy@v1','manifest_sha256':m.digest(campaign),'ledger_prefix_head':rows[-1]['hash'],'provider':'CheaperInference','model':'gpt-6-luna','source_class':'newly-authored-nonprivate-browser-project','authority':m.UNLIMITED_AUTHORITY,'quarantined_unknown_keys':[]}
 ledger.enable_unlimited(policy,m.digest(policy))
 rows=[m.strict(line) for line in Path(campaign['ledger_path']).read_bytes().splitlines()]
 upgrade={'version':'b91-campaign-upgrade@v1','old_manifest_sha256':m.digest(campaign),'old_implementation_sha256':campaign['implementation_sha256'],'new_implementation_sha256':campaign['implementation_sha256'],'ledger_prefix_head':rows[-1]['hash'],'reviewer':'independent-agent:want-loop-v1 binding review; required before dispatch','reason':'Same-implementation launch binding required by existing CLI; no code upgrade and no old ledger mutation.'}
 ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));ledger.attach_upgrade();status=ledger.status()
 old=ROOT.parent/'tsr-engine-m3/target/tsr-integrated-campaign-1/ledger.jsonl'
 save(out/'prior-accounting.json',{'ledger_path':str(old),'ledger_sha256':a.d.file_sha(old),'known_calls':537,'known_groups':200,'known_cost_microusd':1113038,'historical_uncertain_estimate_microusd':50000,'note':'Preserved older campaign; not reset, reconciled or charged into this separate test ledger.'})
 previous=ROOT.parent/'tsr-continuous-development/target/continuous-game-campaign-v2/ledger.jsonl'
 save(out/'previous-test-accounting.json',{'path':str(previous),'sha256':a.d.file_sha(previous),'note':'Continuous-game v2 ledger; separate test, not reset or charged here.'})
 unlimited=status['unlimited_policy_sha256'];assert unlimited
 save(out/'campaign-upgrade.json',upgrade);save(out/'baseline.json',status);save(out/'common-information.json',{'user_input':protocol['user_input'],'facts':facts,'wants':wants,'source':editable,'model':protocol['model']})
 for label,ordinary in [('tsr',False)]:
  arm=out/label;arm.mkdir();(arm/'native').mkdir();save(arm/'scope.json',config);save(arm/'requirements.json',wants)
  c=a.d.Native(arm/'native');c.cli('project','init','--spec',str(arm/'scope.json'),'--requirements',str(arm/'requirements.json'))
  c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':7200000,'reason':'Prospective single corrected-engine fictional browser project; existing unlimited policy applied before dispatch','non_tool':{'model_calls':200,'model_tokens':200*(a.base.capacity.INPUT_TOKENS+32768),'wall_ms':7200000,'patch_bytes':104857600,'external_cost_microusd':0}},'--admission')
  c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root']);c.cli('project','resource-unlimited-admit','--state-root',c.request()['state_root'],'--policy-sha256',unlimited)
  st=c.cli('project','acceptance','status');c.json_cli(('project','acceptance','feedback'),{'requirements_revision':st['requirements_revision'],'decision_record':st['decision_record'],'guidance':wants})
  r=c.request();coding=r['coding'];inputs=coding['relational']['inputs']
  public={'version':'public-browser-origin@v1','scope':r['next_attention']['scope'],'requirements_revision':inputs['requirements_revision'],'parents':inputs['parents'],'protected':{f['path']:f['source_digest'] for f in coding['files'] if not f['editable']},'editable':list(editable),'public_brief':brief,'source_snapshots':[coding['snapshot']],'public_relationships':[],'public_returns':[]}
  origin={'directory':str(c.directory),'public':public,'initial_sources':editable}
  policy={'version':'authorized-feedback-cycle@v2','authorization_id':a.e.sha(AUTH),'max_continuations':None}
  ad=a.admission(arm/'adapter',origin,c.cli('project','config'),campaign,None,max_decisions=None,max_executions=None,correction_allowance=20,feedback_cycle=policy,transient_recovery={'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':60},process_profile='ordinary-memory@v1' if ordinary else None,unlimited_policy_sha256=unlimited,max_review_rejections=12,correction_replay=True,reading_profile=a.reading_policy.PROFILE,abilities_profile=a.abilities_policy.PROFILE,spend_cap_microusd=CAP)
  ca={'version':'local-project-cycle@v3','journal':str(arm/'cycle'),'scope':public['scope'],'max_decisions':None,'max_executions':None,'local_source_profile':'browser-scoped-source@v1','stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':12,'feedback_cycle':policy,'expires_at':None,'unlimited_policy_sha256':unlimited,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
  typ=a.load('game_plain','engine-browser-ordinary.py').Adapter if ordinary else a.Adapter;adapter=typ(ad['directory'],ad,c,ledger)
  authority={'version':'want-loop-scope@v1','authorization':AUTH,'packet_sha256':PACKET,'common_information_sha256':a.d.file_sha(out/'common-information.json'),'arm':label,'stage':'one launch; runs until it stops by itself','spend_cap_microusd':CAP,'baseline':status,'paid_dispatch_at_preparation':False,'later_stages':None}
  grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:want-loop-v1 binding review; required before dispatch',None)
  view=c.cli('project','cycle','request');req,context=adapter.project(view);wire=adapter.wire_context(req,context);body=adapter.proposal_body(view,wire)
  assert a.base.capacity.prompt_upper(body)<=a.base.capacity.INPUT_TOKENS
  for name,value in {'adapter-admission':ad,'cycle-admission':ca,'scope-authority':authority,'scope-grant':grant,'prospective-request':body}.items():save(arm/(name+'.json'),value)
  save(arm/'launch-manifest.json',{'files':{n:a.d.file_sha(ROOT/n) for n in FILES}})
  save(arm/'prepared-manifest.json',{'admission_sha256':a.e.sha(ad),'grant_sha256':m.digest(grant),'binary_sha256':ad['binary_sha256'],'implementation':ad['implementation'],'common_information_sha256':a.d.file_sha(out/'common-information.json'),'model':protocol['model'],'source_snapshot':coding['snapshot'],'original_parents':inputs['parents'],'provider_calls':0,'paid_dispatch':False})
 # One launch. There are no later stages to prepare.
 save(out/'prepared-test.json',{'authorization':AUTH,'packet_sha256':PACKET,'baseline':status,'common_information_sha256':a.d.file_sha(out/'common-information.json'),'arms':{k:a.d.file_sha(out/k/'prepared-manifest.json') for k in ('tsr',)},'paid_dispatch':False,'source_commit':os.environ['TSR_LIVE_COMMIT']})
 return {'path':str(out),'pair_sha256':a.d.file_sha(out/'prepared-test.json'),'baseline':status,'provider_calls':0}
if __name__=='__main__':
 import sys
 print(json.dumps(prepare(sys.argv[1]),indent=2))
