"""Offline readiness snapshot only. No launch, network, credentials or state mutation."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parents[1]

def main():
    old=json.loads((ROOT/'docs/m3-progress-validation-plan.json').read_text())
    summary=json.loads((ROOT/'docs/m3-progress-validation-result.json').read_text())
    prior={(a['task'],a['mode']):a for a in summary['arms']}
    arms=[]
    for job in old['jobs']:
        for mode in job['modes']:
            row=prior[(job['task_id'],mode)]
            if row['process'] is not None or row['result'] is not None:continue
            base='target/m3-continuation/'+job['task_id']
            if (ROOT/base/mode).exists() or list((ROOT/base).glob('startup-'+mode+'.sqlite*')):
                raise ValueError('not fresh')
            arms.append(dict(task=job['task_id'],mode=mode,input=base,function=job['function'],
                file=job['file'],test=job['test'],lineage=job['lineage']))
    expected=[(old['jobs'][0]['task_id'],'rules'),(old['jobs'][1]['task_id'],'record'),(old['jobs'][1]['task_id'],'rules')]
    if [(a['task'],a['mode']) for a in arms]!=expected:raise ValueError('unattempted scope changed')
    if (ROOT/'target/m3-continuation-state').exists():raise ValueError('state exists')
    frozen=json.loads((ROOT/'docs/m3-progress-validation-freeze.json').read_text())['sha256']
    for p,h in frozen.items():
        if hashlib.sha256((ROOT/p).read_bytes()).hexdigest()!=h:raise ValueError('prior freeze changed')
    files=set(frozen)
    for folder in ['target/m3-continuation','docs/m3-progress-validation-execution']:
        files.update(str(p.relative_to(ROOT)) for p in (ROOT/folder).rglob('*') if p.is_file())
    files.update(['scripts/m3-continuation-controller.py','scripts/m3-continuation-broker.py',
        'scripts/m3-freeze-continuation.py','scripts/test_m3_continuation.py',
        'scripts/m3-progress-validation-evidence.py','scripts/m3_smoke_budget.py',
        'docs/m3-continuation-baseline.json','docs/m3-progress-validation-result.json',
        'docs/m3-progress-validation-export-sha256.json','docs/m3-progress-validation-report-tools.json',
        'target/m3-blake3-checksum','target/m3-progress-receipt-check'])
    helpers=json.loads((ROOT/'docs/m3-progress-validation-report-tools.json').read_text())['helper_sha256']
    files.update(helpers)
    for p,h in helpers.items():
        if hashlib.sha256((ROOT/p).read_bytes()).hexdigest()!=h:raise ValueError('helper changed')
    baseline=json.loads((ROOT/'docs/m3-continuation-baseline.json').read_text())
    ledger=(ROOT/'target/m3-provider-cost.jsonl').read_bytes()
    if len(ledger)!=baseline['bytes'] or hashlib.sha256(ledger).hexdigest()!=baseline['sha256']:
        raise ValueError('ledger changed since preparation')
    manifest=dict(version=1,status='offline-ready-pending-scoped-approval',arms=arms,unattempted=expected,
        excluded_prior_arm=dict(task=old['jobs'][0]['task_id'],mode='record',outcome='unavailable',retry=False),
        prior_run='docs/m3-progress-validation-result.json',controller='scripts/m3-continuation-controller.py',
        broker='scripts/m3-continuation-broker.py',engine=old['engine'],
        baseline='docs/m3-continuation-baseline.json',ledger='target/m3-provider-cost.jsonl',
        state='target/m3-continuation-state',endpoint=old['endpoint'],model=old['model'],
        proposed_incremental_cap_usd='10',aggregate_cap_usd='100',baseline_settled_usd=baseline['settled_usd'],
        max_concurrent_arms=1,max_calls_per_arm=20,max_calls_total=60,max_wall_seconds_per_arm=1800,
        global_wall_seconds=6000,initial_minimum_free_gib=240,live_guard_gib=200,automatic_retries=0,
        data_scope='Only the two public RTS task definitions, allowlisted implementation files and bounded test observations; no secrets, unrelated sessions or private logs.',
        pending=['Explicit approval bound to this manifest hash, exact three arms and proposed dollar limits.','Read-only runtime/image/disk/price and frozen-input checks immediately before dispatch.'],
        sha256={p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in sorted(files)})
    (ROOT/'docs/m3-continuation-readiness.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps(dict(arms=len(arms),frozen_files=len(files),manifest_sha256=hashlib.sha256((ROOT/'docs/m3-continuation-readiness.json').read_bytes()).hexdigest())))

if __name__=='__main__':main()
