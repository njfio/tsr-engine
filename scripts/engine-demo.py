#!/usr/bin/env python3
"""Offline recorded replay by default. Paid execution requires an explicit subcommand/flag."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
INDEX=ROOT/'docs/engine-workshop-replay-index.json'

def digest(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()

def replay(directory, scenario='workshop'):
    # Deliberately no driver/transport import, subprocess, SQLite write or recovery path.
    directory=Path(directory).absolute()
    if directory.is_symlink():raise ValueError('symlink evidence directory')
    if scenario not in ('workshop','garden','garden-trial2'):raise ValueError('unregistered replay scenario')
    index_path=INDEX if scenario=='workshop' else ROOT/f'docs/engine-{scenario}-replay-index.json'
    index=json.loads(index_path.read_text());docs={}
    for name,expected in index['files'].items():
        relative=Path(name)
        if relative.is_absolute() or '..' in relative.parts:raise ValueError('invalid evidence index')
        path=directory
        for part in relative.parts:
            path=path/part
            if path.is_symlink():raise ValueError('symlink evidence file')
        if path.stat().st_size>8*1024*1024:raise ValueError('evidence size bound')
        raw=path.read_bytes()
        if hashlib.sha256(raw).hexdigest()!=expected:raise ValueError('recorded evidence changed: '+name)
        docs[name]=json.loads(raw)
    if scenario=='garden':return replay_unknown(docs,index)
    j='synthetic-external/';terminal=docs[j+'terminal.json'];initial=docs['admission.json.source.json'];final=docs['final.json']
    if (terminal['status']!=index['expected_terminal'] or terminal['calls_dispatched']!=5
            or terminal['acknowledged_actions']!=1 or terminal['settled_usd']!=index['cost_usd']):
        raise ValueError('terminal mismatch')
    stages=[]
    for n in range(1,6):
        intent=docs[j+f'intent-{n}.json'];outcome=docs[j+f'outcome-{n}.json'];applied=docs[j+f'applied-{n}.json']
        if (outcome['intent_hash']!=digest(intent) or applied['outcome_hash']!=digest(outcome)
                or intent['payload_hash']!=digest(docs[j+f'request-{n}.json'])):raise ValueError('receipt chain mismatch')
        r=outcome['receipt'];stages.append({'stage':intent['phase'],'usd':r['usd'],
            'input_tokens':r['usage']['prompt_tokens'],'completion_tokens':r['usage']['completion_tokens'],
            'seconds':r['elapsed_seconds'],'state_after':applied['after_root']})
    if final['state_root']!=terminal['state_root'] or initial['archive']['events']!=final['archive']['events'][:4]:raise ValueError('history mismatch')
    result = {'mode':'read_only_recorded_replay','dispatches_this_invocation':0,'actions_this_invocation':0,
        'recorded_code_revision':index['recorded_code_revision'],'initial_root':initial['state_root'],
        'final_root':final['state_root'],'recorded_terminal':terminal['status'],'recorded_calls':5,'recorded_actions':1,
        'recorded_usd':terminal['settled_usd'],'manual_interventions':docs['verification.json']['manual_interventions_after_start'],
        'stages':stages,'aspiration':docs[j+'outcome-1.json']['value']['proposals'][0]['aspiration'],
        'recorded_event':final['archive']['events'][-1],
        'proposed_development':docs[j+'outcome-4.json']['value']['proposals'][0]['development']['proposed_development'],
        'unexecuted_next_proposals':len(docs[j+'outcome-5.json']['value']['directions']),
        'qualification':index.get('qualification','Bounded synthetic cycle met minimum; justified retention, not new insight. Necessity/consent wording imperfect. No causal guidance benefit, reliability or social fulfillment established.')}
    if scenario=='garden-trial2':
        historical=replay(ROOT/'target/garden-ready/run','garden')
        result['historical_unknown']={k:historical[k] for k in ('recorded_total_usd','cost_status','usage_status','retained_reservations','recorded_terminal')}
        result['combined_total_usd']=None
    return result

def replay_unknown(docs,index):
    """Pinned historical uncertainty is displayed, never reconciled or dispatched."""
    j='synthetic-external/'
    terminal=docs[j+'terminal.json'];intent=docs[j+'intent-1.json'];outcome=docs[j+'outcome-1.json']
    source=docs['admission.json.source.json'];admission=docs['admission.json']
    if (terminal['status']!=index['expected_terminal'] or terminal['calls_dispatched']!=1
            or terminal['acknowledged_actions']!=0 or terminal['unknown_usage_or_cost'] is not True
            or outcome['receipt']['usd'] is not None or outcome['value'] is not None
            or outcome['intent_hash']!=digest(intent) or intent['admission_hash']!=digest(admission)
            or intent['payload_hash']!=digest(docs[j+'request-1.json'])
            or terminal['state_root']!=source['state_root'] or terminal['state_root']!=admission['initial_root']
            or any(n.startswith(j+'apply-intent-') for n in docs)):
        raise ValueError('unknown-attempt evidence mismatch')
    return {'mode':'read_only_recorded_replay','scenario':'garden',
        'dispatches_this_invocation':0,'actions_this_invocation':0,
        'recorded_code_revision':index['recorded_code_revision'],'recorded_terminal':terminal['status'],
        'recorded_calls':1,'recorded_actions':0,'initial_root':source['state_root'],
        'final_root':terminal['state_root'],'recorded_settled_usd':terminal['settled_usd'],
        'recorded_total_usd':None,'usage_status':'unknown','cost_status':'unknown',
        'retained_reservations':terminal['retained_reservations'],
        'remaining_slots_expired':terminal['remaining_slots_expired'],
        'qualification':'No generated output or semantic result. Settled subtotal is not total cost. Unknown execution and usage are not failure or zero charge.'}


def main():
    parser=argparse.ArgumentParser(description=__doc__);sub=parser.add_subparsers(dest='command')
    p=sub.add_parser('replay');p.add_argument('--evidence',type=Path)
    p.add_argument('--scenario',choices=['workshop','garden','garden-trial2'],default='workshop')
    p=sub.add_parser('prepare');p.add_argument('--directory',type=Path,required=True);p.add_argument('--expires',required=True)
    p=sub.add_parser('run');p.add_argument('--directory',type=Path,required=True);p.add_argument('--admission',type=Path,required=True)
    p.add_argument('--allow-paid-calls',action='store_true',help='Explicitly opt in to the supplied bounded admission; never inferred from replay')
    args=parser.parse_args()
    if args.command in (None,'replay'):
        scenario=getattr(args,'scenario','workshop')
        directory=getattr(args,'evidence',None) or ROOT/('target/garden-trial2/run' if scenario=='garden-trial2' else f'target/{scenario}-ready/run')
        print(json.dumps(replay(directory,scenario),indent=2));return 0
    if args.command=='prepare':
        command=[sys.executable,str(ROOT/'scripts/engine-workshop-prepare.py'),'--directory',str(args.directory),'--expires',args.expires]
    else:
        if not args.allow_paid_calls:parser.error('run requires --allow-paid-calls and a separately authorized bounded admission')
        command=[sys.executable,str(ROOT/'scripts/engine-directions.py'),'--directory',str(args.directory),'--model','deepseek-v4.1-flash','--synthetic-admission',str(args.admission)]
    return subprocess.call(command,cwd=ROOT)

if __name__=='__main__':
    try:sys.exit(main())
    except (ValueError,OSError,KeyError) as error:
        print('Demo stopped: '+str(error),file=sys.stderr);sys.exit(2)
