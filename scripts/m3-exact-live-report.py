"""Terminal-only independent export and audit. No provider calls or execution of candidates."""
from pathlib import Path
from decimal import Decimal
from datetime import datetime, timezone
from contextlib import closing
import hashlib
import importlib.util
import json
import shutil
import sqlite3
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
def load(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/(name+'.py'))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def digest(raw):return subprocess.check_output([str(ROOT/'target/m3-blake3-checksum')],input=raw,timeout=15).decode().strip()
def save(path,value):
    with path.open('x') as f:json.dump(value,f,indent=2);f.write('\n')

def bind_charges(events, traces, process):
    start=datetime.fromisoformat(process['started']).timestamp()
    end=datetime.fromisoformat(process['finished']).timestamp()
    reserves=[e for e in events if e['kind']=='reserve' and start<=e['at']<=end]
    ids=[e['id'] for e in reserves]
    settled=[e for e in events if e['kind']=='settled' and e['id'] in ids]
    selected=[e for e in traces if e.get('reservation_id') in ids]
    if [e['id'] for e in settled]!=ids or [e['reservation_id'] for e in selected]!=ids or len(ids)>20:
        raise ValueError('arm charge attribution mismatch')
    if any(not start<=e['at']<=end for e in settled+selected) or any(e.get('settled') is not True for e in selected):
        raise ValueError('charge not settled inside process interval')
    return ids,selected,str(sum((Decimal(e['usd']) for e in settled),Decimal(0)))

def verify_completed(arm, item, decision, exported, before, after):
    """Independently bind a completed decision to process, state, receipt and billing."""
    legacy=load('m3-continuation-report');controller=load('m3-continuation-controller')
    result=item['result'];receipt=item['oracle_receipt']
    if result != exported['result'] or item['process'] != exported['process'] or item['diagnostics'] != exported['diagnostics']:
        raise ValueError('raw/exported result, process or diagnostics mismatch')
    if item['stored'] != exported['stored'] or receipt != exported['receipt']:
        raise ValueError('raw/exported store or receipt mismatch')
    legacy.verify_terminal_record(item['process'],exported['process'],result,item['stored'],receipt)
    if result['outcome'] not in ('accepted','rejected') or receipt.get('status')!='checked':
        raise ValueError('completed decision without oracle')
    actions=item['actions']
    if not actions or actions[-1]!='stop':raise ValueError('explicit final Stop missing')
    ids=controller.settled_diagnostics(before,after,item['diagnostics'])
    if len(actions)!=len(ids) or any(d.get('status')!='completed' or d.get('code')!='ok' for d in item['diagnostics']):
        raise ValueError('proposal/diagnostic mismatch')
    if (decision['task'],decision['mode'],decision['outcome'],decision['start_root'],decision['reservations']) != (arm['task'],arm['mode'],result['outcome'],result['state_root_at_start'],ids):
        raise ValueError('decision identity mismatch')
    return ids

