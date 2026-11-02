#!/usr/bin/env python3
"""Read-only source-bound capacity plan. No provider, worker, admission or adoption."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
s=importlib.util.spec_from_file_location('b91_capacity_driver',Path(__file__).with_name('engine-b91-driver.py'))
d=importlib.util.module_from_spec(s);s.loader.exec_module(d)

def plan(admission_path):
    a=d.e.strict(d.e.bounded_file(admission_path,16384));root=Path(a['trial']).resolve()
    view=object.__new__(d.Driver);view.root=root;view.file=root/'driver.jsonl';view.admission=a;view.fixture=a['mode']=='fixture'
    _,pending,done=view.read()
    if pending or a['model']!='gpt-6-luna':raise ValueError('reconciliation/model mismatch')
    journal=view.journal();status=journal.status()
    if status['calls_consumed_or_reserved']!=4 or any(status[k] for k in ('unknown_or_failed','application_unknown','unsettled_metering','unapplied_response')):
        raise ValueError('exact settled four-request prefix required')
    for name in ('execute_patch','execute_check_after'):
        if done.get(name,{}).get('status',{}).get('execution_status')!='passed':raise ValueError('verified repair prefix required')
    c=view.controller();request=c.request();current=c.cli('project','status')
    expected=done['execute_patch']['status']
    if (current['goal_completion']['snapshot']!=expected['goal_completion']['snapshot']
        or current['latest_execution']['commitment']!=expected['latest_execution']['commitment']
        or request.get('pending') is not None):raise ValueError('verified source/action drift')
    history=request['coding']['relational']['history']
    if len(history)!=2:raise ValueError('capacity proof requires exact two-revision prefix')
    stages={}
    for stage in d.e.STAGES[-2:]:
        _,body=d.e.project(journal,c,stage,model=a['model'],payload_limit=32768)
        stages[stage]={'wire_bytes':len(json.dumps(body,allow_nan=False,ensure_ascii=False).encode()),'body_sha256':d.e.sha(body)}
    # Bound a SINGLE native-valid added proposal. Compact UTF-8 proposal <=4096.
    # Continuation wire JSON uses UTF-8 (ensure_ascii=False), so outer encoding
    # expands the compact inner JSON by at most 2x (quotes/backslashes). Unicode
    # bytes remain unchanged, including DEL; no unsupported token/byte ratio used.
    # Fixed new entry: 64-char admission, proposal key, three Boolean flags. Bindings
    # omitted by the existing projection. 512 bytes covers its escaped metadata,
    # list separator and old/new status-flag changes. All other IDs keep fixed width.
    metadata={'admission':'f'*64,'proposal':{},'withdrawn':False,'superseded':False,'current':False}
    metadata_wire=len(json.dumps(d.e.canonical(metadata).decode()).encode())+16
    if metadata_wire>512:raise ValueError('history metadata proof changed')
    upper=stages['final_proposal_only']['wire_bytes']+2*4096+512
    if upper>32768:raise ValueError('prefix exceeds finite continuation capacity; no dispatch')
    hashes={str(p):d.file_sha(p) for p in (root/'driver.jsonl',root/'provider.jsonl',root/'origin.json',root/'STOP')}
    return {'version':'b91-continuation-capacity@v1','dispatch_enabled':False,'admission_granted':False,
        'parent':str(root),'parent_admission_sha256':d.e.sha(a),'parent_files':hashes,
        'implementation':{**d.implementation(),Path(__file__).name:d.file_sha(Path(__file__))},
        'binary_sha256':d.file_sha(d.ROOT/'target/debug/tsr'),
        'state_root':request['state_root'],'snapshot':current['goal_completion']['snapshot'],
        'requirements_revision':current['goal_completion']['requirements_revision'],
        'model':'gpt-6-luna','stages':stages,'proof':{'added_proposals':1,'max_proposal_utf8':4096,
            'json_expansion_factor':2,'fixed_metadata_reserve':512,'measured_metadata_bound':metadata_wire,
            'final_wire_upper_bound':upper,'headroom_at_upper_bound':32768-upper,
            'preconditions':'Exact prefix/source/criteria; no executions, source edits or extra revisions. Actual assembly rechecked before each dispatch.'},
        'proposed_limits':{'calls':2,'execution_groups':0,'payload_bytes_per_call':32768,
            'input_tokens_per_call':32768,'completion_tokens_per_call':8192,'seconds_per_call':240,
            'model_tokens_total':81920,'patch_bytes':0},
        'restart':'Fresh continuation admission and separate prefix-bound ledger required; never remove STOP or reopen expired Trial11 grants.'}

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--parent-admission',type=Path,required=True);p.add_argument('--out',type=Path)
    args=p.parse_args();value=plan(args.parent_admission)
    if args.out:d.e.private_save(args.out,value)
    print(json.dumps(value,indent=2))
