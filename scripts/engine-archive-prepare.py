#!/usr/bin/env python3
"""Prepare the initial synthetic-only possibility request. No network/credentials."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
FROZEN = '612f479aea815883e97d9b3de772ab3ffeb5056114d068770b94d648b565b200'
spec = importlib.util.spec_from_file_location('directions', ROOT/'scripts/engine-directions.py')
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)


def prepare(state):
    raw = (ROOT/'docs/engine-synthetic-archive-inputs.json').read_bytes()
    if hashlib.sha256(raw).hexdigest() != FROZEN:
        raise ValueError('frozen synthetic input drift')
    frozen = json.loads(raw)
    seed = [{'kind':'register','challenge':'access','revision':'hours-v1'}]
    if state['archive']['world'] != frozen['spec']['archive'] or state['archive']['events'] != seed:
        raise ValueError('only the frozen initial synthetic scenario is admitted')
    if state['standing_desires'] != frozen['spec']['wants']:
        raise ValueError('original rich desires changed')
    ids = state['obligation_ids']
    if len(ids) != 2 or len(set(ids)) != 2:
        raise ValueError('two original obligation identities required')
    for value in [state['state_root'], state['observation_revision'], *ids]:
        if not isinstance(value, str) or not re.fullmatch('[0-9a-f]{64}', value):
            raise ValueError('invalid state identity')
    if sorted(state['selected_desires']) != [0,1] or state['observation'] != [1/6, 0.5]:
        raise ValueError('initial attention/observation mismatch')
    guidance = state['relational_guidance']
    if len(guidance) != 1 or guidance[0].get('applicable') is not True:
        raise ValueError('one current original aspiration required')
    g = guidance[0]
    p = g['proposal']
    if (g['observation_revision'] != state['observation_revision'] or
            set(p['parents']) != set(ids) or p['revises'] is not None or p['catalysts'] != []):
        raise ValueError('stale or changed aspiration binding')
    for field in ['relationship', 'aspiration', 'discriminator']:
        if p[field] != frozen['aspiration'][field]:
            raise ValueError('original generated aspiration changed')
    # Reconstruct fields; never serialize the operator report, history, paths or review.
    context = {
        'synthetic_only':True, 'state_root':state['state_root'],
        'observation_revision':state['observation_revision'],
        'standing_desires':frozen['spec']['wants'], 'obligation_ids':ids,
        'selected_desires':state['selected_desires'], 'observation':[1/6,0.5],
        'archive':{'world':frozen['spec']['archive'], 'events':seed},
        'relational_guidance':{**{k:frozen['aspiration'][k] for k in ['relationship','aspiration','discriminator']},
                               'parents':sorted(ids), 'semantic_status':'unknown'},
        'action_contract':d.action_contract(state), 'minimum_directions':1,
        'seed_origin':'Evaluator-authored scenario setup, not a generated operation.'}
    schema_state = {**context, 'available_notes':[], 'notebook':{'known':[], 'claims':[], 'title':''}}
    payload = {'model':'deepseek-v4.1-flash', 'temperature':0, 'max_tokens':2048, 'stream':False,
               'messages':[{'role':'system','content':d.archive.instructions('possibilities')},
                           {'role':'user','content':json.dumps({'context':context,'response_schema':d.possibility_schema(schema_state)},sort_keys=True)}]}
    encoded=json.dumps(payload,sort_keys=True,separators=(',',':')).encode()
    return {'dispatch_enabled':False,'endpoint':'https://api.cheaperinference.com/v1/chat/completions',
            'phase':'possibilities','frozen_input_sha256':FROZEN,'payload_sha256':hashlib.sha256(encoded).hexdigest(),
            'state_root':state['state_root'],'timeout_seconds':240,'payload':payload}

def prepare_followup(state, snapshot_bytes, expected_sha256):
    """Only an explicitly pinned, already observed synthetic continuation snapshot."""
    if hashlib.sha256(snapshot_bytes).hexdigest()!=expected_sha256:
        raise ValueError('admitted continuation snapshot drift')
    snapshot=json.loads(snapshot_bytes)
    if state!=snapshot:
        raise ValueError('current state differs from admitted continuation')
    raw=(ROOT/'docs/engine-synthetic-archive-inputs.json').read_bytes()
    if hashlib.sha256(raw).hexdigest()!=FROZEN:raise ValueError('frozen input drift')
    frozen=json.loads(raw)
    if state['archive']['world']!=frozen['spec']['archive'] or state['standing_desires']!=frozen['spec']['wants']:
        raise ValueError('continuation world/parents drift')
    ids=state['obligation_ids']
    for value in [state['state_root'],state['observation_revision'],*ids]:
        if not isinstance(value,str) or not re.fullmatch('[0-9a-f]{64}',value):raise ValueError('invalid identity')
    if len(ids)!=2 or len(set(ids))!=2:raise ValueError('original parents required')
    history=state['archive']['events'];prefix=[]
    for event in history:
        d.archive.validate({'archive':{'world':frozen['spec']['archive'],'events':prefix}},
                           {'directions':[{'action':{'operation':'archive','event':event}}]})
        prefix.append(event)
    guidance=state['relational_guidance']
    if len(guidance)!=1 or guidance[0]['applicable'] is not True:raise ValueError('current guidance required')
    g=guidance[0];proposal=g['proposal']
    if g['observation_revision']!=state['observation_revision'] or set(proposal['parents'])!=set(ids):
        raise ValueError('guidance binding mismatch')
    origin=state['relational_request']['history'][0]['proposal']
    for field in ['relationship','aspiration','discriminator']:
        if origin[field]!=frozen['aspiration'][field]:raise ValueError('original A1 lineage mismatch')
        if not isinstance(proposal[field],str) or not 0<len(proposal[field].encode())<=512:raise ValueError('bounded guidance required')
    for value in [g['revision'],g['identity']]:
        if not re.fullmatch('[0-9a-f]{64}',value):raise ValueError('invalid guidance identity')
    if any(type(i) is not int or i not in (0,1) for i in state['selected_desires']):raise ValueError('invalid attention')
    if len(state['observation'])!=2 or any(type(x) not in (float,int) or not 0<=x<=1 for x in state['observation']):raise ValueError('invalid observation')
    context={'synthetic_only':True,'state_root':state['state_root'],'observation_revision':state['observation_revision'],
             'standing_desires':frozen['spec']['wants'],'obligation_ids':ids,'selected_desires':state['selected_desires'],
             'observation':state['observation'],'archive':{'world':frozen['spec']['archive'],'events':history},
             'relational_guidance':{**{k:proposal[k] for k in ['relationship','aspiration','discriminator']},
                'parents':sorted(ids),'identity':g['identity'],'revision':g['revision'],'semantic_status':'unknown'},
             'action_contract':d.action_contract(state),'minimum_directions':state['minimum_directions'],
             'continuation_origin':'Actual prior synthetic return and persisted generated revision; no new evaluator action.'}
    body={'model':'deepseek-v4.1-flash','temperature':0,'max_tokens':8192,'stream':False,
          'messages':[{'role':'system','content':d.archive.instructions('possibilities')},
                      {'role':'user','content':json.dumps({'context':context,'response_schema':d.possibility_schema(state)},sort_keys=True)}]}
    return {'dispatch_enabled':False,'endpoint':'https://api.cheaperinference.com/v1/chat/completions',
            'phase':'possibilities','frozen_input_sha256':FROZEN,'source_snapshot_sha256':expected_sha256,
            'payload_sha256':hashlib.sha256(json.dumps(body,sort_keys=True,separators=(',',':')).encode()).hexdigest(),
            'state_root':state['state_root'],'timeout_seconds':240,'payload':body}

if __name__ == '__main__':
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('request',type=Path);ap.add_argument('output',type=Path)
    args=ap.parse_args()
    prepared=prepare(json.loads(args.request.read_text()))
    with args.output.open('x') as stream:json.dump(prepared,stream,indent=2)
