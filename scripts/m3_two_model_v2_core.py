#!/usr/bin/env python3
"""Scoped production adapter for the authorized synthetic integration stage. No retries."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[1]
def load(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/(name+'.py'))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

edit=load('m3-edit-v3-broker')
budget=load('m3-two-model-budget')
MANIFEST=ROOT/'docs/m3-two-model-v2-freeze.json'
budget.BASELINE=ROOT/'docs/m3-two-model-baseline.json'
edit.transport.smoke_budget=budget.locked_check


def allowed_path(request):
    fields={'protocol','id','task','state_root','state_version','remaining_tokens','remaining_action_calls','progress','deadline_ms','observation','allowed_test_ids','required_reads','failing_tests','failing_test_id'}
    if set(request)!=fields:raise edit.transport.BrokerFailure('integration_scope')
    if type(request['remaining_action_calls']) is not int or not 1<=request['remaining_action_calls']<=20:
        raise edit.transport.BrokerFailure('progress_scope')
    progress=request['progress']
    if not isinstance(progress,dict) or set(progress)!={'source_state','inspection_current','test_current','submission'} or not all(type(progress[k]) is bool for k in ('inspection_current','test_current')):
        raise edit.transport.BrokerFailure('progress_scope')
    m=json.loads(MANIFEST.read_text())
    expected={a['task']:(a['file'],a['test']) for a in m['arms']}.get(request.get('task'))
    if expected is None or request['required_reads']!=[expected[0]] or request['failing_tests']!=[expected[1]] or request['allowed_test_ids']!=['named-failing-test'] or request['failing_test_id']!='named-failing-test':
        raise edit.transport.BrokerFailure('integration_scope')
    return expected[0]

edit.transport.allowed_path=allowed_path

def main(index, entrypoint):
    parser=argparse.ArgumentParser();parser.add_argument('--ledger',required=True);parser.add_argument('--approved-live',action='store_true');args=parser.parse_args()
    try:
        if not args.approved_live:raise ValueError('authorization missing')
        m=json.loads(MANIFEST.read_text())
        if Path(args.ledger)!=ROOT/m['ledger']:raise ValueError('ledger mismatch')
        arm=m['arms'][index]
        if Path(entrypoint).resolve()!=ROOT/arm['broker']:raise ValueError('entrypoint mismatch')
        if arm['index']!=index or arm['model'] not in ('deepseek-v4.1-flash','glm-5.3-flash'):raise ValueError('model scope')
        edit.transport.MODEL=arm['model']
        raw=sys.stdin.buffer.read(edit.transport.MAX_INPUT+1)
        if len(raw)>edit.transport.MAX_INPUT:raise ValueError('request limit')
        request=json.loads(raw);edit.encode(request)
        print(json.dumps(edit.transport.complete_with_diagnostics(request,args.ledger)))
    except Exception:
        print('broker unavailable; see content-free diagnostics and settlement ledger',file=sys.stderr);sys.exit(2)
