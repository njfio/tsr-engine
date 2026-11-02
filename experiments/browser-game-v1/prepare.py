#!/usr/bin/env python3
"""Create a fresh isolated native browser state and prospective admissions; no dispatch."""
import argparse
import datetime
import importlib.util
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('browser_packet_adapter',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
BRIEF='Operator-authored initialization of the user-requested game direction, not observed player feedback. Build a playable local browser game inspired by maze chase, falling blocks, or a thoughtful combination. Decide the design rather than copying an expected solution. Aim for understandable controls and an engaging challenge. The user will play it and request changes later. Preserve those contributing wants and experience. No external resources, packages, downloads or network. Only the three admitted files may change. The protected probe is a limited smoke check, not proof of gameplay quality or player satisfaction.'
REQUIREMENTS={'understandable_play':'Provide a playable game with visible controls and a clear way to start or restart. Preserve the user ability to understand and direct later changes.','engaging_challenge':'Offer a coherent challenge with meaningful player control and understandable outcomes. Design choices remain open; do not invent player feedback.'}
def prepare(out,campaign):
    out=Path(out).resolve();out.mkdir(parents=True,exist_ok=False);(out/'native').mkdir()
    editable={'index.html':'<!doctype html><html><head><meta charset="utf-8"><title>Game project</title><link rel="stylesheet" href="style.css"></head><body><main><button data-tsr="start">Start</button><p data-tsr="status">Not started</p><canvas width="640" height="480"></canvas></main><script src="app.js"></script></body></html>\n','style.css':'/* Open design space. No game implementation supplied. */\n','app.js':'// No game implementation supplied.\n'}
    probe={'version':'browser-probe@v1','assets':list(editable),'entry':'index.html','actions':[{'kind':'click','value':'[data-tsr="start"]'},{'kind':'key','value':'ArrowLeft'},{'kind':'key','value':'Space'},{'kind':'wait','value':250}],'checks':[{'selector':'[data-tsr="status"]','text':'Playing'}]}
    config={'goal':'Develop a playable browser game; user experience remains independently assessed','image':a.IMAGE,'user':'501:20','browser':True,'browser_container_only':True,'editable':editable,'protected':{'browser-harness.cjs':(ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(probe)},'probe_entry':'browser-harness.cjs','cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}],'timeout_ms':30000}
    def save(name,value):(out/name).write_text(json.dumps(value,indent=2)+'\n')
    save('scope.json',config);save('requirements.json',REQUIREMENTS)
    c=a.d.Native(out/'native');c.cli('project','init','--spec',str(out/'scope.json'),'--requirements',str(out/'requirements.json'))
    expires=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=2)).isoformat()
    c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':240000,'reason':'Prospective isolated browser session; no dispatch authority from preparation','non_tool':{'model_calls':10,'model_tokens':409600,'wall_ms':7200000,'patch_bytes':8192,'external_cost_microusd':0}},'--admission')
    completion=c.cli('project','acceptance','status')
    c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':REQUIREMENTS})
    c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root'])
    r=c.request();coding=r['coding'];i=coding['relational']['inputs']
    if {p['requirement']:p['meaning'] for p in i['parents']}!=REQUIREMENTS:raise ValueError('initial public wants not active')
    public={'version':'public-browser-origin@v1','scope':r['next_attention']['scope'],'requirements_revision':i['requirements_revision'],'parents':i['parents'],'protected':{f['path']:f['source_digest'] for f in coding['files'] if not f['editable']},'editable':list(editable),'public_brief':BRIEF+' Harness interface only: keep [data-tsr=start] clickable and [data-tsr=status] containing Playing once started. This does not prescribe game mechanics.','source_snapshots':[coding['snapshot']],'public_relationships':[],'public_returns':[]}
    origin={'directory':str(c.directory),'public':public,'initial_sources':editable}
    admission=a.admission(out/'adapter',origin,c.cli('project','config'),campaign,expires,max_decisions=4,max_executions=2,correction_allowance=2)
    ca={'version':'local-project-cycle@v3','journal':str(out/'cycle'),'scope':public['scope'],'max_decisions':4,'max_executions':2,'expires_at':expires,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
    context=a.web.project(r,public);body={'model':a.base.MODEL,'response_format':{'type':'json_object'},'max_completion_tokens':8192,'stream':False,'messages':[{'role':'system','content':'Prospective bounded native browser proposal; no dispatch during preparation.'},{'role':'user','content':a.e.canonical(context).decode()}]}
    if len(a.e.canonical(body))>32768:raise ValueError('initial request does not fit')
    save('adapter-admission.json',admission);save('cycle-admission.json',ca);save('public-preview.json',context)
    result={'launch_enabled':False,'reason':'Requires separate explicit game scope approval, fresh ledger-prefix grant and launch preflight','adapter_sha256':a.e.sha(admission),'suite_sha256':a.e.sha({'public':public,'configuration':admission['configuration_sha256']}),'initial_preview_bytes':len(a.e.canonical(body)),'calls':10,'operations':2,'corrections':2,'provider_calls':0,'target_executions':0,'expires_at':expires}
    save('prepared.json',result);return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--campaign-manifest',type=Path,required=True);p.add_argument('--packet-sha256',required=True);args=p.parse_args()
    if a.d.file_sha(Path(__file__).with_name('manifest.json'))!=args.packet_sha256:raise ValueError('packet identity mismatch')
    manifest=json.loads(Path(__file__).with_name('manifest.json').read_text())
    if any(a.d.file_sha(ROOT/name)!=digest for name,digest in manifest['files'].items()):raise ValueError('packet source drift')
    print(json.dumps(prepare(args.output,json.loads(args.campaign_manifest.read_text())),indent=2))
