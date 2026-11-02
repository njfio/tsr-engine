"""Prepare amendment only; no paid dispatch, no modification of original terminal."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    prior=ROOT/'docs/m3-handle-live-execution-freeze.json';m=json.loads(prior.read_text())
    state=ROOT/'target/m3-handle-live-amendment-v1-state'
    if state.exists():raise ValueError('amendment already attempted')
    m.update(amendment=1,state=str(state.relative_to(ROOT)),observer=str((HERE/'observer.py').relative_to(ROOT)),prior_terminal='target/m3-handle-live-state/terminal.json',prior_decision='target/m3-handle-live-state/0/decision.json',prior_evidence='target/m3-handle-live-state/0/evidence',launch_baseline='docs/m3-handle-live-amendment-v1-launch-baseline.json',absolute_deadline_epoch=datetime(2026,10,18,19,45,tzinfo=timezone.utc).timestamp())
    import importlib.util
    spec=importlib.util.spec_from_file_location('budget',ROOT/'experiments/m3-handle-live/budget.py');b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
    raw=(ROOT/m['ledger']).read_bytes();assert raw==(ROOT/m['prior_evidence']/'ledger.jsonl').read_bytes();baseline=b.history.freeze(raw)
    (ROOT/m['launch_baseline']).write_text(json.dumps(baseline,indent=2)+'\n')
    for i,a in enumerate(m['arms']):
        if i==0:continue
        base=ROOT/a['input'];assert not (base/'record').exists() and not list(base.glob('startup-*'))
        wrapper=HERE/f'arm-{i}.py';wrapper.write_text('from pathlib import Path\nimport sys\nROOT=Path(__file__).resolve().parents[2]\nsys.path.insert(0,str(ROOT/"experiments/m3-handle-live"))\nimport live_core\nlive_core.MANIFEST=ROOT/"docs/m3-handle-live-amendment-v1.json"\nlive_core.main('+str(i)+',Path(__file__).resolve())\n');a['broker']=str(wrapper.relative_to(ROOT))
    files=list(HERE.glob('*.py'))+[prior,ROOT/m['launch_baseline']]
    files+=list((ROOT/'target/m3-handle-live-state').rglob('*'))
    files += [Path('/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python')]
    for p in files:
        if p.is_file():m['sha256'][str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p)]=sha(p)
    assert all(sha(ROOT/p)==h for p,h in m['sha256'].items())
    (ROOT/'docs/m3-handle-live-amendment-v1.json').write_text(json.dumps(m,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(pins=len(m['sha256']),prior_settled_usd=baseline['settled_usd'],remaining_assignments=11,deadline_utc='2026-10-18T19:45:00Z')))
if __name__=='__main__':main()
