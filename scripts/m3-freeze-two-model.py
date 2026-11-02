"""Create immutable six-attempt inputs and manifest. Never starts paid inference."""
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
    old=json.loads((ROOT/'docs/m3-integration-v3-freeze.json').read_text())
    spec=importlib.util.spec_from_file_location('b',ROOT/'scripts/m3-two-model-budget.py');b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
    raw=(ROOT/old['ledger']).read_bytes();baseline=b.history.freeze(raw)
    if baseline['settled_usd']!='0.231086':raise ValueError('unexpected launch accounting')
    save(ROOT/'docs/m3-two-model-baseline.json',baseline)
    m=old.copy();m.update(stage='six-attempt-synthetic-model-comparison',broker='scripts/m3-two-model-broker.py',controller='scripts/m3-two-model-controller.py',state='target/m3-two-model-state',baseline='docs/m3-two-model-baseline.json',launch_baseline='docs/m3-two-model-baseline.json',max_calls_total=120,arms=[],settings=dict(max_output_tokens=16384,temperature='omitted-provider-default',reasoning='omitted-provider-default',tool_choice='required propose',routing='existing transport defaults; no model-lock header'),comparison_limitations='Synthetic trivial task only; three attempts per model; provider defaults/routing not independently controlled. No inference from visible-test success alone.')
    for key in ('prior_stage_requests','prior_stage_spend_usd'):m.pop(key,None)
    source=ROOT/old['arms'][0]['input']
    for i,model in enumerate(['deepseek-v4.1-flash','glm-5.3-flash']*3):
        base=ROOT/f'target/m3-two-model-input-{i}';base.mkdir()
        for name in ('tasks.jsonl','profile.toml','broker-contexts.json','preparation.json'):shutil.copyfile(source/name,base/name)
        shutil.copytree(source/'cas',base/'cas')
        m['arms'].append(old['arms'][0]|dict(index=i,model=model,input=str(base.relative_to(ROOT))))
    files=set(old['sha256'])
    for arm in m['arms']:files.update(str(p.relative_to(ROOT)) for p in (ROOT/arm['input']).rglob('*') if p.is_file())
    files.update(str(p.relative_to(ROOT)) for p in (ROOT/'scripts').glob('*two_model*') if p.is_file())
    files.update(str(p.relative_to(ROOT)) for p in (ROOT/'scripts').glob('*two-model*') if p.is_file())
    files.update(['docs/m3-two-model-catalog.json','docs/m3-two-model-baseline.json'])
    m['sha256']={p:sha(ROOT/p) for p in sorted(files)}
    save(ROOT/'docs/m3-two-model-freeze.json',m)
    print(json.dumps(dict(files=len(files),baseline=baseline,freeze_sha256=sha(ROOT/'docs/m3-two-model-freeze.json'))))
if __name__=='__main__':main()
