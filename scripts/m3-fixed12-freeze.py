"""Freeze selected paired inputs and implementation before offline/native review."""
import hashlib
import json
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    m=json.loads((ROOT/'docs/m3-fixed12-plan.json').read_text());prior=json.loads((ROOT/'docs/m3-two-model-v2-freeze.json').read_text());files=set(prior['sha256']);sources=set()
    for arm in m['arms']:
        base=ROOT/arm['input'];files.update(str(p.relative_to(ROOT)) for p in base.rglob('*') if p.is_file())
        files.add(arm['admission']);files.add(str(Path(arm['admission']).with_name('tasks.jsonl')))
        t=json.loads((base/'tasks.jsonl').read_text());sources.add((t['repo'],t['base_snapshot']))
        if arm['preparation_id']!=t['preparation_id']:raise ValueError('preparation mismatch')
    m['source_objects']=[dict(repo=r,object=o,type=subprocess.check_output(['git','-C',r,'cat-file','-t',o],text=True,timeout=20).strip()) for r,o in sorted(sources)]
    files.update(str(p.relative_to(ROOT)) for p in (ROOT/'scripts').glob('*fixed12*') if p.is_file())
    files.update(['docs/m3-fixed12-selection.json','docs/m3-fixed12-v2-selection.json','docs/m3-fixed12-plan.json','docs/m3-fixed12-baseline.json','docs/m3-fixed12-analysis-plan.md','docs/m3-corpus-progress.json','docs/m3-two-model-v2-catalog.json'])
    m.update(revision=1,status='frozen-for-offline-preflight-and-review-no-paid-launch',catalog='docs/m3-two-model-v2-catalog.json',sha256={p:sha(ROOT/p) for p in sorted(files)})
    dest=ROOT/'docs/m3-fixed12-freeze.json'
    with dest.open('x') as f:json.dump(m,f,indent=2);f.write('\n')
    print(json.dumps(dict(freeze_sha256=sha(dest),pinned_files=len(files),tasks=12,arms=24)))
if __name__=='__main__':main()
