"""Terminal-only evidence export/audit. Never launches or retries an arm."""
from pathlib import Path
import hashlib
import json
import shutil
import sqlite3
import struct
import subprocess
from contextlib import closing
import m3_smoke_budget as budget
from decimal import Decimal
import importlib.util

ROOT=Path(__file__).resolve().parents[1]
def load(name):
    s=importlib.util.spec_from_file_location(name,ROOT/'scripts'/(name+'.py'))
    m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def digest(raw):return subprocess.check_output([str(ROOT/'target/m3-blake3-checksum')],input=raw,timeout=15).decode().strip()
def write(path,value):
    with path.open('x') as f:json.dump(value,f,indent=2);f.write('\n')

def verify_terminal_record(process, exported_process, result, stored, receipt):
    if not process or process.get('status')!='completed' or process.get('exit')!=0 or process.get('errors') or process!=exported_process:
        raise ValueError('process receipt mismatch or failure')
    closures=[e['kind']['reason'] for _,e in stored if e['kind']['kind']=='task_closed']
    if len(closures)!=1 or not stored or stored[-1][1]['kind']['kind']!='task_closed' or result.get('denials'):
        raise ValueError('closure/denial evidence')
    if result['outcome']=='unavailable':
        if result.get('oracle') is not None or closures!=['unavailable:broker failed; see controller cost ledger for reserved charges']:
            raise ValueError('unavailable closure')
    elif result['outcome'] in ('accepted','rejected'):
        want=result['outcome']=='accepted'
        if result.get('oracle') is None or receipt.get('status')!='checked' or result['oracle'].get('accepted') is not want or receipt.get('verified_success') is not want or closures!=['oracle:'+str(want).lower()]:
            raise ValueError('outcome/oracle/closure mismatch')
    else:raise ValueError('unknown outcome')
    return closures


def compare_roots(manifest, results, historical):
    root_comparisons=[]
    for task in dict.fromkeys(a['task'] for a in manifest['arms']):
        observed={r['mode']:r['result']['state_root_at_start'] for r in results if r['task']==task and r['evidence_status']=='verified'}
        if task.startswith('0c5a91'):
            old=next(a for a in historical['arms'] if a['task']==task and a['mode']=='record')
            if old['result'] is not None:observed['record']=old['result']['state_root_at_start']
        root_comparisons.append(dict(task=task,observed_start_roots=observed,both_observed=len(observed)==2,equal_when_both_observed=len(set(observed.values()))==1 if len(observed)==2 else None))
    return root_comparisons


