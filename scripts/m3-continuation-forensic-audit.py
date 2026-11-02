"""Read-only terminal audit of the correctly halted, partial second export.

Does not create a controller continuation decision or reinterpret rejection.
"""
from pathlib import Path
from decimal import Decimal
from datetime import datetime
import hashlib
import json
import sqlite3
import struct
import subprocess
from contextlib import closing
import m3_smoke_budget as budget

ROOT=Path(__file__).resolve().parents[1]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def digest(raw):return subprocess.check_output([str(ROOT/'target/m3-blake3-checksum')],input=raw,timeout=15).decode().strip()
def main():
    export=ROOT/'docs/m3-continuation-execution'
    manifest=json.loads((ROOT/'docs/m3-continuation-export-sha256.json').read_text())
    if {str(p.relative_to(export)):sha(p) for p in export.rglob('*') if p.is_file()}!=manifest:raise ValueError('export changed')
    freeze=json.loads((ROOT/'docs/m3-continuation-readiness.json').read_text())
    for path,expected in freeze['sha256'].items():
        if sha(ROOT/path)!=expected:raise ValueError('frozen input changed')
    baseline=json.loads((ROOT/'docs/m3-continuation-baseline.json').read_text())
    ledger=(export/'m3-provider-cost.jsonl').read_bytes()
    if budget.freeze(ledger[:baseline['bytes']])!=baseline:raise ValueError('baseline mismatch')
    before=(export/'evidence-0/ledger.jsonl').read_bytes()
    if not ledger.startswith(before):raise ValueError('first arm prefix mismatch')
    tail=[json.loads(x) for x in ledger[len(before):].splitlines()]
    reserves=[e for e in tail if e['kind']=='reserve']
    settlements=[e for e in tail if e['kind']=='settled']
    budget.total(ledger)
    if [e['id'] for e in reserves]!=[e['id'] for e in settlements] or len(reserves)!=16:raise ValueError('second arm settlement count/order')
    traces=[json.loads(x) for x in (export/'m3-provider-cost-diagnostics.jsonl').read_text().splitlines()]
    ids=[e['id'] for e in reserves]
    trace=[d for d in traces if d.get('reservation_id') in ids]
    if [d['reservation_id'] for d in trace]!=ids or any(d.get('code')!='ok' or d.get('status')!='completed' or d.get('settled') is not True for d in trace):raise ValueError('second arm diagnostics mismatch')
    label='47c787ed4f89-record';partial=export/'evidence-1';raw=export/'unverified-raw-1'
    process=json.loads((export/(label+'.json')).read_text())
    if process['status']!='completed' or process['exit']!=0 or process['errors']:raise ValueError('process failure')
    start=datetime.fromisoformat(process['started']).timestamp();finish=datetime.fromisoformat(process['finished']).timestamp()
    if any(not start<=e['at']<=finish for e in reserves+settlements+trace):raise ValueError('charge outside arm interval')
    result_file=partial/(label+'-results.jsonl')
    if result_file.read_bytes()!=(raw/'results-record.jsonl').read_bytes():raise ValueError('result copies differ')
    rows=[json.loads(x) for x in result_file.read_text().splitlines()]
    result=rows[0]
    if len(rows)!=1 or result['task']!=freeze['arms'][1]['task'] or result['system']!='record' or result['outcome']!='rejected' or result['oracle'] is not None or result['denials']:raise ValueError('result mismatch')
    with closing(sqlite3.connect('file:'+str(partial/(label+'-state.sqlite'))+'?mode=ro&immutable=1',uri=True)) as db:
        if db.execute('pragma integrity_check').fetchone()[0]!='ok':raise ValueError('store integrity')
        events=[(i,json.loads(p)) for i,p in db.execute('select id,payload from events order by seq')]
    root=digest(b''.join(struct.pack('<Q',len(i.encode()))+i.encode() for i,_ in events))
    if root!=result['state_root_at_end'] or result['state_root_at_start']!=digest(b''):raise ValueError('root mismatch')
    kinds=[e['kind'] for _,e in events]
    closure=[e['reason'] for e in kinds if e['kind']=='task_closed']
    if closure!=['tamper:changed full-suite test evidence `crates/rts-mcp/src/roots.rs`'] or kinds[-1]['kind']!='task_closed':raise ValueError('closure mismatch')
    actions=[e['candidate']['kind'] for e in kinds if e['kind']=='proposed']
    if len(actions)!=16 or actions[-1]!='stop':raise ValueError('action count or submission mismatch')
    oracle_events=[e for e in kinds if e.get('observation',{}).get('evidence',{}).get('source')=='final-oracle-receipts@v1']
    if oracle_events:raise ValueError('unexpected oracle receipts')
    cas_count=0
    for directory in (partial/'input-cas',partial/label/'cas',raw/'cas'):
        for p in directory.iterdir():
            if p.is_symlink() or not p.is_file() or digest(p.read_bytes())!=p.name:raise ValueError('CAS mismatch')
            cas_count+=1
    if (export/'2-intent.json').exists() or (export/'47c787ed4f89-rules.json').exists():raise ValueError('third arm attempted')
    second=sum((Decimal(e['usd']) for e in settlements),Decimal(0))
    first=budget.total(before)-Decimal(baseline['settled_usd'])
    total=budget.total(ledger)
    out=dict(status='passed-forensic-audit-not-continuation-eligible',export_files=len(manifest),frozen_inputs=len(freeze['sha256']),second_arm_cas_objects_checked=cas_count,
        second_arm=dict(task=result['task'],mode='record',raw_outcome='rejected',closure=closure[0],final_oracle='not_invoked',final_oracle_receipt_events=0,start_root=result['state_root_at_start'],end_root=root,actions=actions,visible_tests=[{k:e[k] for k in ['exit_code','duration_ms']} for e in kinds if e['kind']=='executed'],settled_requests=16,settled_usd=str(second),process_started=process['started'],process_finished=process['finished'],source_sha256=sha(raw/'allowed-implementation-source.txt')),
        spending=dict(first_arm_usd=str(first),second_arm_usd=str(second),third_arm_usd='0',incremental_usd=str(first+second),aggregate_usd=str(total),outstanding_usd='0'),third_arm='not_attempted',
        limitations=['Original controller export remains partial; no manifest/decision was retroactively added.','The initial report unbound_charges means not bound to a successful controller export decision, not unpaid or uncertain. This audit binds all16 second-arm charges by exact ledger suffix, diagnostic order and arm timestamps.','Root checks hash ordered event IDs, not canonical payload identities. No candidate code was executed by this audit.'])
    with (ROOT/'docs/m3-continuation-forensic-audit.json').open('x') as f:json.dump(out,f,indent=2);f.write('\n')
    print(json.dumps(out,indent=2))
if __name__=='__main__':main()
