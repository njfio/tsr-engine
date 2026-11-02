#!/usr/bin/env python3
"""Initialize only a pinned synthetic five-stage scenario and freeze its offline admission.

The four prescribed setup events are evaluator-authored initialization, not generated
behavior. No provider call, new aspiration or discretionary operation occurs here.
Use a new directory; partial setup must be inspected, never silently resumed/reset.
"""
import argparse
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace

ROOT=Path(__file__).resolve().parents[1]

def prepare(directory, expires, scenario='workshop', model='deepseek-v4.1-flash', multi=False, idle=False, runtime_review=False, review_recovery=False, reviewed_multi=False):
    if reviewed_multi and (multi or idle or not runtime_review or not review_recovery or model!='gpt-6-luna'):raise ValueError('reviewed multi requires explicit review/recovery and exclusive profile')
    if (multi and idle) or ((multi or idle) and (model!='gpt-6-luna' or scenario not in ('workshop','garden'))):raise ValueError('explicit compatible multi or idle profile required')
    spec=importlib.util.spec_from_file_location('workshop_driver',ROOT/'scripts/engine-directions.py')
    d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d);c=d.load_synthetic_cycle()
    plan=c.fixture(scenario);directory=Path(directory).resolve();directory.mkdir(exist_ok=False)
    c.save(directory/'setup-provenance.json',{'contract':'workshop-frozen-initialization@v1','scenario':scenario,'plan_sha256':c.SCENARIOS[scenario][1],
        'events':plan['initial_events'],'meaning':'Evaluator-authored simulation setup only. No generated aspiration or provider call.'})
    c.save(directory/'spec.json',plan['spec']);d.cli(directory,'direction-init','--spec',str(directory/'spec.json'))
    past=[];world=plan['spec']['archive'];n=len(world['challenges'])
    for index,event in enumerate(plan['initial_events']):
        q=d.cli(directory,'direction-request')
        d.archive.validate({'archive':{'world':world,'events':past}},
                           {'directions':[{'action':{'operation':'archive','event':event}}]})
        past.append(event)
        registered=sum(any(e['kind']=='register' and e['challenge']==ch['id'] for e in past) for ch in world['challenges'])
        routed=sum(any(e['kind']=='route' and e['challenge']==ch['id'] for e in past) for ch in world['challenges'])
        # Frozen setup contains registration/routing only; no response or outcome supplied.
        assert all(e['kind'] in ('register','route') for e in past)
        anticipated=[(registered+routed)/(3*n),registered/n]
        offer={'state_root':q['state_root'],'directions':[{'purpose':'Evaluator-authored frozen synthetic initialization.',
            'action':{'operation':'archive','event':event},'forecasts':dict(zip(q['obligation_ids'],anticipated)),
            'supports':[key for key,old,new in zip(q['obligation_ids'],q['observation'],anticipated) if new>old]}]}
        path=directory/f'setup-{index+1}.json';c.save(path,d.normalize_forecasts(q,offer))
        d.cli(directory,'deliberate','--response',str(path))
    if idle:
        for index in range(2):
            q=d.cli(directory,'direction-request');path=directory/f'withdraw-setup-{index}.json'
            c.save(path,{'state_root':q['state_root'],'preference':q['obligation_ids'][index],'status':'withdrawn'})
            d.cli(directory,'direction-status','--update',str(path))
    args=SimpleNamespace(directory=directory,model=model,calls=0 if idle else (8 if multi or reviewed_multi else 5),synthetic_actions=0 if idle else (2 if multi or reviewed_multi else 1),synthetic_multi=multi,synthetic_reviewed_multi=reviewed_multi,synthetic_idle=idle,synthetic_runtime_review=runtime_review,synthetic_review_recovery=review_recovery,synthetic_scenario=scenario,
         synthetic_expires=expires,prepare_synthetic_admission=directory/'admission.json')
    result=d.run(args);c.save(directory/'preparation.json',result)
    q=d.cli(directory,'direction-request');a=c.read(directory/'admission.json')
    body=None
    if not idle:body,_=c.payload(q,'relational',None,a,d);c.save(directory/'initial-request.json',body)
    return {'status':'prepared_not_dispatched','directory':str(directory),'initial_root':q['state_root'],
            'initial_payload_bytes':len(json.dumps(body).encode()) if body is not None else 0,'admission':str(directory/'admission.json')}

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True)
    p.add_argument('--scenario',choices=['workshop','garden'],default='workshop')
    p.add_argument('--model',choices=['deepseek-v4.1-flash','gpt-6-luna'],default='deepseek-v4.1-flash')
    p.add_argument('--review-recovery',action='store_true',help='One attributed semantic reconsideration; requires runtime review')
    p.add_argument('--runtime-review',action='store_true',help='Separate bounded semantic veto/hold calls, charged to same campaign')
    p.add_argument('--idle',action='store_true',help='Withdraw both original wants during fixture initialization; zero calls/actions')
    p.add_argument('--reviewed-multi',action='store_true',help='Two-operation reviewed/recoverable profile without scripted desire withdrawal')
    p.add_argument('--multi',action='store_true',help='Explicit garden/workshop eight-call/two-operation profile with preregistered withdrawal')
    p.add_argument('--expires',required=True,help='Timezone-aware admission expiry; explicit, no automatic renewal')
    a=p.parse_args();print(json.dumps(prepare(a.directory,a.expires,a.scenario,a.model,a.multi,a.idle,a.runtime_review,a.review_recovery,a.reviewed_multi),indent=2))
