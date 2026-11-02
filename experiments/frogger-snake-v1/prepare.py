#!/usr/bin/env python3
"""Create a fresh isolated native browser state and prospective admissions; no dispatch."""
import argparse
import datetime
import importlib.util
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('browser_packet_adapter',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
BRIEF="User request: 'ok, do a new game, frogger and snake please'. Build a new original local browser game combining Frogger-style crossing hazards with Snake-style growth/body navigation as a coherent game. Choose the mechanics, interactions, presentation, title and controls yourself. This is an open design brief, not a desired solution or observed player feedback. Make it playable and understandable; support desktop and mobile-friendly use where sensible. Original simple visuals only, no fetched assets, packages or network. Preserve the contributing wants and later experience in this separate project; do not import Lantern Maze code or user-satisfaction claims. You may form a useful relationship, edit incrementally, retain an approach or abstain; no forced novelty or relationship is required. Protected smoke checks are minimum interface obligations, not proof of full gameplay."
REQUIREMENTS={'understandable_play':'Provide a playable original browser game with visible understandable controls, a way to start and restart, and legible outcomes. Preserve genuine future user feedback and history. Do not invent satisfaction.', 'integrated_challenge':'Explore a coherent playable fusion of crossing hazards and a growing snake body. How these interact, the design tradeoffs, progression and visuals are open to the engine. Preserve both contributing intentions; no prescribed joint solution or forced relationship revision.'}

def prepare(out,campaign):
    out=Path(out).resolve();out.mkdir(parents=True,exist_ok=False);(out/'native').mkdir()
    editable={'index.html':'<!doctype html><html><head><meta charset="utf-8"><title>Game project</title><link rel="stylesheet" href="style.css"></head><body><main><button data-tsr="start">Start</button><p data-tsr="status">Not started</p><canvas width="640" height="480"></canvas></main><script src="app.js"></script></body></html>\n','style.css':'/* Open design space. No game implementation supplied. */\n','app.js':'// No game implementation supplied.\n'}
    probe={'version':'browser-probe@v1','assets':list(editable),'entry':'index.html','actions':[{'kind':'click','value':'[data-tsr="start"]'},{'kind':'key','value':'ArrowLeft'},{'kind':'key','value':'Space'},{'kind':'wait','value':250}],'checks':[{'selector':'[data-tsr="status"]','text':'Playing'}]}
    config={'goal':'Develop a playable browser game; user experience remains independently assessed','image':a.IMAGE,'user':'501:20','browser':True,'browser_container_only':True,'editable':editable,'protected':{'browser-harness.cjs':(ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(probe)},'probe_entry':'browser-harness.cjs','cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}],'timeout_ms':30000}
    def save(name,value):(out/name).write_text(json.dumps(value,indent=2)+'\n')
    save('scope.json',config);save('requirements.json',REQUIREMENTS)
    c=a.d.Native(out/'native');c.cli('project','init','--spec',str(out/'scope.json'),'--requirements',str(out/'requirements.json'))
    expires=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=2)).isoformat()
    c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':7200000,'reason':'Prospective isolated browser session; no dispatch authority from preparation','non_tool':{'model_calls':200,'model_tokens':200*(a.base.capacity.INPUT_TOKENS+32768),'wall_ms':7200000,'patch_bytes':104857600,'external_cost_microusd':0}},'--admission')
    completion=c.cli('project','acceptance','status')
    c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':REQUIREMENTS})
    c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root'])
    r=c.request();coding=r['coding'];i=coding['relational']['inputs']
    if {p['requirement']:p['meaning'] for p in i['parents']}!=REQUIREMENTS:raise ValueError('initial public wants not active')
    public={'version':'public-browser-origin@v1','scope':r['next_attention']['scope'],'requirements_revision':i['requirements_revision'],'parents':i['parents'],'protected':{f['path']:f['source_digest'] for f in coding['files'] if not f['editable']},'editable':list(editable),'public_brief':BRIEF+' Harness interface only: keep [data-tsr=start] clickable and [data-tsr=status] containing Playing once started. This does not prescribe game mechanics.','source_snapshots':[coding['snapshot']],'public_relationships':[],'public_returns':[]}
    origin={'directory':str(c.directory),'public':public,'initial_sources':editable}
    policy={'version':'authorized-feedback-cycle@v1','authorization_id':'Sentinel_61f54657df5481919087d5287e34f738','max_continuations':4}
    admission=a.admission(out/'adapter',origin,c.cli('project','config'),campaign,expires,max_decisions=90,max_executions=100,correction_allowance=20,feedback_cycle=policy,transient_recovery={'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':120})
    ca={'version':'local-project-cycle@v3','journal':str(out/'cycle'),'scope':public['scope'],'max_decisions':90,'max_executions':100,'local_source_profile':'browser-scoped-source@v1','stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'feedback_cycle':policy,'expires_at':expires,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
    context=a.web.project(r,public,context_budget=a.base.capacity.PAYLOAD_BYTES-65536);body={'model':a.base.MODEL,'response_format':{'type':'json_object'},'max_completion_tokens':32768,'stream':False,'messages':[{'role':'system','content':'Prospective bounded native browser proposal; no dispatch during preparation.'},{'role':'user','content':a.e.canonical(context).decode()}]}
    if a.base.capacity.prompt_upper(body)>a.base.capacity.INPUT_TOKENS:raise ValueError('initial request does not fit')
    save('adapter-admission.json',admission);save('cycle-admission.json',ca);save('public-preview.json',context)
    m=a.load('campaign','engine-b91-campaign.py');old_upgrade=json.loads((ROOT/'target/harness-feedback-cycle-live-v2-repaired-4/campaign-upgrade.json').read_text())
    rows=[json.loads(line) for line in Path(campaign['ledger_path']).read_text().splitlines() if line]
    upgrade={**old_upgrade,'new_implementation_sha256':a.d.file_sha(Path(m.__file__)),'ledger_prefix_head':rows[-1]['hash'],'reviewer':'independent-agent:/root/handoff_audit','reason':'New explicitly requested original Frogger–Snake game; preserve settled prior accounting and admit full finite session capacity.'}
    ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));status=ledger.status()
    assert (status['calls'],status['execution_groups'],status['known_cost_microusd'])==(315,84,327385) and not status['unknown_calls'] and not status['unknown_executions']
    authority={'version':'authorized-browser-game-session@v1','user_message':'Sentinel_61f54657df5481919087d5287e34f738','scope':'new original Frogger–Snake-inspired fictional browser game, engine-authored; isolated candidate, independent controls testing, no publishing or host browser','max_calls':200,'max_operations':100,'max_microusd':10000000,'expires_at':expires,'provider_failure_retries':1,'backoff_seconds':120,'final_user_acceptance':False,'baseline':status}
    adapter=a.Adapter(admission['directory'],admission,c,ledger);grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:/root/handoff_audit',10000000,extend_authority=True)
    save('campaign-upgrade.json',upgrade);save('scope-authority.json',authority);save('scope-grant.json',grant)
    result={'launch_enabled':False,'reason':'Requires independent packet review and fresh launch preflight','adapter_sha256':a.e.sha(admission),'grant_sha256':m.digest(grant),'suite_sha256':a.e.sha({'public':public,'configuration':admission['configuration_sha256']}),'initial_preview_bytes':len(a.e.canonical(body)),'calls':200,'operations':100,'corrections':20,'provider_calls':0,'target_executions':0,'expires_at':expires,'baseline':status,'source_pins':admission['implementation'],'binary_sha256':admission['binary_sha256'],'packet_files':{str(p.relative_to(ROOT)):a.d.file_sha(p) for p in (Path(__file__),Path(__file__).with_name('README.md'))}}
    save('prepared-manifest.json',result);result['manifest_sha256']=a.d.file_sha(out/'prepared-manifest.json');return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--campaign-manifest',type=Path,required=True);args=p.parse_args()
    print(json.dumps(prepare(args.output,json.loads(args.campaign_manifest.read_text())),indent=2))