def main():
    c=load('m3-exact-live-controller')
    manifest=json.loads(c.MANIFEST.read_text());state=ROOT/manifest['state']
    terminal=json.loads((state/'terminal.json').read_text())
    if terminal['status'] not in ('completed','halted-review-required'):raise ValueError('not terminal')
    c.previous.verify_freeze(manifest)
    c.load('m3-progress-validation-controller').require_idle()
    pids={json.loads((ROOT/'docs/m3-exact-live-launch.json').read_text())['controller_pid']}
    for path in state.glob('*.json'):
        value=json.loads(path.read_text())
        if isinstance(value,dict) and 'pid' in value:pids.add(value['pid'])
    rows=subprocess.check_output(['ps','-axo','pid=,pgid='],text=True,timeout=15).splitlines()
    if any(set(map(int,row.split()))&pids for row in rows):raise ValueError('run process/group remains')
    prior=ROOT/'docs/m3-continuation-execution'
    prior_hashes=json.loads((ROOT/'docs/m3-continuation-export-sha256.json').read_text())
    if {str(p.relative_to(prior)):sha(p) for p in prior.rglob('*') if p.is_file()}!=prior_hashes:raise ValueError('prior evidence changed')
    baseline=json.loads((ROOT/manifest['baseline']).read_text());ledger=(ROOT/manifest['ledger']).read_bytes()
    total=c.budget.check(ledger,baseline,dispatch=False)  # Unresolved charges block success, never guess.
    events=[json.loads(line) for line in ledger[baseline['bytes']:].splitlines()]
    tracepath=ROOT/'target/m3-provider-cost-diagnostics.jsonl'
    alltraces=[json.loads(line) for line in tracepath.read_text().splitlines()]
    destination=ROOT/'docs/m3-exact-live-execution'
    shutil.copytree(state,destination)
    for name in ('freeze','baseline','approval','approval-provenance','launch-preflight','launch'):
        shutil.copyfile(ROOT/'docs'/('m3-exact-live-'+name+'.json'),destination/('m3-exact-live-'+name+'.json'))
    shutil.copyfile(ROOT/'docs/m3-exact-live-launch.log',destination/'m3-exact-live-launch.log')
    (destination/'m3-provider-cost.jsonl').write_bytes(ledger)
    shutil.copyfile(tracepath,destination/tracepath.name)
    ops=c.LocalOperations(manifest);arms=[];bound=[];cas_count=0;previous_ledger=ledger[:baseline['bytes']]
    for index,arm in enumerate(manifest['arms']):
        label=arm['task'][:12]+'-'+arm['mode'];processpath=destination/(label+'.json')
        item=dict(task=arm['task'],mode=arm['mode'],intent=(destination/f'{index}-intent.json').exists(),status='not_attempted',settled_requests=0,settled_usd='0',result=None,oracle_receipt=None)
        if not processpath.exists():
            if item['intent']:item['status']='intent_without_process_uncertain'
            arms.append(item);continue
        process=json.loads(processpath.read_text());item['process']=process
        if process.get('status')=='running' or not process.get('finished'):raise ValueError('process not terminal')
        ids,traces,cost=bind_charges(events,alltraces,process);bound+=ids
        item.update(status='attempted',settled_requests=len(ids),settled_usd=cost,diagnostics=traces)
        start=datetime.fromisoformat(process['started']).timestamp();end=datetime.fromisoformat(process['finished']).timestamp()
        item['all_process_diagnostics']=[t for t in alltraces if start<=t.get('at',0)<=end]
        raw=ROOT/arm['input']/arm['mode'];archive=destination/f'forensic-{index}';archive.mkdir()
        for name in ('profile.toml','broker-contexts.json','preparation.json'):
            shutil.copyfile(ROOT/arm['input']/name,archive/name)
        shutil.copyfile(ROOT/arm['input']/'tasks.jsonl',archive/(arm['task'][:12]+'-tasks.jsonl'))
        shutil.copytree(ROOT/arm['input']/'cas',archive/'input-cas')
        if (raw/'cas').exists():shutil.copytree(raw/'cas',archive/label/'cas')
        stores=list((raw/'state').glob('*/*.sqlite'))
        kinds=[];stored=[]
        if len(stores)==1:
            # Backup a quiescent store, preserving original files and any committed WAL.
            target=archive/(label+'-state.sqlite')
            with closing(sqlite3.connect(stores[0].as_uri()+'?mode=ro',uri=True)) as source,closing(sqlite3.connect(target)) as db:
                source.backup(db);db.execute('pragma journal_mode=DELETE')
            with closing(sqlite3.connect(target.as_uri()+'?mode=ro&immutable=1',uri=True)) as db:
                if db.execute('pragma integrity_check').fetchone()[0]!='ok':raise ValueError('store integrity')
                stored=[(i,json.loads(p)) for i,p in db.execute('select id,payload from events order by seq')]
            kinds=[e['kind'] for _,e in stored]
            item['store_root']=digest(b''.join(struct.pack('<Q',len(i.encode()))+i.encode() for i,_ in stored))
        elif len(stores)>1:raise ValueError('multiple arm stores')
        for directory in (archive/'input-cas',archive/label/'cas'):
            if not directory.exists():continue
            for p in directory.iterdir():
                if p.is_symlink() or not p.is_file() or digest(p.read_bytes())!=p.name:raise ValueError('CAS mismatch')
                cas_count+=1
        resultpath=raw/('results-'+arm['mode']+'.jsonl')
        if resultpath.exists():
            shutil.copyfile(resultpath,archive/(label+'-results.jsonl'))
            results=[json.loads(line) for line in resultpath.read_text().splitlines()]
            if len(results)!=1 or results[0]['task']!=arm['task'] or results[0]['system']!=arm['mode']:raise ValueError('result identity')
            result=results[0];item['result']=result
            if result['state_root_at_start']!=digest(b'') or result['state_root_at_end']!=item.get('store_root'):raise ValueError('roots disagree')
            receipt=ops.receipts.verify_receipts(archive,label,dict(result=result,task=arm['task'],mode=arm['mode']))
            item['oracle_receipt']=receipt
            if result.get('oracle') is not None and receipt['status']!='checked':raise ValueError('oracle receipt unverified')
            item['status']='raw_'+result['outcome']
            item['forensic_outcome_verified']=False
            if result['outcome']=='unavailable' or result.get('oracle') is not None:
                load('m3-continuation-report').verify_terminal_record(process,process,result,stored,receipt)
                item['forensic_outcome_verified']=True
        if (raw/'runs.jsonl').exists():shutil.copyfile(raw/'runs.jsonl',archive/'runs.jsonl')
        source=raw/'copies'/arm['task']/arm['mode']/arm['file']
        if source.is_file() and not source.is_symlink():shutil.copyfile(source,archive/'allowed-implementation-source.txt')
        item['actions']=[e['candidate']['kind'] for e in kinds if e['kind']=='proposed']
        item['closures']=[e['reason'] for e in kinds if e['kind']=='task_closed']
        item['visible_tests']=[{k:e[k] for k in ['exit_code','duration_ms']} for e in kinds if e['kind']=='executed']
        item['oracle_receipt_event_count']=sum(e.get('observation',{}).get('evidence',{}).get('source')=='final-oracle-receipts@v1' for e in kinds)
        folder=destination/f'evidence-{index}'
        if (folder/'sha256.json').exists():
            preserved_hashes=json.loads((folder/'sha256.json').read_text())
            if {str(p.relative_to(folder)):sha(p) for p in folder.rglob('*') if p.is_file() and p.name!='sha256.json'}!=preserved_hashes:
                raise ValueError('partial controller export changed')
        if item['result'] is not None and item['result'].get('oracle') is None and item['oracle_receipt_event_count']:
            raise ValueError('unexpected oracle event')
        decisionpath=destination/f'{index}-decision.json';item['controller_decision']=None
        if decisionpath.exists():
            decision=json.loads(decisionpath.read_text());folder=destination/f'evidence-{index}'
            hashes=json.loads((folder/'sha256.json').read_text())
            actual={str(p.relative_to(folder)):sha(p) for p in folder.rglob('*') if p.is_file() and p.name!='sha256.json'}
            if actual!=hashes or decision['evidence_sha256']!=sha(folder/'sha256.json') or decision['reservations']!=ids:raise ValueError('decision/export mismatch')
            if item['result'] is None or item['result']['outcome']!=decision['outcome'] or item['oracle_receipt']['status']!='checked':raise ValueError('decision outcome mismatch')
            exported_ledger=(folder/'ledger.jsonl').read_bytes()
            if not ledger.startswith(exported_ledger):raise ValueError('exported ledger is not terminal prefix')
            with closing(sqlite3.connect((folder/(label+'-state.sqlite')).as_uri()+'?mode=ro&immutable=1',uri=True)) as db:
                exported_events=[(i,json.loads(p)) for i,p in db.execute('select id,payload from events order by seq')]
            exported_rows=[json.loads(line) for line in (folder/(label+'-results.jsonl')).read_text().splitlines()]
            if len(exported_rows)!=1:raise ValueError('export result count')
            exported=dict(result=exported_rows[0],process=json.loads((folder/'process.json').read_text()),
                diagnostics=json.loads((folder/'diagnostics.json').read_text()),stored=exported_events,
                receipt=json.loads((folder/'receipt-verification.json').read_text()))
            verify_completed(arm,item|{'stored':stored},decision,exported,previous_ledger,exported_ledger)
            previous_ledger=exported_ledger
            item['controller_decision']=decision;item['status']='verified_'+decision['outcome']
        arms.append(item)
    allids=[e['id'] for e in events if e['kind']=='reserve']
    if bound!=allids or len(bound)!=len(set(bound)):raise ValueError('unattributed or duplicate charges')
    roots={a['mode']:a['result']['state_root_at_start'] for a in arms if a['result'] is not None}
    eligible=len(roots)==2 and len(set(roots.values()))==1 and all(a['controller_decision'] is not None for a in arms)
    out=dict(controller_status=terminal['status'],arms=arms,paired_roots=dict(observed=roots,equal_when_both_observed=len(set(roots.values()))==1 if len(roots)==2 else None,eligible_complete_pair=eligible),
        costs=dict(prior_usd=baseline['settled_usd'],incremental_usd=str(total-Decimal(baseline['settled_usd'])),aggregate_usd=str(total),outstanding_usd='0',settled_requests=len(allids)),
        audit=dict(frozen_inputs=len(manifest['sha256']),prior_export_files_verified=len(prior_hashes),cas_objects_verified=cas_count,quiescent=True),
        limitations=['Root audit hashes ordered event IDs, not each canonical payload ID.','Forensic snapshots do not create missing controller decisions or make halted outcomes continuation-eligible.','Synthetic fixtures are excluded from live outcomes.'])
    save(ROOT/'docs/m3-exact-live-result.json',out)
    hashes={str(p.relative_to(destination)):sha(p) for p in destination.rglob('*') if p.is_file()}
    save(ROOT/'docs/m3-exact-live-export-sha256.json',hashes)
    save(ROOT/'docs/m3-exact-live-terminal-runtime.json',dict(at=datetime.now(timezone.utc).isoformat(),quiescent=True,checked_pids=sorted(pids),free_gib=shutil.disk_usage(ROOT).free/1024**3,export_files=len(hashes)))
    print(json.dumps(dict(status=terminal['status'],costs=out['costs'],paired_roots=out['paired_roots'],export_files=len(hashes))))

if __name__=='__main__':main()
