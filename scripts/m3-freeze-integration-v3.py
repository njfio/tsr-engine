"""Freeze fresh corrected wire integration without resetting the cumulative stage budget."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    old=json.loads((ROOT/'docs/m3-integration-freeze.json').read_text())
    base=ROOT/'target/m3-integration-v3-input';base.mkdir()
    source=ROOT/old['arms'][0]['input']
    for name in ('tasks.jsonl','profile.toml','broker-contexts.json','preparation.json'):shutil.copyfile(source/name,base/name)
    shutil.copytree(source/'cas',base/'cas')
    spec=importlib.util.spec_from_file_location('b',ROOT/'scripts/m3-exact-budget.py');b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
    raw=(ROOT/old['ledger']).read_bytes();baseline=json.loads((ROOT/old['baseline']).read_text());b.check(raw,baseline)
    launch=b.history.freeze(raw)
    if launch['settled_usd']!='0.229988':raise ValueError('unexpected current total')
    launch_path=ROOT/'docs/m3-integration-v3-launch-baseline.json'
    with launch_path.open('x') as f:json.dump(launch,f,indent=2);f.write('\n')
    m=old.copy();m.update(broker='scripts/m3-integration-v3-broker.py',controller='scripts/m3-integration-v3-controller.py',state='target/m3-integration-v3-state',launch_baseline=str(launch_path.relative_to(ROOT)),prior_stage_requests=1,prior_stage_spend_usd='0.000244',wire_contract='exact-edit@v3',automatic_retries=0)
    m['arms']=[a|{'input':str(base.relative_to(ROOT))} for a in old['arms']]
    files=set(old['sha256'])
    files.update(str(p.relative_to(ROOT)) for p in base.rglob('*') if p.is_file())
    files.update(['scripts/m3-edit-v3-broker.py','scripts/m3-integration-v3-broker.py','scripts/m3-integration-v3-controller.py','scripts/m3-integration-v3-local-check.py','scripts/test_m3_edit_v3.py','scripts/m3-freeze-integration-v3.py','docs/m3-integration-v3-launch-baseline.json','docs/m3-integration-result.json','docs/m3-integration-export-sha256.json'])
    m['sha256']={p:sha(ROOT/p) for p in sorted(files)}
    dest=ROOT/'docs/m3-integration-v3-freeze.json'
    with dest.open('x') as f:json.dump(m,f,indent=2);f.write('\n')
    print(json.dumps(dict(freeze_sha256=sha(dest),files=len(files),stage_baseline=baseline['settled_usd'],launch_baseline=launch['settled_usd'],remaining_stage_requests=39)))
if __name__=='__main__':main()
