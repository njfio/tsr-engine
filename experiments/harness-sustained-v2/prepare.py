#!/usr/bin/env python3
"""Create a fresh isolated native browser state and prospective admissions; no dispatch."""
import argparse
import datetime
import importlib.util
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('browser_packet_adapter',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
BRIEF='Synthetic harness verification, not a new player request or reported player experience. Start from the previously engine-produced fictional Lantern Maze source. Preserve its working maze/falling-block game. Exercise ordinary incremental proposals, independent review, actual browser returns and admitted feedback through two checkpoints. Do not invent observed success or force relationship novelty. Only admitted HTML/CSS/JS may change; no external resources. The fixed protected checks are minimum functional obligations, not proof of game quality: Start, click #harness-help-toggle, press h, then ArrowLeft. Afterwards [data-tsr=status] contains Playing, #harness-help-toggle contains Hide help, and #harness-help contains Controls. Preserve these exact public interface contracts.'
REQUIREMENTS={'understandable_play':'Preserve understandable playable controls and support the explicitly attributed current harness-verification guidance. Original game mechanics remain open within this goal.','engaging_challenge':'Preserve the coherent original maze/falling-block challenge. No invented user satisfaction or relationship development.'}
EVENTS=[{'id':'harness-verification-help-1','attribution':'synthetic_harness_verification','guidance':{'understandable_play':REQUIREMENTS['understandable_play']+' Synthetic verification change one: add a visible help panel #harness-help with concise current game controls, initially shown, and a button #harness-help-toggle. Its exact visible label is Hide help when shown and Show help when hidden. Clicking it and pressing H must each toggle the panel without resetting gameplay. Retain [data-tsr=start] and [data-tsr=status]. This is a functional interface check, not observed player feedback.'}}, {'id':'harness-verification-help-2','attribution':'synthetic_harness_verification','guidance':{'understandable_play':REQUIREMENTS['understandable_play']+' Synthetic verification change two: retain the help panel, toggle button labels, H shortcut and all game behavior from change one. Add an accessible live announcement of help visibility (aria-live polite), with correct visible/hidden announcements after both mouse and keyboard toggles. Do not steal focus or restart gameplay. This is a later test input, not player testimony.'}}]
def prepare(out,campaign):
    out=Path(out).resolve();out.mkdir(parents=True,exist_ok=False);(out/'native').mkdir()
    editable={name:(ROOT/'artifacts/lantern-maze-v2'/name).read_text() for name in ('index.html','app.js','style.css')}
    probe={'version':'browser-probe@v1','assets':list(editable),'entry':'index.html','actions':[{'kind':'click','value':'[data-tsr="start"]'},{'kind':'click','value':'#harness-help-toggle'},{'kind':'key','value':'h'},{'kind':'key','value':'ArrowLeft'}],'checks':[{'selector':'[data-tsr="status"]','text':'Playing'},{'selector':'#harness-help-toggle','text':'Hide help'},{'selector':'#harness-help','text':'Controls'}]}
    config={'goal':'Develop a playable browser game; user experience remains independently assessed','image':a.IMAGE,'user':'501:20','browser':True,'browser_container_only':True,'editable':editable,'protected':{'browser-harness.cjs':(ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(probe)},'probe_entry':'browser-harness.cjs','cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}],'timeout_ms':30000}
    def save(name,value):(out/name).write_text(json.dumps(value,indent=2)+'\n')
    save('scope.json',config);save('requirements.json',REQUIREMENTS)
    c=a.d.Native(out/'native');c.cli('project','init','--spec',str(out/'scope.json'),'--requirements',str(out/'requirements.json'))
    expires=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=2)).isoformat()
    c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':720000,'reason':'Prospective isolated browser session; no dispatch authority from preparation','non_tool':{'model_calls':44,'model_tokens':46200000,'wall_ms':7200000,'patch_bytes':104857600,'external_cost_microusd':0}},'--admission')
    completion=c.cli('project','acceptance','status')
    c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':REQUIREMENTS})
    c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root'])
    r=c.request();coding=r['coding'];i=coding['relational']['inputs']
    if {p['requirement']:p['meaning'] for p in i['parents']}!=REQUIREMENTS:raise ValueError('initial public wants not active')
    public={'version':'public-browser-origin@v1','scope':r['next_attention']['scope'],'requirements_revision':i['requirements_revision'],'parents':i['parents'],'protected':{f['path']:f['source_digest'] for f in coding['files'] if not f['editable']},'editable':list(editable),'public_brief':BRIEF+' Harness interface only: keep [data-tsr=start] clickable and [data-tsr=status] containing Playing once started. This does not prescribe game mechanics.','source_snapshots':[coding['snapshot']],'public_relationships':[],'public_returns':[]}
    origin={'directory':str(c.directory),'public':public,'initial_sources':editable}
    admission=a.admission(out/'adapter',origin,c.cli('project','config'),campaign,expires,max_decisions=20,max_executions=8,correction_allowance=4,feedback_events=EVENTS)
    ca={'version':'local-project-cycle@v3','journal':str(out/'cycle'),'scope':public['scope'],'max_decisions':20,'max_executions':8,'local_source_profile':'browser-scoped-source@v1','stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'expires_at':expires,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
    context=a.web.project(r,public,context_budget=a.base.capacity.PAYLOAD_BYTES-65536);body={'model':a.base.MODEL,'response_format':{'type':'json_object'},'max_completion_tokens':32768,'stream':False,'messages':[{'role':'system','content':'Prospective bounded native browser proposal; no dispatch during preparation.'},{'role':'user','content':a.e.canonical(context).decode()}]}
    if a.base.capacity.prompt_upper(body)>a.base.capacity.INPUT_TOKENS:raise ValueError('initial request does not fit')
    save('feedback-events.json',EVENTS)
    save('adapter-admission.json',admission);save('cycle-admission.json',ca);save('public-preview.json',context)
    result={'launch_enabled':False,'reason':'Requires exact prospective verification scope grant, independent review and fresh launch preflight','adapter_sha256':a.e.sha(admission),'suite_sha256':a.e.sha({'public':public,'configuration':admission['configuration_sha256']}),'initial_preview_bytes':len(a.e.canonical(body)),'calls':44,'operations':8,'corrections':4,'provider_calls':0,'target_executions':0,'expires_at':expires}
    save('prepared.json',result);return result
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--campaign-manifest',type=Path,required=True);p.add_argument('--packet-sha256',required=True);args=p.parse_args()
    if a.d.file_sha(Path(__file__).with_name('manifest.json'))!=args.packet_sha256:raise ValueError('packet identity mismatch')
    manifest=json.loads(Path(__file__).with_name('manifest.json').read_text())
    if any(a.d.file_sha(ROOT/name)!=digest for name,digest in manifest['files'].items()):raise ValueError('packet source drift')
    print(json.dumps(prepare(args.output,json.loads(args.campaign_manifest.read_text())),indent=2))
