#!/usr/bin/env python3
"""Explicit frozen synthetic scope extension on the existing campaign; no new budget or ledger."""
import argparse
import hashlib
import json
from pathlib import Path
from types import SimpleNamespace
import importlib.util
import time
import os
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[1]
def load(name,file):
    s=importlib.util.spec_from_file_location(name,ROOT/'scripts'/file);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
c=load('next_campaign','engine-b91-campaign.py')
d=load('next_directions','engine-directions.py')
s=d.load_synthetic_cycle()
PLAN=ROOT/'docs/engine-campaign-next-choice-plan.md'
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def extension(manifest,admission):
    value={'contract':'campaign-garden-extension@v1','campaign_sha256':c.digest(manifest),
        'admission_sha256':s.digest(admission),'source_class':'frozen-synthetic-'+admission.get('scenario','')+'-only',
        'plan_sha256':sha(PLAN),'wrapper_sha256':sha(__file__),'scenario_sha256':s.SCENARIOS[admission['scenario']][1],
        'calls':admission['calls'],'simulated_operations':admission['actions'],'execution_accounting':'one_simulator_attempt_consumes_one_shared_execution_slot',
        'original_campaign_manifest_unchanged':True}
    if admission.get('runtime_review'):
        value['runtime_review']=admission['runtime_review'];value['review_recovery']=admission.get('review_recovery');value['total_calls']=admission['total_call_limit']
        value['review_plan_sha256']=sha(ROOT/'docs/engine-campaign-runtime-review-plan.md')
    if admission.get('review_recovery'):value['recovery_plan_sha256']=sha(ROOT/'docs/engine-campaign-review-recovery-plan.md')
    if admission.get('scenario')=='workshop' or admission.get('idle_profile'):value['transfer_plan_sha256']=sha(ROOT/'docs/engine-campaign-transfer-plan.md')
    if admission.get('reviewed_multi_profile'):
        value['reviewed_multi_profile']=admission['reviewed_multi_profile']
        value['reviewed_multi_plan_sha256']=sha(ROOT/'docs/engine-reviewed-multi-plan.md')
    if admission.get('autonomy_profile'):value['autonomy_plan_sha256']=sha(ROOT/'docs/engine-campaign-autonomy-plan.md')
    if admission.get('continuation'):value['continuation_plan_sha256']=sha(ROOT/'docs/engine-campaign-next-choice-continuation.md')
    return value
def run(ledger,a,ext,ext_hash,provider,*,quote=None,planned_restart=False):
    args=SimpleNamespace(directory=Path(a['directory']),model=c.MODEL,synthetic_admission=Path(a['directory'])/'admission.json',stop_file=Path(a['directory'])/'STOP',expire_remaining=None,expected_admission_hash=s.digest(a))
    s.validate_admission(a,args)
    if a.get('scenario') not in ('garden','workshop') or ext!=extension(ledger.manifest,a) or s.digest(ext)!=ext_hash:raise ValueError('exact reviewed synthetic extension required')
    if ledger.status()['stopped']:raise ValueError('campaign stopped')
    trial=s.digest({'garden_directory':str(args.directory.resolve()),'admission':s.digest(a),'extension':ext_hash})
    ledger.enroll(trial,s.digest(a))
    def invoke(number,body,timeout):
        if quote is not None and time.time()-quote['retrieved_at']>3600:raise ValueError('price quote expired')
        return ledger.run_call(trial,f'garden-call-{number}',body,timeout,provider,payload_limit=a['max_payload_bytes'])
    if planned_restart:
        if a.get('autonomy_profile') not in ('garden-multi@v1','workshop-multi@v1') and a.get('reviewed_multi_profile')!='reviewed-multi@v1':raise ValueError('restart profile not admitted')
        def checkpoint(point):
            out=args.directory/'synthetic-external';marker=out/'planned-restart.json'
            if point=='outcome' and (out/'outcome-2.json').exists() and not marker.exists():
                s.save(marker,{'admission_sha256':s.digest(a),'outcome_sha256':s.digest(s.read(out/'outcome-2.json')),'kind':'acknowledged_provider_outcome_not_application'})
                os._exit(75)
        s.checkpoint=checkpoint
    args.synthetic_provider=invoke
    args.synthetic_action=lambda number,apply:ledger.run_execution(trial,f'garden-simulated-operation-{number}',s.digest(a),apply)
    return s.run(args,d.__dict__)
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('mode',choices=['prepare-extension','run'])
    p.add_argument('--campaign',type=Path,required=True);p.add_argument('--campaign-sha256',required=True)
    p.add_argument('--upgrade',type=Path);p.add_argument('--upgrade-sha256')
    p.add_argument('--admission',type=Path,required=True);p.add_argument('--extension',type=Path,required=True)
    p.add_argument('--extension-sha256');p.add_argument('--allow-paid-inference',action='store_true');p.add_argument('--allow-simulated-operation',action='store_true')
    p.add_argument('--resume-child',action='store_true',help=argparse.SUPPRESS)
    args=p.parse_args();m=c.strict((args.campaign/'manifest.json').read_bytes());a=s.read(args.admission)
    if c.digest(m)!=args.campaign_sha256:raise ValueError('campaign hash mismatch')
    if args.admission.resolve()!=Path(a['directory']).resolve()/'admission.json':raise ValueError('admission location')
    if args.mode=='prepare-extension':
        x=extension(m,a);s.save(args.extension,x);print(json.dumps({'extension_sha256':s.digest(x),'dispatch':False}));return
    if not args.allow_paid_inference or not args.allow_simulated_operation:raise ValueError('both explicit effect opt-ins required')
    if (a.get('autonomy_profile') or a.get('reviewed_multi_profile')) and not args.resume_child:
        command=[sys.executable,str(Path(__file__).resolve()),*sys.argv[1:],'--resume-child']
        result=subprocess.run(command,check=False)
        if result.returncode==75:
            out=Path(a['directory'])/'synthetic-external';marker=s.read(out/'planned-restart.json')
            if marker!={'admission_sha256':s.digest(a),'outcome_sha256':s.digest(s.read(out/'outcome-2.json')),'kind':'acknowledged_provider_outcome_not_application'}:raise ValueError('planned restart binding')
            if (out/'intent-3.json').exists() or (out/'apply-intent-2.json').exists():raise ValueError('restart crossed unexpected effect')
            result=subprocess.run(command,check=False)
        raise SystemExit(result.returncode)
    upgrade=c.strict(args.upgrade.read_bytes()) if args.upgrade else None
    ledger=c.Ledger(args.campaign/'ledger.jsonl',m,args.campaign_sha256,upgrade=upgrade,upgrade_hash=args.upgrade_sha256)
    ext=s.read(args.extension)
    if ext!=extension(m,a) or args.extension_sha256!=s.digest(ext):raise ValueError('extension mismatch')
    if ledger.status()['stopped']:raise ValueError('campaign stopped')
    quote=c.official_preflight() if a['calls'] else None
    if quote is not None:ledger.record_price(quote)
    transport=s.load_transport()
    result=run(ledger,a,ext,args.extension_sha256,lambda body,timeout:transport.invoke(body,timeout,expected_model=c.MODEL,payload_limit=a['max_payload_bytes']),quote=quote,planned_restart=bool(a.get('autonomy_profile') or a.get('reviewed_multi_profile')))
    print(json.dumps({'result':result,'campaign':ledger.status()},indent=2))
if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':main()
