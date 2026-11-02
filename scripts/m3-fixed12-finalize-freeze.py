"""Preserve first freeze, then pin reviewed controller and completed offline evidence."""
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    p=ROOT/'docs/m3-fixed12-freeze.json';old=p.read_bytes();m=json.loads(old)
    assert m['revision']==1
    with (ROOT/'docs/m3-fixed12-freeze-v1.json').open('xb') as f:f.write(old)
    for name,h in m['sha256'].items():assert sha(ROOT/name)==h,name
    m['revision']=2;m['previous_freeze_sha256']=hashlib.sha256(old).hexdigest()
    m['controller']='scripts/m3-fixed12-v6-controller.py'
    m['implementation_base_commit']=m.pop('implementation_commit');m.pop('sandbox_image',None)
    m['implementation_identity']='Exact file SHA256 pins; base commit is not the commit containing these new files.'
    m['comparison_limitations']='Twelve constructed mutation tasks from two real repositories; source-size-compatible stratified sample, prior pilot reuse disclosed, not a historical bug-fix or untouched holdout. Provider defaults/routing/upstream revision not independently pinned. Offline checks do not validate real repairs or full-suite oracle outcomes.'
    m['status']='offline-preflight-complete-no-paid-launch-this-turn'
    m['deadline_semantics']='Dispatch/acceptance ceiling, including checks immediately before run and after export; not a hard interruption of filesystem operations.'
    evidence=ROOT/'docs/m3-fixed12-native-evidence'
    summary=json.loads((evidence/'summary.json').read_text());assert len(summary['checks'])==36 and summary['provider_calls']==0
    m['native_preflight']='docs/m3-fixed12-native-evidence/summary.json'
    zero=json.loads((evidence/'zero-native/summary.json').read_text());assert zero['status']=='passed' and zero['provider_calls']==0
    m['native_zero_response_regression']='docs/m3-fixed12-native-evidence/zero-native/summary.json'
    files=set(m['sha256'])
    files.update(str(f.relative_to(ROOT)) for f in (ROOT/'scripts').glob('*fixed12*.py'))
    files.update(str(f.relative_to(ROOT)) for f in (ROOT/'docs/m3-validation').glob('m3-fixed12-*.log'))
    files.update(str(f.relative_to(ROOT)) for f in evidence.rglob('*') if f.is_file())
    files.update(['scripts/m3-fixed12-v2-controller.py','scripts/m3-fixed12-v3-controller.py','scripts/test_m3_fixed12_v3.py','scripts/m3-fixed12-v3-replay.py','scripts/m3-fixed12-export-preflight.py','scripts/m3-fixed12-finalize-freeze.py','docs/m3-fixed12-prior-exposure.json','docs/m3-fixed12-readiness.md','docs/m3-fixed12-freeze-v1.json','docs/m3-validation/m3-fixed12-unit.log','docs/m3-validation/m3-fixed12-v2-unit.log','docs/m3-validation/m3-fixed12-v3-unit.log','docs/m3-validation/m3-fixed12-native.log','docs/m3-validation/m3-fixed12-v6-replay.log'])
    m['sha256']={n:sha(ROOT/n) for n in sorted(files)}
    p.write_text(json.dumps(m,indent=2)+'\n')
    print(json.dumps(dict(sha256=sha(p),pins=len(files))))
if __name__=='__main__':main()
