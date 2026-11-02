"""Prospective initial pair preparation; no provider calls or campaign grant application."""
import copy,importlib.util,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('game_pair_adapter',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
FROZEN=ROOT/'experiments/game-wants-v4';PACKET='c834d838604b65e33a708775ddde1da421e46c77de476c0d74567d951cf2afc0'
AUTH={'message_id':'Sentinel_9881af6eb4f0819184c6b1e4ed55756b','at':'2026-11-01T18:04:11Z','scope':'ONE paired TSR and ordinary-memory comparison on CheaperInference/gpt-6-luna; no cumulative spending, call or time cap; original prompt and frozen partial observation policy; operator mapping disclosed; no subsequent experiments'}
def prepare(out):
 assert a.d.file_sha(FROZEN/'manifest.json')==PACKET
 frozen=json.loads((FROZEN/'manifest.json').read_text())
 for path,expected in frozen['files'].items():assert a.d.file_sha(ROOT/path)==expected,path
 out=Path(out).resolve();out.mkdir(exist_ok=False);save=lambda path,value:path.write_text(json.dumps(value,indent=2)+'\n')
 protocol=json.loads((FROZEN/'protocol.json').read_text())
 facts={k:copy.deepcopy(protocol[k]) for k in ('operational_roots','criteria','minimum_obligations','feedback','unknown_rule','shared_consequences','excluded_claims','observation_policy')}
 brief=protocol['user_input']['text']+'\n\nExperiment-authored operational context, not additional player preferences:\n'+json.dumps(facts,ensure_ascii=False,sort_keys=True)
 wants={k:v+'\nArtifact criterion: '+protocol['criteria'][k] for k,v in protocol['operational_roots'].items() if k!='attribution'}
 editable={'index.html':'<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><link rel="stylesheet" href="style.css"></head><body><main></main><script src="app.js"></script></body></html>\n','app.js':'// Empty project scaffold; no game implementation.\n','style.css':'/* Empty project scaffold. */\n'}
 probe={'version':'browser-probe@v1','assets':list(editable),'entry':'index.html','actions':[],'checks':[{'selector':'body','text':''}]}
 config={'goal':'Browser project under the published crossing and growth roots','image':a.IMAGE,'user':'501:20','browser':True,'browser_container_only':True,'editable':editable,'protected':{'browser-harness.cjs':(ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(probe)},'probe_entry':'browser-harness.cjs','cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}],'timeout_ms':30000}
 m=a.load('game_pair_campaign','engine-b91-campaign.py');campaign=json.loads((ROOT/'target/tsr-integrated-campaign-1/manifest.json').read_text());rows=[m.strict(line) for line in Path(campaign['ledger_path']).read_bytes().splitlines()]
 upgrade={'version':'b91-campaign-upgrade@v1','old_manifest_sha256':m.digest(campaign),'old_implementation_sha256':campaign['implementation_sha256'],'new_implementation_sha256':a.d.file_sha(ROOT/'scripts/engine-b91-campaign.py'),'ledger_prefix_head':rows[-1]['hash'],'reviewer':'independent-agent:/root/binding_review','reason':'Prospective separately approved game-wants paired evaluation; historical accounting and quarantined uncertainty retained.'}
 ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));status=ledger.status()
 if status['stopped'] or status['blocking_unknown_calls'] or status['blocking_unknown_executions']:raise ValueError('unresolved external effects')
 unlimited=status['unlimited_policy_sha256'];assert unlimited
 save(out/'campaign-upgrade.json',upgrade);save(out/'baseline.json',status);save(out/'common-information.json',{'user_input':protocol['user_input'],'facts':facts,'wants':wants,'source':editable,'model':protocol['model']})
 for label,ordinary in [('tsr',False),('ordinary',True)]:
  arm=out/label;arm.mkdir();(arm/'native').mkdir();save(arm/'scope.json',config);save(arm/'requirements.json',wants)
  c=a.d.Native(arm/'native');c.cli('project','init','--spec',str(arm/'scope.json'),'--requirements',str(arm/'requirements.json'))
  c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':7200000,'reason':'Prospective paired fictional browser project; existing unlimited policy applied before dispatch','non_tool':{'model_calls':200,'model_tokens':200*(a.base.capacity.INPUT_TOKENS+32768),'wall_ms':7200000,'patch_bytes':104857600,'external_cost_microusd':0}},'--admission')
  c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root']);c.cli('project','resource-unlimited-admit','--state-root',c.request()['state_root'],'--policy-sha256',unlimited)
  st=c.cli('project','acceptance','status');c.json_cli(('project','acceptance','feedback'),{'requirements_revision':st['requirements_revision'],'decision_record':st['decision_record'],'guidance':wants})
  r=c.request();coding=r['coding'];inputs=coding['relational']['inputs']
  public={'version':'public-browser-origin@v1','scope':r['next_attention']['scope'],'requirements_revision':inputs['requirements_revision'],'parents':inputs['parents'],'protected':{f['path']:f['source_digest'] for f in coding['files'] if not f['editable']},'editable':list(editable),'public_brief':brief,'source_snapshots':[coding['snapshot']],'public_relationships':[],'public_returns':[]}
  origin={'directory':str(c.directory),'public':public,'initial_sources':editable}
  policy={'version':'authorized-feedback-cycle@v2','authorization_id':AUTH['message_id'],'max_continuations':None}
  ad=a.admission(arm/'adapter',origin,c.cli('project','config'),campaign,None,max_decisions=None,max_executions=None,correction_allowance=20,feedback_cycle=policy,transient_recovery={'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':60},process_profile='ordinary-memory@v1' if ordinary else None,unlimited_policy_sha256=unlimited)
  ca={'version':'local-project-cycle@v3','journal':str(arm/'cycle'),'scope':public['scope'],'max_decisions':None,'max_executions':None,'local_source_profile':'browser-scoped-source@v1','stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'feedback_cycle':policy,'expires_at':None,'unlimited_policy_sha256':unlimited,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
  typ=a.load('game_plain','engine-browser-ordinary.py').Adapter if ordinary else a.Adapter;adapter=typ(ad['directory'],ad,c,ledger)
  authority={'version':'game-wants-pair-scope@v1','authorization':AUTH,'packet_sha256':PACKET,'common_information_sha256':a.d.file_sha(out/'common-information.json'),'arm':label,'stage':'initial build checkpoint; no observation claims','baseline':status,'paid_dispatch':False,'later_stages':'Three frozen observation batches within this same pair; each uses fresh reviewed source binding, never reopens a terminal journal or resets usage.'}
  grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:/root/binding_review; fresh binding review required before dispatch',None)
  view=c.cli('project','cycle','request');req,context=adapter.project(view);wire=adapter.wire_context(req,context);body=adapter.proposal_body(view,wire)
  assert a.base.capacity.prompt_upper(body)<=a.base.capacity.INPUT_TOKENS
  for name,value in {'adapter-admission':ad,'cycle-admission':ca,'scope-authority':authority,'scope-grant':grant,'prospective-request':body}.items():save(arm/(name+'.json'),value)
  save(arm/'prepared-manifest.json',{'admission_sha256':a.e.sha(ad),'grant_sha256':m.digest(grant),'binary_sha256':ad['binary_sha256'],'implementation':ad['implementation'],'common_information_sha256':a.d.file_sha(out/'common-information.json'),'model':protocol['model'],'source_snapshot':coding['snapshot'],'original_parents':inputs['parents'],'provider_calls':0,'paid_dispatch':False})
 # Both stages are prepared before either output exists. B's grant must be
 # prospectively refreshed against the actual ledger prefix before B dispatch.
 save(out/'prepared-pair.json',{'authorization':AUTH,'packet_sha256':PACKET,'baseline':status,'common_information_sha256':a.d.file_sha(out/'common-information.json'),'arms':{k:a.d.file_sha(out/k/'prepared-manifest.json') for k in ('tsr','ordinary')},'paid_dispatch':False,'source_commit':'8c5b1df0'})
 return {'path':str(out),'pair_sha256':a.d.file_sha(out/'prepared-pair.json'),'baseline':status,'provider_calls':0}
if __name__=='__main__':
 import sys
 print(json.dumps(prepare(sys.argv[1]),indent=2))
