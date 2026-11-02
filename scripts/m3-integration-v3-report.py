"""Preserve the incomplete v3 native run; never invent a result or final receipt."""
from decimal import Decimal
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sqlite3
import struct
import subprocess
ROOT=Path(__file__).resolve().parents[1]
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def save(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
def load(name):
    s=importlib.util.spec_from_file_location(name,ROOT/'scripts'/(name+'.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m

def main():
    m=json.loads((ROOT/'docs/m3-integration-v3-freeze.json').read_text());state=ROOT/m['state'];terminal=json.loads((state/'terminal.json').read_text())
    if terminal['status']!='halted-review-required':raise ValueError('unexpected terminal')
    runtime=json.loads((ROOT/'docs/m3-integration-v3-terminal-runtime.json').read_text())
    if runtime['status']!='quiescent':raise ValueError('runtime unchecked')
    for p,h in m['sha256'].items():
        if sha(ROOT/p)!=h:raise ValueError('frozen input changed')
    budget=load('m3-exact-budget');raw=(ROOT/m['ledger']).read_bytes();baseline=json.loads((ROOT/m['baseline']).read_text());launch=json.loads((ROOT/m['launch_baseline']).read_text())
    total=budget.check(raw,baseline,dispatch=False)
    if budget.history.freeze(raw[:launch['bytes']])!=launch:raise ValueError('launch prefix changed')
    rows=[json.loads(l) for l in raw[launch['bytes']:].splitlines()];reserves=[r['id'] for r in rows if r['kind']=='reserve'];settled=[r['id'] for r in rows if r['kind']=='settled']
    if len(set(reserves))!=len(reserves) or reserves!=settled or len(reserves)!=4:raise ValueError('settlement identity mismatch')
    diagnostics=[json.loads(l) for l in (ROOT/'target/m3-provider-cost-diagnostics.jsonl').read_text().splitlines()]
    diagnostics=[d for d in diagnostics if d.get('reservation_id') in reserves]
    if [d['reservation_id'] for d in diagnostics]!=reserves or any(d.get('settled') is not True for d in diagnostics):raise ValueError('diagnostic billing mismatch')
    if [d.get('proposal_action') for d in diagnostics]!=['inspect','edit','test','edit'] or any(d.get('code')!='ok' for d in diagnostics):raise ValueError('unexpected protocol outcome')
    if (state/'1-intent.json').exists():raise ValueError('rules unexpectedly started')
    arm=m['arms'][0];base=ROOT/arm['input'];out=base/'record';label=arm['task'][:12]+'-record'
    if list(out.glob('results*')):raise ValueError('unexpected result; audit separately')
    process=json.loads((state/(label+'.json')).read_text())
    if process['exit']!=2 or process['errors']:raise ValueError('unexpected process termination')
    export=ROOT/'docs/m3-integration-v3-execution';shutil.copytree(state,export);partial=export/'partial-record';partial.mkdir()
    stores=list((out/'state').glob('*/*.sqlite'))
    if len(stores)!=1 or stores[0].is_symlink():raise ValueError('store count/type')
    dbpath=partial/'state.sqlite'
    with sqlite3.connect(stores[0].as_uri()+'?mode=ro',uri=True) as src,sqlite3.connect(dbpath) as dst:src.backup(dst);dst.execute('pragma journal_mode=DELETE')
    with sqlite3.connect(dbpath.as_uri()+'?mode=ro&immutable=1',uri=True) as db:
        if db.execute('pragma integrity_check').fetchone()[0]!='ok':raise ValueError('store corrupt')
        events=[(i,json.loads(p)['kind']) for i,p in db.execute('select id,payload from events order by seq')]
    proposals=[e['candidate']['kind'] for _,e in events if e['kind']=='proposed'];exits=[e.get('exit_code') for _,e in events if e['kind']=='executed']
    if proposals!=['inspect','propose_patch','run_approved_test'] or exits!=[0] or any(e['kind']=='task_closed' for _,e in events):raise ValueError('unexpected partial events')
    digest=lambda data:subprocess.check_output([str(ROOT/'target/m3-blake3-checksum')],input=data,timeout=15).decode().strip()
    for folder,dest in [(out/'cas',partial/'cas'),(base/'cas',partial/'input-cas')]:
        dest.mkdir()
        for p in folder.iterdir():
            if p.is_symlink() or not p.is_file() or digest(p.read_bytes())!=p.name:raise ValueError('unsafe or corrupt CAS')
            shutil.copyfile(p,dest/p.name)
    for name in ['tasks.jsonl','profile.toml','broker-contexts.json','preparation.json']:shutil.copyfile(base/name,partial/name)
    candidate=out/'copies'/arm['task']/'record'/'src/lib.rs'
    if candidate.is_symlink() or not candidate.is_file() or candidate.stat().st_size>65536:raise ValueError('candidate source unsafe or oversized')
    source=candidate.read_bytes();(partial/'candidate-lib.rs').write_bytes(source)
    save(partial/'diagnostics.json',diagnostics)
    for name in ['m3-integration-v3-freeze.json','m3-integration-v3-launch-baseline.json','m3-integration-baseline.json','m3-integration-v3-terminal-runtime.json']:shutil.copyfile(ROOT/'docs'/name,export/name)
    (export/'m3-provider-cost.jsonl').write_bytes(raw)
    stage_calls=sum(json.loads(l)['kind']=='reserve' for l in raw[baseline['bytes']:].splitlines())
    summary=dict(status='blocked-edit-digest-precondition',synthetic_only=True,record=dict(process=process,native_result=None,result_missing=True,proposal_kinds=proposals,visible_test_exit_codes=exits,explicit_submission=False,task_closure=False,final_oracle_invocations=0,persisted_partial_root=digest(b''.join(struct.pack('<Q',len(i.encode()))+i.encode() for i,_ in events)),candidate_source_bytes=len(source),candidate_sha256=hashlib.sha256(source).hexdigest(),diagnostics=diagnostics),rules=dict(status='not_attempted'),stage_requests=stage_calls,current_attempt_requests=len(reserves),spending=dict(current_attempt_usd=str(total-Decimal(launch['settled_usd'])),cumulative_stage_usd=str(total-Decimal(baseline['settled_usd'])),aggregate_usd=str(total),outstanding_usd='0'),prior_failure_preserved=True,qualified_corpus_tasks_added=0,freeze_sha256=sha(ROOT/'docs/m3-integration-v3-freeze.json'),blocker='All wire proposals validated. Final edit failed the native source-digest precondition; no Stop, closure, result row or final oracle receipt. Visible test success is not final acceptance.')
    save(ROOT/'docs/m3-integration-v3-result.json',summary)
    save(ROOT/'docs/m3-integration-v3-export-sha256.json',{str(p.relative_to(export)):sha(p) for p in export.rglob('*') if p.is_file()})
    print(json.dumps({k:v for k,v in summary.items() if k not in ('record','rules')}))
if __name__=='__main__':main()
