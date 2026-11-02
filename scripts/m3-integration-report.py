"""Preserve the terminal synthetic attempt without replaying or calling providers."""
import datetime
from decimal import Decimal
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sqlite3

ROOT=Path(__file__).resolve().parents[1]
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def save(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
def main():
    freeze=ROOT/'docs/m3-integration-freeze.json';m=json.loads(freeze.read_text());state=ROOT/m['state'];base=ROOT/m['arms'][0]['input']
    terminal=json.loads((state/'terminal.json').read_text())
    if terminal['status']!='halted-review-required':raise ValueError('unexpected terminal status')
    runtime=json.loads((ROOT/'docs/m3-integration-terminal-runtime.json').read_text())
    if runtime['status']!='quiescent':raise ValueError('runtime not checked')
    for p,h in m['sha256'].items():
        if sha(ROOT/p)!=h:raise ValueError('frozen input changed')
    spec=importlib.util.spec_from_file_location('budget',ROOT/'scripts/m3-exact-budget.py');budget=importlib.util.module_from_spec(spec);spec.loader.exec_module(budget)
    baseline=json.loads((ROOT/m['baseline']).read_text());ledger=(ROOT/m['ledger']).read_bytes();total=budget.check(ledger,baseline,dispatch=False)
    new=[json.loads(l) for l in ledger[baseline['bytes']:].splitlines()];reserves=[r for r in new if r['kind']=='reserve'];settled=[r for r in new if r['kind']=='settled']
    if len(reserves)!=1 or len(settled)!=1 or reserves[0]['id']!=settled[0]['id']:raise ValueError('unexpected stage accounting')
    evidence=state/'evidence-0';diag=json.loads((evidence/'diagnostics.json').read_text())
    if len(diag)!=1 or diag[0]['reservation_id']!=reserves[0]['id'] or diag[0]['settled'] is not True:raise ValueError('diagnostic accounting mismatch')
    task=m['arms'][0]['task'];label=task[:12]+'-record'
    result=json.loads((evidence/(label+'-results.jsonl')).read_text())
    with sqlite3.connect((evidence/(label+'-state.sqlite')).as_uri()+'?mode=ro&immutable=1',uri=True) as db:
        events=[json.loads(r[0])['kind'] for r in db.execute('select payload from events order by seq')]
    proposals=[e for e in events if e['kind']=='proposed'];closures=[e['reason'] for e in events if e['kind']=='task_closed']
    if result['outcome']!='unavailable' or proposals or result.get('oracle') is not None or (state/'1-intent.json').exists():raise ValueError('unexpected stage outcome')
    export=ROOT/'docs/m3-integration-execution';shutil.copytree(state,export)
    inputs=export/'inputs';inputs.mkdir()
    for name in ['tasks.jsonl','profile.toml','preparation.json','broker-contexts.json','preparation-request.json','synthetic.lock']:shutil.copyfile(base/name,inputs/name)
    shutil.copytree(base/'cas',inputs/'cas')
    for name in ['m3-integration-freeze.json','m3-integration-baseline.json','m3-integration-terminal-runtime.json']:shutil.copyfile(ROOT/'docs'/name,export/name)
    (export/'m3-provider-cost.jsonl').write_bytes(ledger)
    summary=dict(status='blocked-provider-contract',task=task,synthetic_only=True,qualified_corpus_tasks_added=0,freeze_sha256=sha(freeze),frozen_inputs_verified=len(m['sha256']),record=dict(outcome=result['outcome'],proposals=len(proposals),oracle_invocations=0,closure_reasons=closures,process=json.loads((state/(label+'.json')).read_text()),diagnostic=diag[0]),rules=dict(status='not_attempted',intent_exists=False),spending=dict(stage_usd=str(total-Decimal(baseline['settled_usd'])),aggregate_usd=str(total),outstanding_usd='0',requests=len(reserves)),blocker='The first live response omitted required /arguments/id. No adapter/schema defect demonstrated; no retry permitted by the approved stage rule.',comparison='Not frozen or launched: prerequisite live integration path has not succeeded.',checked_at=datetime.datetime.now(datetime.timezone.utc).isoformat())
    save(ROOT/'docs/m3-integration-result.json',summary)
    save(ROOT/'docs/m3-integration-export-sha256.json',{str(p.relative_to(export)):sha(p) for p in export.rglob('*') if p.is_file()})
    print(json.dumps(dict(status=summary['status'],spending=summary['spending'],export_files=sum(p.is_file() for p in export.rglob('*')))))
if __name__=='__main__':main()
