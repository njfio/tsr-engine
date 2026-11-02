"""Freeze the minimal environment-compatible revision; preserve the failed launch."""
import hashlib
import json
from pathlib import Path
import shutil
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    old=json.loads((ROOT/'docs/m3-two-model-freeze.json').read_text())
    for p,h in old['sha256'].items():
        if sha(ROOT/p)!=h:raise ValueError('old frozen input changed')
    baseline=json.loads((ROOT/old['baseline']).read_text());raw=(ROOT/old['ledger']).read_bytes()
    if len(raw)!=baseline['bytes'] or hashlib.sha256(raw).hexdigest()!=baseline['sha256']:raise ValueError('failed launch incurred requests')
    m=old.copy();m.update(revision=3,controller='scripts/m3-two-model-v2-controller.py',state='target/m3-two-model-v2-state',review_corrections='Static controller-owned attempt entrypoints survive native env_clear; environment isolation unchanged.',prior_failed_revision='docs/m3-two-model-freeze.json',arms=[])
    m.pop('broker',None)
    for i,a in enumerate(old['arms']):
        base=ROOT/f'target/m3-two-model-v2-input-{i}';base.mkdir()
        source=ROOT/a['input']
        for name in ('tasks.jsonl','profile.toml','broker-contexts.json','preparation.json'):shutil.copyfile(source/name,base/name)
        shutil.copytree(source/'cas',base/'cas')
        m['arms'].append(a|dict(input=str(base.relative_to(ROOT)),broker=f'scripts/m3-two-model-v2-arm-{i}.py'))
    files=set(old['sha256'])
    for arm in m['arms']:files.update(str(p.relative_to(ROOT)) for p in (ROOT/arm['input']).rglob('*') if p.is_file())
    files.update(str(p.relative_to(ROOT)) for p in (ROOT/'scripts').glob('*two-model-v2*.py'))
    files.update(['scripts/m3_two_model_v2_core.py','docs/m3-two-model-v2-catalog.json','docs/m3-two-model-freeze.json','docs/m3-two-model-result.json','docs/m3-two-model-export-sha256.json'])
    m['sha256']={p:sha(ROOT/p) for p in sorted(files)}
    dest=ROOT/'docs/m3-two-model-v2-freeze.json'
    with dest.open('x') as f:json.dump(m,f,indent=2);f.write('\n')
    print(json.dumps(dict(freeze_sha256=sha(dest),inputs=len(files),baseline=baseline)))
if __name__=='__main__':main()
