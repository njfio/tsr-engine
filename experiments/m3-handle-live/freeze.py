"""Offline preparation only; never creates a live controller lease or dispatches."""
from pathlib import Path
import hashlib,json,shutil,sys
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    if (ROOT/'target/m3-handle-live-state').exists():raise ValueError('cohort already attempted; refuse re-freeze')
    original=json.loads((ROOT/'docs/m3-fixed12-freeze.json').read_text())
    inventory=json.loads((ROOT/'docs/m3-handle-live-payload-inventory.json').read_text())
    m={k:original[k] for k in ('python','ledger','settings','oracle','source_objects')}
    m.update(version=1,stage='six-pair-handle-interface',engine='target/m3-exact-build/debug/tsr',baseline='docs/m3-handle-live-budget-baseline.json',launch_baseline='docs/m3-handle-live-budget-baseline.json',state='target/m3-handle-live-state',initial_minimum_free_gib=240,live_guard_gib=200,global_wall_seconds=21600,max_calls_total=240,max_tokens_total=2400000,incremental_cap_usd='5',aggregate_cap_usd='100',automatic_retries=0,arms=[],sha256={})
    for i,interface in enumerate(['old','new','new','old','old','new','new','old','old','new','new','old']):
        broker=HERE/f'arm-{i}.py'
        broker.write_text('from pathlib import Path\nimport sys\nsys.path.insert(0,str(Path(__file__).resolve().parent))\nfrom live_core import main\nmain('+str(i)+',Path(__file__).resolve())\n')
        base=ROOT/f'target/m3-handle-live-inputs/{i}'
        if (base/'record').exists() or list(base.glob('startup-*')):raise ValueError('live input is not fresh')
        base.mkdir(parents=True,exist_ok=True)
        for name in ('tasks.jsonl','profile.toml','broker-contexts.json','preparation.json','cas'):
            source=ROOT/'target/m3-fixed12-inputs/2'/name;dest=base/name
            if not dest.exists():
                if source.is_dir():shutil.copytree(source,dest)
                else:shutil.copyfile(source,dest)
        m['arms'].append(dict(index=i,pair_index=i//2,interface=interface,task=inventory['task'],file=inventory['file'],mode='record',model='deepseek-v4.1-flash',broker=str(broker.relative_to(ROOT)),input=str(base.relative_to(ROOT)),engine='target/m3-handle-replay-build/debug/tsr' if interface=='new' else m['engine'],sandbox_image=inventory['sandbox_image']))
    # Preserve existing pin set including external task-source objects and preparation artifacts.
    m['sha256'].update(original['sha256'])
    prior=json.loads((ROOT/'docs/m3-inspection-handle-replay-freeze.json').read_text())
    m['sha256'].update(prior.get('sha256',{}))
    files=list(HERE.glob('*.py'))+[ROOT/'target/m3-handle-replay-build/debug/tsr',ROOT/'target/m3-handle-replay-audit',ROOT/'experiments/m3-inspection-handle-replay/broker.py',ROOT/m['baseline']]
    files+=list((ROOT/'target/m3-handle-live-inputs').rglob('*'))
    files+=list((ROOT/'docs/m3-handle-live-offline-evidence').rglob('*'))
    files+=list((ROOT/'target/m3-handle-live-offline-native-v3').glob('*/body-*.json'))
    files += [HERE/'README.md',ROOT/'docs/m3-handle-live-readiness.md',ROOT/'docs/m3-handle-live-validation-plan.md',ROOT/'docs/m3-handle-live-payload-inventory.json']
    for p in files:
        if p.is_file():m['sha256'][str(p.relative_to(ROOT))]=sha(p)
    path=ROOT/'docs/m3-handle-live-execution-freeze.json';path.write_text(json.dumps(m,indent=2,sort_keys=True)+'\n')
    assert all(sha(ROOT/p)==h for p,h in m['sha256'].items())
    print(f'Frozen {len(m["sha256"])} files; 12 fresh inputs; no lease or dispatch.')
if __name__=='__main__':main()