def main():
    controller=load('m3-continuation-controller')
    manifest=json.loads((ROOT/'docs/m3-continuation-readiness.json').read_text())
    state=ROOT/manifest['state'];terminal=json.loads((state/'terminal.json').read_text())
    if terminal['status'] not in ('completed','halted-review-required'):raise ValueError('not terminal')
    controller.verify_freeze(manifest)
    load('m3-progress-validation-controller').require_idle()
    pids={json.loads((ROOT/'docs/m3-continuation-launch.json').read_text())['controller_pid']}
    for p in state.glob('*.json'):
        row=json.loads(p.read_text())
        if isinstance(row,dict) and 'pid' in row:pids.add(row['pid'])
    ps=subprocess.check_output(['ps','-axo','pid=,pgid='],text=True,timeout=15).splitlines()
    if any(set(map(int,row.split()))&pids for row in ps):raise ValueError('run process/group remains')
    prior=ROOT/'docs/m3-progress-validation-execution'
    prior_hashes=json.loads((ROOT/'docs/m3-progress-validation-export-sha256.json').read_text())
    if {str(p.relative_to(prior)):sha(p) for p in prior.rglob('*') if p.is_file()}!=prior_hashes:raise ValueError('prior evidence changed')
    destination=ROOT/'docs/m3-continuation-execution'
    shutil.copytree(state,destination)
    for name in ('m3-provider-cost.jsonl','m3-provider-cost-diagnostics.jsonl','m3-continuation-launch.log'):
        shutil.copyfile(ROOT/'target'/name,destination/name)
    for name in ('m3-continuation-readiness.json','m3-continuation-baseline.json','m3-continuation-authorization.json','m3-continuation-authorization-provenance.json','m3-continuation-live-preflight.json','m3-continuation-price-preflight.json','m3-continuation-launch.json'):
        shutil.copyfile(ROOT/'docs'/name,destination/name)
    baseline=json.loads((ROOT/manifest['baseline']).read_text())
    ledger=(destination/'m3-provider-cost.jsonl').read_bytes()
    if budget.freeze(ledger[:baseline['bytes']])!=baseline:raise ValueError('baseline prefix changed')
    # An unsettled terminal ledger is not silently assigned a settled cost.
    total=budget.total(ledger);delta=total-Decimal(baseline['settled_usd'])
    if not 0<=delta<=10 or total>100:raise ValueError('cost outside approved caps')
    events=[json.loads(x) for x in ledger[baseline['bytes']:].splitlines()]
    charges={e['id']:Decimal(e['usd']) for e in events if e['kind']=='settled'}
    trace_all=[json.loads(x) for x in (destination/'m3-provider-cost-diagnostics.jsonl').read_text().splitlines()]
    traces=[r for r in trace_all if r.get('reservation_id') in charges]
    receipts=load('m3-progress-validation-evidence')
    results=[];bound=[];cas_count=0
    for index,arm in enumerate(manifest['arms']):
        label=arm['task'][:12]+'-'+arm['mode'];folder=destination/('evidence-'+str(index))
        process_file=destination/(label+'.json')
        process=json.loads(process_file.read_text()) if process_file.exists() else None
        intent_present=(destination/(str(index)+'-intent.json')).exists()
        item=dict(task=arm['task'],mode=arm['mode'],process=process,result=None,evidence_status='not_attempted' if process is None and not intent_present else 'unverified_attempt',receipt_verification=None)
        decision_file=destination/(str(index)+'-decision.json')
        complete_export=(folder/'sha256.json').is_file() and decision_file.is_file()
        if complete_export:
            expected=json.loads((folder/'sha256.json').read_text())
            actual={str(p.relative_to(folder)):sha(p) for p in folder.rglob('*') if p.is_file() and p.name!='sha256.json'}
            if actual!=expected:raise ValueError('per-arm export hash mismatch')
            rows=[json.loads(x) for x in (folder/(label+'-results.jsonl')).read_text().splitlines()]
            if len(rows)!=1 or rows[0]['task']!=arm['task'] or rows[0]['system']!=arm['mode']:raise ValueError('result identity')
            item['result']=rows[0]
            with closing(sqlite3.connect('file:'+str(folder/(label+'-state.sqlite'))+'?mode=ro&immutable=1',uri=True)) as db:
                if db.execute('pragma integrity_check').fetchone()[0]!='ok':raise ValueError('sqlite integrity')
                stored=[(i,json.loads(p)) for i,p in db.execute('select id,payload from events order by seq')]
            root=digest(b''.join(struct.pack('<Q',len(i.encode()))+i.encode() for i,_ in stored))
            if rows[0]['state_root_at_start']!=digest(b'') or rows[0]['state_root_at_end']!=root:raise ValueError('root mismatch')
            for directory in (folder/'input-cas',folder/label/'cas'):
                for p in directory.iterdir():
                    if digest(p.read_bytes())!=p.name:raise ValueError('CAS mismatch')
                    cas_count+=1
            verified=receipts.verify_receipts(folder,label,dict(result=rows[0],task=arm['task'],mode=arm['mode']))
            if verified!=json.loads((folder/'receipt-verification.json').read_text()):raise ValueError('receipt reread mismatch')
            if rows[0].get('oracle') is not None and verified['status']!='checked':raise ValueError('unverified invoked oracle')
            verify_terminal_record(process,json.loads((folder/'process.json').read_text()),rows[0],stored,verified)
            diagnostics=json.loads((folder/'diagnostics.json').read_text())
            ids=[d['reservation_id'] for d in diagnostics]
            if [d for d in traces if d['reservation_id'] in ids]!=diagnostics:raise ValueError('diagnostic binding')
            decision_file=destination/(str(index)+'-decision.json')
            if not decision_file.exists():raise ValueError('export has no verified decision')
            decision=json.loads(decision_file.read_text())
            if decision['evidence_sha256']!=sha(folder/'sha256.json') or decision['reservations']!=ids or decision['outcome']!=rows[0]['outcome']:raise ValueError('decision binding')
            exported_ledger=(folder/'ledger.jsonl').read_bytes()
            before=ledger[:baseline['bytes']] if not bound else previous_ledger
            controller.disposition(rows[0],diagnostics,before,exported_ledger,sum(e['kind']['kind']=='proposed' for _,e in stored))
            if not ledger.startswith(exported_ledger):raise ValueError('arm ledger prefix')
            previous_ledger=exported_ledger;bound+=ids
            item.update(evidence_status='verified',receipt_verification=verified,diagnostics=diagnostics,
                settled_usd=str(sum((charges[i] for i in ids),Decimal(0))),closure_reasons=[e['kind']['reason'] for _,e in stored if e['kind']['kind']=='task_closed'])
        elif process is not None or intent_present or folder.exists():
            item['evidence_status']='partial_export' if folder.exists() else 'unverified_attempt'
            # Preserve original raw output on an infrastructure/global halt; do not fabricate verification.
            raw=ROOT/arm['input']/arm['mode']
            if raw.exists():
                saved=destination/('unverified-raw-'+str(index));saved.mkdir()
                for name in ('cas','state'):
                    if (raw/name).exists():shutil.copytree(raw/name,saved/name)
                for name in ('results-'+arm['mode']+'.jsonl','runs.jsonl'):
                    if (raw/name).exists():shutil.copyfile(raw/name,saved/name)
                source=raw/'copies'/arm['task']/arm['mode']/arm['file']
                if source.is_file() and not source.is_symlink():shutil.copyfile(source,saved/'allowed-implementation-source.txt')
            result_path=raw/('results-'+arm['mode']+'.jsonl')
            if result_path.exists():item['raw_results']=[json.loads(x) for x in result_path.read_text().splitlines()]
        results.append(item)
    if len(bound)!=len(set(bound)):raise ValueError('duplicate arm charges')
    unbound={i:str(cost) for i,cost in charges.items() if i not in bound}
    route=[r for r in results if r['task'].startswith('47c787')]
    pairs=[]
    if len(route)==2 and all(r['evidence_status']=='verified' and r['result']['outcome'] in ('accepted','rejected') for r in route):
        if route[0]['result']['state_root_at_start']!=route[1]['result']['state_root_at_start']:raise ValueError('paired start mismatch')
        pairs.append(dict(task=route[0]['task'],start_root=route[0]['result']['state_root_at_start'],outcomes={r['mode']:r['result']['outcome'] for r in route}))
    historical=json.loads((ROOT/'docs/m3-progress-validation-result.json').read_text())
    root_comparisons=compare_roots(manifest,results,historical)
    report=dict(controller_status=terminal['status'],arms=results,completed_pairs=pairs,observed_start_root_comparisons=root_comparisons,
        historical_protocol_record='unavailable; excluded from execution and complete pairs',
        spending=dict(prior_usd=baseline['settled_usd'],incremental_usd=str(delta),aggregate_usd=str(total),outstanding_usd='0',requests=len(charges),unbound_charges=unbound),
        audit=dict(frozen_inputs=len(manifest['sha256']),prior_export_files=len(prior_hashes),cas_verified=cas_count,all_attempted_evidence_verified=all(r['evidence_status'] in ('verified','not_attempted') for r in results) and not unbound),
        limitations=['Stored roots recompute ordered event IDs, not each canonical payload ID.','No synthetic fixture result is treated as a live outcome.','Unavailable arms and incomplete pairs remain visible.'])
    write(ROOT/'docs/m3-continuation-result.json',report)
    hashes={str(p.relative_to(destination)):sha(p) for p in destination.rglob('*') if p.is_file()}
    write(ROOT/'docs/m3-continuation-export-sha256.json',hashes)
    print(json.dumps(dict(status=report['controller_status'],audit=report['audit'],spending=report['spending'],export_files=len(hashes))))

if __name__=='__main__':main()
