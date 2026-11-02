"""Export terminal comparison evidence and accounting without further inference."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
def main():
    s=importlib.util.spec_from_file_location('c',ROOT/'scripts/m3-two-model-v2-controller.py');c=importlib.util.module_from_spec(s);s.loader.exec_module(c)
    m=json.loads((ROOT/'docs/m3-two-model-v2-freeze.json').read_text());ops=c.Operations(m);ops.guard()
    state=ROOT/m['state'];terminal=json.loads((state/'terminal.json').read_text());dest=ROOT/'docs/m3-two-model-v2-execution'
    if dest.exists():raise ValueError('export exists')
    for p in state.rglob('*'):
        if p.is_symlink() or not (p.is_dir() or p.is_file()):raise ValueError('unsafe evidence entry')
    shutil.copytree(state,dest)
    for name in ('m3-two-model-v2-freeze.json','m3-two-model-baseline.json','m3-two-model-catalog.json'):shutil.copyfile(ROOT/'docs'/name,dest/name)
    raw=ops.ledger();(dest/'ledger.jsonl').write_bytes(raw);baseline=json.loads((ROOT/m['baseline']).read_text());used=c.budget.check(raw,baseline,dispatch=False)
    delta=raw[baseline['bytes']:];reserves=[json.loads(l) for l in delta.splitlines() if json.loads(l)['kind']=='reserve']
    rows=terminal['results'];counts={}
    for model in ('deepseek-v4.1-flash','glm-5.3-flash'):
        selected=[r for r in rows if r['model']==model]
        counts[model]=dict(attempts=len(selected),accepted=sum(r['final_oracle_success'] for r in selected),oracle_completed=sum(r['final_oracle_completed'] for r in selected),schema_failures=sum(r['schema_failures'] for r in selected),stale_digest_failures=sum(r['stale_digest_failures'] for r in selected),requests=sum(r['requests'] for r in selected),latency_seconds=sum(r['latency_seconds'] for r in selected),settled_usd=str(sum((c.budget.D(r['settled_usd']) for r in selected),c.budget.D(0))))
    summary=dict(status=terminal['status'],attempts=rows,by_model=counts,requests=len(reserves),additional_usd=str(used-c.budget.D(baseline['settled_usd'])),aggregate_usd=str(used),outstanding_usd='0',synthetic_only=True,qualified_corpus_tasks_added=0,launched_attempts=len(list(state.glob('*/intent.json'))),unlaunched_attempts=6-len(list(state.glob('*/intent.json'))),recorded_model_outcomes=len(rows),terminal_reason=terminal.get('reason'),freeze_sha256=sha(ROOT/'docs/m3-two-model-v2-freeze.json'))
    if terminal['status']=='completed' and (len(rows)!=6 or len(reserves)!=sum(r['requests'] for r in rows)):raise ValueError('terminal request reconciliation')
    save(ROOT/'docs/m3-two-model-v2-result.json',summary)
    save(ROOT/'docs/m3-two-model-v2-export-sha256.json',{str(p.relative_to(dest)):sha(p) for p in dest.rglob('*') if p.is_file()})
    print(json.dumps(summary))
if __name__=='__main__':main()
